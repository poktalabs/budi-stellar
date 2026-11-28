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

#[contract]
pub struct Examen;

#[contractimpl]
impl Examen {
    /// `huella` es el sha256 de las respuestas correctas unidas (por ejemplo "bca").
    pub fn __constructor(env: Env, insignias: Address, ruta: Symbol, huella: BytesN<32>) {
        let s = env.storage().instance();
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

    /// El id de la insignia que ganó `alumno` con este examen, si ya aprobó.
    pub fn aprobado(env: Env, alumno: Address) -> Option<u32> {
        env.storage().persistent().get(&Clave::Aprobado(alumno))
    }

    /// La ruta que acredita este examen.
    pub fn ruta(env: Env) -> Symbol {
        env.storage().instance().get(&Clave::Ruta).unwrap()
    }
}

#[cfg(test)]
mod test;
