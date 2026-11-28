//! Cuenta: la cuenta de un agente, en la línea de ERC-6551 (token bound accounts).
//!
//! Cada agente registrado en `Agentes` puede tener su propia cuenta en Stellar: un
//! contrato con dirección propia que recibe tokens y actúa en otros contratos. No
//! tiene dueño fijo: la controla quien sea dueño del agente en ese momento, así que
//! si el agente cambia de manos, la cuenta (con su saldo y su historial) se va con él.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, Symbol, Val, Vec,
};

#[contracttype]
enum Clave {
    Agentes,
    Agente,
}

#[contractclient(name = "AgentesClient")]
pub trait AgentesApi {
    fn dueno(env: Env, id: u32) -> Address;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MontoInvalido = 1,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pagado {
    #[topic]
    pub token: Address,
    pub a: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ejecutado {
    #[topic]
    pub contrato: Address,
    pub funcion: Symbol,
}

#[contract]
pub struct Cuenta;

#[contractimpl]
impl Cuenta {
    pub fn __constructor(env: Env, agentes: Address, agente: u32) {
        env.storage().instance().set(&Clave::Agentes, &agentes);
        env.storage().instance().set(&Clave::Agente, &agente);
    }

    /// El dueño actual del agente paga desde la cuenta del agente.
    pub fn pagar(env: Env, token: Address, a: Address, monto: i128) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &a, &monto);
        Pagado { token, a, monto }.publish(&env);
        Ok(())
    }

    /// El dueño actual del agente hace que la cuenta llame a otro contrato, como si la
    /// cuenta fuera quien firma (por ejemplo, contratar a un equipo o dar feedback).
    pub fn ejecutar(env: Env, contrato: Address, funcion: Symbol, args: Vec<Val>) -> Val {
        Self::dueno(env.clone()).require_auth();
        let r: Val = env.invoke_contract(&contrato, &funcion, args);
        Ejecutado { contrato, funcion }.publish(&env);
        r
    }

    /// Quién controla la cuenta hoy: el dueño del agente.
    pub fn dueno(env: Env) -> Address {
        let agentes: Address = env.storage().instance().get(&Clave::Agentes).unwrap();
        let agente: u32 = env.storage().instance().get(&Clave::Agente).unwrap();
        AgentesClient::new(&env, &agentes).dueno(&agente)
    }

    pub fn agente(env: Env) -> (Address, u32) {
        (
            env.storage().instance().get(&Clave::Agentes).unwrap(),
            env.storage().instance().get(&Clave::Agente).unwrap(),
        )
    }
}
