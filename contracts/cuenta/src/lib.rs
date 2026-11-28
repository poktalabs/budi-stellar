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

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagoEnLote {
    #[topic]
    pub token: Address,
    pub pagos: u32,
    pub total: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GastoAprobado {
    #[topic]
    pub token: Address,
    pub gastador: Address,
    pub monto: i128,
    pub hasta_ledger: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Retirado {
    #[topic]
    pub token: Address,
    pub a: Address,
    pub monto: i128,
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

    /// El dueño hace que la cuenta llame a varios contratos en una sola transacción
    /// (todo o nada). Regresa lo que regresó cada llamada.
    pub fn ejecutar_lote(env: Env, llamadas: Vec<(Address, Symbol, Vec<Val>)>) -> Vec<Val> {
        Self::dueno(env.clone()).require_auth();
        let mut res = Vec::new(&env);
        for (contrato, funcion, args) in llamadas.iter() {
            let r: Val = env.invoke_contract(&contrato, &funcion, args);
            Ejecutado { contrato, funcion }.publish(&env);
            res.push_back(r);
        }
        res
    }

    /// El dueño paga a varios a la vez desde la cuenta del agente (por ejemplo, a los
    /// humanos que ayudaron en un reto). Regresa el total pagado.
    pub fn pagar_lote(env: Env, token: Address, pagos: Vec<(Address, i128)>) -> Result<i128, Error> {
        Self::dueno(env.clone()).require_auth();
        if pagos.is_empty() {
            return Err(Error::MontoInvalido);
        }
        let mut total: i128 = 0;
        for (_, m) in pagos.iter() {
            if m <= 0 {
                return Err(Error::MontoInvalido);
            }
            total += m;
        }
        let t = token::Client::new(&env, &token);
        for (a, m) in pagos.iter() {
            t.transfer(&env.current_contract_address(), &a, &m);
        }
        PagoEnLote { token, pagos: pagos.len(), total }.publish(&env);
        Ok(total)
    }

    /// El dueño deja que `gastador` (por ejemplo, un servicio que el agente usa) cobre
    /// hasta `monto` de la cuenta antes del ledger `hasta_ledger`.
    pub fn aprobar_gasto(env: Env, token: Address, gastador: Address, monto: i128, hasta_ledger: u32) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        if monto < 0 {
            return Err(Error::MontoInvalido);
        }
        token::Client::new(&env, &token).approve(&env.current_contract_address(), &gastador, &monto, &hasta_ledger);
        GastoAprobado { token, gastador, monto, hasta_ledger }.publish(&env);
        Ok(())
    }

    /// El dueño vacía la cuenta del agente hacia `a`. Regresa lo retirado.
    pub fn retirar_todo(env: Env, token: Address, a: Address) -> Result<i128, Error> {
        Self::dueno(env.clone()).require_auth();
        let t = token::Client::new(&env, &token);
        let monto = t.balance(&env.current_contract_address());
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        t.transfer(&env.current_contract_address(), &a, &monto);
        Retirado { token, a, monto }.publish(&env);
        Ok(monto)
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
