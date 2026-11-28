//! Cuentas: el registro de cuentas de agentes, en la línea del registro de ERC-6551.
//!
//! `crear_cuenta` despliega el contrato `Cuenta` de un agente en una dirección que
//! depende solo del registro de agentes y del id, así que cualquiera puede saber de
//! antemano cuál es la cuenta de un agente (`cuenta_de`) y mandarle tokens.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    Bytes, BytesN, Env,
};

#[contracttype]
enum Clave {
    Wasm,
}

#[contractclient(name = "AgentesClient")]
pub trait AgentesApi {
    fn dueno(env: Env, id: u32) -> Address;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    YaExiste = 1,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuentaCreada {
    #[topic]
    pub agentes: Address,
    #[topic]
    pub agente: u32,
    pub cuenta: Address,
}

#[contract]
pub struct Cuentas;

#[contractimpl]
impl Cuentas {
    /// `wasm` es el hash del contrato `Cuenta` ya subido a la red.
    pub fn __constructor(env: Env, wasm: BytesN<32>) {
        env.storage().instance().set(&Clave::Wasm, &wasm);
    }

    /// Crea la cuenta del agente `agente` del registro `agentes`. Cualquiera la puede
    /// crear (no da control a quien la crea: la controla el dueño del agente). El
    /// agente tiene que existir. Regresa la dirección de la cuenta.
    pub fn crear_cuenta(env: Env, agentes: Address, agente: u32) -> Result<Address, Error> {
        // Falla si el agente no existe.
        AgentesClient::new(&env, &agentes).dueno(&agente);
        let destino = Self::cuenta_de(env.clone(), agentes.clone(), agente);
        if env.storage().persistent().has(&destino) {
            return Err(Error::YaExiste);
        }
        let wasm: BytesN<32> = env.storage().instance().get(&Clave::Wasm).unwrap();
        let cuenta = env
            .deployer()
            .with_current_contract(sal(&env, &agentes, agente))
            .deploy_v2(wasm, (agentes.clone(), agente));
        env.storage().persistent().set(&cuenta, &true);
        CuentaCreada { agentes, agente, cuenta: cuenta.clone() }.publish(&env);
        Ok(cuenta)
    }

    /// La dirección de la cuenta de un agente, exista ya o no.
    pub fn cuenta_de(env: Env, agentes: Address, agente: u32) -> Address {
        env.deployer().with_current_contract(sal(&env, &agentes, agente)).deployed_address()
    }
}

fn sal(env: &Env, agentes: &Address, agente: u32) -> BytesN<32> {
    let mut b = Bytes::new(env);
    b.append(&agentes.to_string().to_bytes());
    b.extend_from_array(&agente.to_be_bytes());
    env.crypto().sha256(&b).into()
}

#[cfg(test)]
mod test;
