//! Examen: el quiz de una lección de Budi para Stellar, con su insignia.
//!
//! El alumno presenta sus respuestas desde el aula y firma. El contrato compara
//! el sha256 de las respuestas con la huella guardada al desplegar (nunca guarda
//! las respuestas) y, si pasa, le otorga la insignia de la ruta en el contrato
//! Insignias. Examen es el emisor de ese contrato: solo se gana pasando el quiz.
//!
//! Regla para pasar (Mel, 27 nov 2026): las 3 preguntas correctas. Una
//! insignia por persona; reintentar está permitido hasta pasar.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    Bytes, BytesN, Env, Symbol,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 3_110_400; // ~180 días

#[contracttype]
enum Clave {
    Insignias,
    Ruta,
    Huella,
    Aprobado(Address),
    Admin,
    Pausado,
    Cierre,
}

/// Lo único que Examen le pide a Insignias.
#[contractclient(name = "InsigniasClient")]
pub trait InsigniasApi {
    fn otorgar(env: Env, titular: Address, ruta: Symbol) -> u32;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Incorrecto = 1,
    YaAprobo = 2,
    Pausado = 3,
    Cerrado = 4,
    MismoAdmin = 5,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExamenAprobado {
    #[topic]
    pub alumno: Address,
    #[topic]
    pub ruta: Symbol,
    pub insignia: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExamenPausado {
    #[topic]
    pub ruta: Symbol,
    pub pausado: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExamenActualizado {
    #[topic]
    pub ruta: Symbol,
    pub huella: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CierreFijado {
    #[topic]
    pub ruta: Symbol,
    pub cierre: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExamenCedido {
    #[topic]
    pub ruta: Symbol,
    pub de: Address,
    pub a: Address,
}

#[contract]
pub struct Examen;

#[contractimpl]
impl Examen {
    /// `huella` es el sha256 de las respuestas correctas unidas (por ejemplo "bca").
    /// `admin` puede pausar el examen y publicar una nueva edición.
    pub fn __constructor(
        env: Env,
        admin: Address,
        insignias: Address,
        ruta: Symbol,
        huella: BytesN<32>,
    ) {
        let s = env.storage().instance();
        s.set(&Clave::Admin, &admin);
        s.set(&Clave::Insignias, &insignias);
        s.set(&Clave::Ruta, &ruta);
        s.set(&Clave::Huella, &huella);
        s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
    }

    /// El alumno firma sus respuestas. Si son correctas, recibe la insignia de la
    /// ruta y regresa su id. Si no, regresa `Incorrecto` y puede intentar otra vez.
    pub fn presentar(env: Env, alumno: Address, respuestas: Bytes) -> Result<u32, Error> {
        alumno.require_auth();
        let s = env.storage().instance();
        s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        if s.get(&Clave::Pausado).unwrap_or(false) {
            return Err(Error::Pausado);
        }
        let cierre: u64 = s.get(&Clave::Cierre).unwrap_or(0);
        if cierre > 0 && env.ledger().timestamp() > cierre {
            return Err(Error::Cerrado);
        }
        if env.storage().persistent().has(&Clave::Aprobado(alumno.clone())) {
            return Err(Error::YaAprobo);
        }
        let ruta: Symbol = s.get(&Clave::Ruta).unwrap();
        let huella: BytesN<32> = s.get(&Clave::Huella).unwrap();
        let hash: BytesN<32> = env.crypto().sha256(&respuestas).into();
        if hash != huella {
            return Err(Error::Incorrecto);
        }
        let insignias: Address = s.get(&Clave::Insignias).unwrap();
        let id = InsigniasClient::new(&env, &insignias).otorgar(&alumno, &ruta);
        let clave = Clave::Aprobado(alumno.clone());
        env.storage().persistent().set(&clave, &id);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        ExamenAprobado { alumno, ruta, insignia: id }.publish(&env);
        Ok(id)
    }

    /// El admin pausa el examen (por ejemplo, mientras corrige una pregunta):
    /// nadie puede presentar hasta que lo reanude.
    pub fn pausar_examen(env: Env) {
        poner_pausa(&env, true);
    }

    /// El admin reanuda un examen pausado.
    pub fn reanudar_examen(env: Env) {
        poner_pausa(&env, false);
    }

    /// El admin publica una nueva edición del examen: reemplaza la huella de las
    /// respuestas correctas. Quien ya aprobó conserva su insignia.
    pub fn actualizar_examen(env: Env, huella: BytesN<32>) {
        let s = env.storage().instance();
        let admin: Address = s.get(&Clave::Admin).unwrap();
        admin.require_auth();
        s.set(&Clave::Huella, &huella);
        s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        let ruta: Symbol = s.get(&Clave::Ruta).unwrap();
        ExamenActualizado { ruta, huella }.publish(&env);
    }

    /// El admin fija la fecha de cierre (segundos unix) de esta edición del examen: después
    /// de esa fecha nadie presenta. 0 la quita.
    pub fn fijar_cierre(env: Env, cierre: u64) {
        let s = env.storage().instance();
        let admin: Address = s.get(&Clave::Admin).unwrap();
        admin.require_auth();
        s.set(&Clave::Cierre, &cierre);
        s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        let ruta: Symbol = s.get(&Clave::Ruta).unwrap();
        CierreFijado { ruta, cierre }.publish(&env);
    }

    /// El admin cede el examen a otra cuenta (por ejemplo, al mentor de la cohorte).
    /// Firman los dos, para no cederlo a una cuenta que nadie controla.
    pub fn ceder_examen(env: Env, nuevo: Address) -> Result<(), Error> {
        let s = env.storage().instance();
        let admin: Address = s.get(&Clave::Admin).unwrap();
        admin.require_auth();
        nuevo.require_auth();
        if nuevo == admin {
            return Err(Error::MismoAdmin);
        }
        s.set(&Clave::Admin, &nuevo);
        s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        let ruta: Symbol = s.get(&Clave::Ruta).unwrap();
        ExamenCedido { ruta, de: admin, a: nuevo }.publish(&env);
        Ok(())
    }

    /// La fecha de cierre (0 = sin cierre).
    pub fn cierre(env: Env) -> u64 {
        env.storage().instance().get(&Clave::Cierre).unwrap_or(0)
    }

    /// Si el examen está pausado.
    pub fn pausado(env: Env) -> bool {
        env.storage().instance().get(&Clave::Pausado).unwrap_or(false)
    }

    /// El id de la insignia que ganó `alumno` con este examen, si ya aprobó.
    pub fn aprobado(env: Env, alumno: Address) -> Option<u32> {
        env.storage().persistent().get(&Clave::Aprobado(alumno))
    }

    /// La ruta que acredita este examen.
    pub fn ruta(env: Env) -> Symbol {
        env.storage().instance().get(&Clave::Ruta).unwrap()
    }
}

fn poner_pausa(env: &Env, pausado: bool) {
    let s = env.storage().instance();
    let admin: Address = s.get(&Clave::Admin).unwrap();
    admin.require_auth();
    s.set(&Clave::Pausado, &pausado);
    s.extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
    let ruta: Symbol = s.get(&Clave::Ruta).unwrap();
    ExamenPausado { ruta, pausado }.publish(env);
}

#[cfg(test)]
mod test;
