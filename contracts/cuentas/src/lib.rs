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
    Heredero(Address),
    Cerrada(Address),
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
    NoExiste = 2,
    Cerrada = 3,
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

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HerederoNombrado {
    #[topic]
    pub cuenta: Address,
    pub heredero: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HerederoQuitado {
    #[topic]
    pub cuenta: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuentaCerrada {
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

    /// El dueño del agente nombra un heredero para la cuenta: la dirección de
    /// recuperación que Budi reconoce si el dueño pierde su llave. Firma el dueño.
    pub fn heredar_cuenta(
        env: Env,
        agentes: Address,
        agente: u32,
        heredero: Address,
    ) -> Result<(), Error> {
        let cuenta = cuenta_viva(&env, &agentes, agente)?;
        AgentesClient::new(&env, &agentes).dueno(&agente).require_auth();
        env.storage().persistent().set(&Clave::Heredero(cuenta.clone()), &heredero);
        HerederoNombrado { cuenta, heredero }.publish(&env);
        Ok(())
    }

    /// El dueño del agente quita al heredero de la cuenta.
    pub fn quitar_heredero(env: Env, agentes: Address, agente: u32) -> Result<(), Error> {
        let cuenta = cuenta_viva(&env, &agentes, agente)?;
        AgentesClient::new(&env, &agentes).dueno(&agente).require_auth();
        env.storage().persistent().remove(&Clave::Heredero(cuenta.clone()));
        HerederoQuitado { cuenta }.publish(&env);
        Ok(())
    }

    /// El dueño del agente da de baja la cuenta en el registro: deja de
    /// anunciarse como la cuenta del agente y no se puede volver a crear.
    pub fn cerrar_cuenta(env: Env, agentes: Address, agente: u32) -> Result<(), Error> {
        let cuenta = cuenta_viva(&env, &agentes, agente)?;
        AgentesClient::new(&env, &agentes).dueno(&agente).require_auth();
        env.storage().persistent().set(&Clave::Cerrada(cuenta.clone()), &true);
        env.storage().persistent().remove(&Clave::Heredero(cuenta.clone()));
        CuentaCerrada { agentes, agente, cuenta }.publish(&env);
        Ok(())
    }

    /// El heredero de la cuenta de un agente, si tiene.
    pub fn heredero(env: Env, agentes: Address, agente: u32) -> Option<Address> {
        let cuenta = Self::cuenta_de(env.clone(), agentes, agente);
        env.storage().persistent().get(&Clave::Heredero(cuenta))
    }

    /// Si la cuenta de un agente fue dada de baja.
    pub fn cerrada(env: Env, agentes: Address, agente: u32) -> bool {
        let cuenta = Self::cuenta_de(env.clone(), agentes, agente);
        env.storage().persistent().has(&Clave::Cerrada(cuenta))
    }

    /// La dirección de la cuenta de un agente, exista ya o no.
    pub fn cuenta_de(env: Env, agentes: Address, agente: u32) -> Address {
        env.deployer().with_current_contract(sal(&env, &agentes, agente)).deployed_address()
    }
}

/// La cuenta del agente, si existe y no está cerrada.
fn cuenta_viva(env: &Env, agentes: &Address, agente: u32) -> Result<Address, Error> {
    let cuenta = env.deployer().with_current_contract(sal(env, agentes, agente)).deployed_address();
    if !env.storage().persistent().has(&cuenta) {
        return Err(Error::NoExiste);
    }
    if env.storage().persistent().has(&Clave::Cerrada(cuenta.clone())) {
        return Err(Error::Cerrada);
    }
    Ok(cuenta)
}

fn sal(env: &Env, agentes: &Address, agente: u32) -> BytesN<32> {
    let mut b = Bytes::new(env);
    b.append(&agentes.to_string().to_bytes());
    b.extend_from_array(&agente.to_be_bytes());
    env.crypto().sha256(&b).into()
}

#[cfg(test)]
mod test;
