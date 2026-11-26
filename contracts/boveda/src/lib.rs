//! Bóveda: un bono que se reclama con un código.
//!
//! Budi deposita XLM en la bóveda con `depositar`. Quien presente el código correcto con
//! `reclamar` se lleva todo lo que haya dentro. El contrato guarda solo el sha256 del código
//! normalizado (minúsculas, sin espacios), así que «Ship It», «ship it» y «SHIPIT» abren la
//! misma bóveda y el código nunca queda escrito en la red. Cada dirección reclama una sola vez.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Bytes,
    BytesN, Env,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
enum Clave {
    Admin,
    Token,
    Codigo,
    Reclamo(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MontoInvalido = 1,
    CodigoIncorrecto = 2,
    YaReclamaste = 3,
    BovedaVacia = 4,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Depositado {
    #[topic]
    pub de: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reclamado {
    #[topic]
    pub quien: Address,
    pub monto: i128,
}

#[contract]
pub struct Boveda;

#[contractimpl]
impl Boveda {
    /// `token` es el token que guarda la bóveda (el contrato de XLM nativo en testnet).
    /// `codigo` es el sha256 del código normalizado: minúsculas y sin espacios.
    pub fn __constructor(env: Env, admin: Address, token: Address, codigo: BytesN<32>) {
        let s = env.storage().instance();
        s.set(&Clave::Admin, &admin);
        s.set(&Clave::Token, &token);
        s.set(&Clave::Codigo, &codigo);
    }

    /// Mete `monto` en la bóveda desde la cuenta `de`. Cualquiera puede llenarla.
    pub fn depositar(env: Env, de: Address, monto: i128) -> Result<i128, Error> {
        de.require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        Self::token(&env).transfer(&de, env.current_contract_address(), &monto);
        Self::extender(&env);
        Depositado { de, monto }.publish(&env);
        Ok(Self::saldo(env))
    }

    /// Con el código correcto, `quien` se lleva todo el saldo. Una vez por dirección.
    pub fn reclamar(env: Env, quien: Address, codigo: Bytes) -> Result<i128, Error> {
        quien.require_auth();
        let esperado: BytesN<32> = env.storage().instance().get(&Clave::Codigo).unwrap();
        let hash: BytesN<32> = env.crypto().sha256(&normalizar(&env, &codigo)).into();
        if hash != esperado {
            return Err(Error::CodigoIncorrecto);
        }
        let clave = Clave::Reclamo(quien.clone());
        if env.storage().persistent().has(&clave) {
            return Err(Error::YaReclamaste);
        }
        let monto = Self::saldo(env.clone());
        if monto <= 0 {
            return Err(Error::BovedaVacia);
        }
        env.storage().persistent().set(&clave, &monto);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        Self::token(&env).transfer(&env.current_contract_address(), &quien, &monto);
        Self::extender(&env);
        Reclamado { quien, monto }.publish(&env);
        Ok(monto)
    }

    /// Lo que hay dentro de la bóveda ahora.
    pub fn saldo(env: Env) -> i128 {
        Self::token(&env).balance(&env.current_contract_address())
    }

    /// Cuánto se llevó `quien` (0 si no ha reclamado).
    pub fn reclamado(env: Env, quien: Address) -> i128 {
        env.storage().persistent().get(&Clave::Reclamo(quien)).unwrap_or(0)
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&Clave::Admin).unwrap()
    }

    fn token(env: &Env) -> token::Client<'_> {
        let id: Address = env.storage().instance().get(&Clave::Token).unwrap();
        token::Client::new(env, &id)
    }

    fn extender(env: &Env) {
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
    }
}

/// Minúsculas ASCII y sin espacios: «Ship It» -> «shipit».
fn normalizar(env: &Env, codigo: &Bytes) -> Bytes {
    let mut out = Bytes::new(env);
    for b in codigo.iter() {
        match b {
            b' ' | b'\t' | b'\n' | b'\r' => {}
            b'A'..=b'Z' => out.push_back(b + 32),
            _ => out.push_back(b),
        }
    }
    out
}

#[cfg(test)]
mod test;
