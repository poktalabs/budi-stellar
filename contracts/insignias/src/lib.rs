//! Insignias: insignias onchain por terminar una ruta de Budi.
//!
//! Cada insignia es un NFT que no se puede transferir: pertenece a quien la
//! ganó y el contrato no tiene función de transferencia. Solo el emisor
//! (Budi) puede otorgarlas o revocarlas. Una insignia abre una habilidad; la
//! reputación en esa habilidad se gana con trabajo y vive en otro lado.
//!
//! Reglas (aceptadas el 24 de noviembre de 2026):
//! - Una insignia por ruta por persona.
//! - Las insignias no caducan. Revocar es aparte y deja rastro público.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, String,
    Symbol, Vec,
};

/// Una persona guarda hasta este número de insignias.
pub const MAX_INSIGNIAS: u32 = 200;

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 3_110_400; // ~180 días

#[contracttype]
enum Clave {
    Emisor,
    Siguiente,
    Insignia(u32),
    DelTitular(Address),
    PorRuta(Address, Symbol),
    UriBase,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Insignia {
    pub id: u32,
    pub titular: Address,
    pub ruta: Symbol,
    /// Momento del ledger en que se otorgó (segundos Unix).
    pub otorgada_en: u64,
    pub revocada: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    YaLaTiene = 1,
    NoExiste = 2,
    YaRevocada = 3,
    DemasiadasInsignias = 4,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsigniaOtorgada {
    #[topic]
    pub titular: Address,
    #[topic]
    pub ruta: Symbol,
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsigniaRevocada {
    #[topic]
    pub titular: Address,
    #[topic]
    pub ruta: Symbol,
    pub id: u32,
    pub motivo: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsigniaRenunciada {
    #[topic]
    pub titular: Address,
    #[topic]
    pub ruta: Symbol,
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UriFijada {
    pub uri_base: String,
}

#[contract]
pub struct Insignias;

#[contractimpl]
impl Insignias {
    /// Fija el emisor una sola vez, al desplegar.
    pub fn __constructor(env: Env, emisor: Address) {
        env.storage().instance().set(&Clave::Emisor, &emisor);
        env.storage().instance().set(&Clave::Siguiente, &1u32);
    }

    /// Otorga la insignia de `ruta` a `titular`. Solo firma el emisor.
    /// Regresa el id de la insignia.
    pub fn otorgar(env: Env, titular: Address, ruta: Symbol) -> Result<u32, Error> {
        Self::emisor(env.clone()).require_auth();
        let por_ruta = Clave::PorRuta(titular.clone(), ruta.clone());
        if env.storage().persistent().has(&por_ruta) {
            return Err(Error::YaLaTiene);
        }
        let del_titular = Clave::DelTitular(titular.clone());
        let mut ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&del_titular)
            .unwrap_or(Vec::new(&env));
        if ids.len() >= MAX_INSIGNIAS {
            return Err(Error::DemasiadasInsignias);
        }

        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap();
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        env.storage()
            .instance()
            .extend_ttl(TTL_UMBRAL, TTL_EXTENDER);

        let insignia = Insignia {
            id,
            titular: titular.clone(),
            ruta: ruta.clone(),
            otorgada_en: env.ledger().timestamp(),
            revocada: false,
        };
        ids.push_back(id);
        Self::guardar(&env, &Clave::Insignia(id), &insignia);
        Self::guardar(&env, &del_titular, &ids);
        Self::guardar(&env, &por_ruta, &id);

        InsigniaOtorgada { titular, ruta, id }.publish(&env);
        Ok(id)
    }

    /// Revoca una insignia (fraude o error). La insignia sigue visible,
    /// marcada como revocada, y el motivo queda en el evento.
    pub fn revocar(env: Env, id: u32, motivo: String) -> Result<(), Error> {
        Self::emisor(env.clone()).require_auth();
        let clave = Clave::Insignia(id);
        let mut insignia: Insignia = env
            .storage()
            .persistent()
            .get(&clave)
            .ok_or(Error::NoExiste)?;
        if insignia.revocada {
            return Err(Error::YaRevocada);
        }
        insignia.revocada = true;
        Self::guardar(&env, &clave, &insignia);
        InsigniaRevocada {
            titular: insignia.titular,
            ruta: insignia.ruta,
            id,
            motivo,
        }
        .publish(&env);
        Ok(())
    }

    /// La insignia con este id.
    pub fn insignia(env: Env, id: u32) -> Result<Insignia, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Insignia(id))
            .ok_or(Error::NoExiste)
    }

    /// Todas las insignias de `titular`, revocadas incluidas, de la más antigua a la más nueva.
    pub fn insignias(env: Env, titular: Address) -> Vec<Insignia> {
        let ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&Clave::DelTitular(titular))
            .unwrap_or(Vec::new(&env));
        let mut todas = Vec::new(&env);
        for id in ids.iter() {
            if let Some(i) = env.storage().persistent().get(&Clave::Insignia(id)) {
                todas.push_back(i);
            }
        }
        todas
    }

    /// El titular renuncia a su insignia (no la quiere en su perfil). Deja de contar como
    /// licencia y sale de su lista; puede volver a ganarla. Firma el titular.
    pub fn renunciar(env: Env, id: u32) -> Result<(), Error> {
        let insignia = Self::insignia(env.clone(), id)?;
        insignia.titular.require_auth();
        if insignia.revocada {
            return Err(Error::YaRevocada);
        }
        let del_titular = Clave::DelTitular(insignia.titular.clone());
        let mut ids: Vec<u32> = env.storage().persistent().get(&del_titular).unwrap_or(Vec::new(&env));
        if let Some(i) = ids.first_index_of(id) {
            ids.remove(i);
        }
        Self::guardar(&env, &del_titular, &ids);
        env.storage().persistent().remove(&Clave::PorRuta(insignia.titular.clone(), insignia.ruta.clone()));
        env.storage().persistent().remove(&Clave::Insignia(id));
        InsigniaRenunciada { titular: insignia.titular, ruta: insignia.ruta, id }.publish(&env);
        Ok(())
    }

    /// Verdadero si `titular` tiene la insignia de `ruta` y no está revocada.
    pub fn tiene(env: Env, titular: Address, ruta: Symbol) -> bool {
        let id: Option<u32> = env
            .storage()
            .persistent()
            .get(&Clave::PorRuta(titular, ruta));
        match id {
            Some(id) => Self::insignia(env, id).is_ok_and(|i| !i.revocada),
            None => false,
        }
    }

    /// Vista NFT: quién tiene la insignia `id`.
    pub fn owner_of(env: Env, id: u32) -> Result<Address, Error> {
        Ok(Self::insignia(env, id)?.titular)
    }

    /// Vista NFT: cuántas insignias activas (no revocadas) tiene `titular`.
    pub fn balance(env: Env, titular: Address) -> u32 {
        Self::insignias(env, titular)
            .iter()
            .filter(|i| !i.revocada)
            .count() as u32
    }

    /// Vista NFT: nombre de la colección.
    pub fn name(env: Env) -> String {
        String::from_str(&env, "Insignias Budi para Stellar")
    }

    /// Vista NFT: símbolo de la colección.
    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "BUDI")
    }

    /// El emisor fija dónde vive la metadata (imagen y descripción) de las insignias, para
    /// que las wallets las muestren. Cada insignia apunta a `uri_base` + su id + ".json".
    pub fn fijar_uri(env: Env, uri_base: String) {
        Self::emisor(env.clone()).require_auth();
        env.storage().instance().set(&Clave::UriBase, &uri_base);
        UriFijada { uri_base }.publish(&env);
    }

    /// Vista NFT: la metadata de la insignia `id` (base fijada por el emisor + id + ".json").
    pub fn token_uri(env: Env, id: u32) -> Result<String, Error> {
        let insignia = Self::insignia(env.clone(), id)?;
        let base: String = env
            .storage()
            .instance()
            .get(&Clave::UriBase)
            .unwrap_or(String::from_str(&env, ""));
        // Dígitos del id, al revés y luego en orden.
        let mut digitos = [0u8; 10];
        let mut n = insignia.id;
        let mut d = 0usize;
        loop {
            digitos[d] = b'0' + (n % 10) as u8;
            d += 1;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        let mut buf = [0u8; 256];
        let largo_base = base.len() as usize;
        if largo_base + d + 5 > buf.len() {
            return Ok(base);
        }
        base.copy_into_slice(&mut buf[..largo_base]);
        for i in 0..d {
            buf[largo_base + i] = digitos[d - 1 - i];
        }
        buf[largo_base + d..largo_base + d + 5].copy_from_slice(b".json");
        Ok(String::from_bytes(&env, &buf[..largo_base + d + 5]))
    }

    /// La dirección que puede otorgar y revocar insignias.
    pub fn emisor(env: Env) -> Address {
        env.storage().instance().get(&Clave::Emisor).unwrap()
    }

    fn guardar<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
        env: &Env,
        clave: &Clave,
        valor: &V,
    ) {
        env.storage().persistent().set(clave, valor);
        env.storage()
            .persistent()
            .extend_ttl(clave, TTL_UMBRAL, TTL_EXTENDER);
    }
}

#[cfg(test)]
mod test;
