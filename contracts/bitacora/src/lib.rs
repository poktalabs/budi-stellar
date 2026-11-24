//! Bitácora: cada builder registra sus hitos onchain.
//!
//! Es el contrato de ejemplo que Budi abre en cada sesión nueva: corto,
//! con autorización, almacenamiento y eventos, para que el entorno pruebe
//! todo el ciclo (compilar, probar, desplegar) desde el primer minuto.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, String, Vec,
};

/// Un hito no puede pasar de este largo (en bytes).
pub const MAX_HITO: u32 = 140;
/// Un builder guarda hasta este número de hitos.
pub const MAX_HITOS: u32 = 100;

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
enum Clave {
    Hitos(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    HitoVacio = 1,
    HitoMuyLargo = 2,
    BitacoraLlena = 3,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoRegistrado {
    #[topic]
    pub builder: Address,
    pub numero: u32,
    pub hito: String,
}

#[contract]
pub struct Bitacora;

#[contractimpl]
impl Bitacora {
    /// Registra un hito para `builder`. Solo el propio builder puede firmar.
    /// Regresa el número del hito (empieza en 1).
    pub fn registrar(env: Env, builder: Address, hito: String) -> Result<u32, Error> {
        builder.require_auth();
        if hito.is_empty() {
            return Err(Error::HitoVacio);
        }
        if hito.len() > MAX_HITO {
            return Err(Error::HitoMuyLargo);
        }
        let clave = Clave::Hitos(builder.clone());
        let mut hitos: Vec<String> = env
            .storage()
            .persistent()
            .get(&clave)
            .unwrap_or(Vec::new(&env));
        if hitos.len() >= MAX_HITOS {
            return Err(Error::BitacoraLlena);
        }
        hitos.push_back(hito.clone());
        let numero = hitos.len();
        env.storage().persistent().set(&clave, &hitos);
        env.storage()
            .persistent()
            .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        HitoRegistrado {
            builder,
            numero,
            hito,
        }
        .publish(&env);
        Ok(numero)
    }

    /// Todos los hitos de `builder`, del primero al último.
    pub fn hitos(env: Env, builder: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&Clave::Hitos(builder))
            .unwrap_or(Vec::new(&env))
    }

    /// Cuántos hitos lleva `builder`.
    pub fn contar(env: Env, builder: Address) -> u32 {
        Self::hitos(env, builder).len()
    }
}

#[cfg(test)]
mod test;
