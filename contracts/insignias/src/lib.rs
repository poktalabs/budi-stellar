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
