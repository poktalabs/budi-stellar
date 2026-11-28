//! Retos: un tablero de recompensas para agentes de IA.
//!
//! Quien publica un reto deja el premio en custodia. Los agentes registrados en
//! `Agentes` se postulan (firma su dueño). El creador elige al ganador y el premio
//! va a la cuenta del agente (su cuenta tipo ERC-6551 en `Cuentas`), no a una
//! persona: el agente cobra, y quien sea dueño del agente controla lo cobrado.
//! Si nadie gana, el creador cancela y recupera el premio.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, String, Vec,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EstadoReto {
    Abierto = 0,
    Pagado = 1,
    Cancelado = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reto {
    pub creador: Address,
    pub token: Address,
    pub premio: i128,
    pub uri: String,
    pub postulantes: Vec<u32>,
    pub ganador: u32,
    pub estado: EstadoReto,
}

#[contracttype]
enum Clave {
    Agentes,
    Cuentas,
    Siguiente,
    Reto(u32),
}

#[contractclient(name = "AgentesClient")]
pub trait AgentesApi {
    fn dueno(env: Env, id: u32) -> Address;
}

#[contractclient(name = "CuentasClient")]
pub trait CuentasApi {
    fn cuenta_de(env: Env, agentes: Address, agente: u32) -> Address;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    RetoNoExiste = 1,
    MontoInvalido = 2,
    RetoCerrado = 3,
    YaPostulado = 4,
    NoPostulado = 5,
    UriVacia = 6,
    EsDelCreador = 7,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetoPublicado {
    #[topic]
    pub creador: Address,
    pub id: u32,
    pub premio: i128,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Postulado {
    #[topic]
    pub reto: u32,
    pub agente: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GanadorElegido {
    #[topic]
    pub reto: u32,
    pub agente: u32,
    pub cuenta: Address,
    pub premio: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetoCancelado {
    #[topic]
    pub reto: u32,
    pub premio: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremioAmpliado {
    #[topic]
    pub reto: u32,
    pub extra: i128,
    pub premio: i128,
}

#[contract]
pub struct Retos;

#[contractimpl]
impl Retos {
    pub fn __constructor(env: Env, agentes: Address, cuentas: Address) {
        env.storage().instance().set(&Clave::Agentes, &agentes);
        env.storage().instance().set(&Clave::Cuentas, &cuentas);
    }

    /// Publica un reto y deja `premio` del `token` en custodia. Regresa el id.
    pub fn publicar_reto(env: Env, creador: Address, token: Address, premio: i128, uri: String) -> Result<u32, Error> {
        creador.require_auth();
        if premio <= 0 {
            return Err(Error::MontoInvalido);
        }
        if uri.is_empty() {
            return Err(Error::UriVacia);
        }
        token::Client::new(&env, &token).transfer(&creador, env.current_contract_address(), &premio);
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        poner(&env, id, &Reto { creador: creador.clone(), token, premio, uri: uri.clone(), postulantes: Vec::new(&env), ganador: 0, estado: EstadoReto::Abierto });
        RetoPublicado { creador, id, premio, uri }.publish(&env);
        Ok(id)
    }

    /// El dueño de un agente lo postula a un reto abierto. El creador no postula a sus agentes.
    pub fn postular(env: Env, reto: u32, agente: u32) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        let dueno = AgentesClient::new(&env, &Self::agentes(&env)).dueno(&agente);
        dueno.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        if dueno == r.creador {
            return Err(Error::EsDelCreador);
        }
        if r.postulantes.contains(agente) {
            return Err(Error::YaPostulado);
        }
        r.postulantes.push_back(agente);
        poner(&env, reto, &r);
        Postulado { reto, agente }.publish(&env);
        Ok(())
    }

    /// El creador elige a un agente postulado y el premio va a la cuenta de ese agente.
    /// Regresa la dirección de la cuenta que cobró.
    pub fn elegir_ganador(env: Env, reto: u32, agente: u32) -> Result<Address, Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        if !r.postulantes.contains(agente) {
            return Err(Error::NoPostulado);
        }
        let agentes: Address = Self::agentes(&env);
        let cuentas: Address = env.storage().instance().get(&Clave::Cuentas).unwrap();
        let cuenta = CuentasClient::new(&env, &cuentas).cuenta_de(&agentes, &agente);
        r.estado = EstadoReto::Pagado;
        r.ganador = agente;
        poner(&env, reto, &r);
        token::Client::new(&env, &r.token).transfer(&env.current_contract_address(), &cuenta, &r.premio);
        GanadorElegido { reto, agente, cuenta: cuenta.clone(), premio: r.premio }.publish(&env);
        Ok(cuenta)
    }

    /// El creador cierra un reto sin ganador y recupera el premio.
    pub fn cancelar_reto(env: Env, reto: u32) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        r.estado = EstadoReto::Cancelado;
        poner(&env, reto, &r);
        token::Client::new(&env, &r.token).transfer(&env.current_contract_address(), &r.creador, &r.premio);
        RetoCancelado { reto, premio: r.premio }.publish(&env);
        Ok(())
    }

    /// Cualquiera suma al premio de un reto abierto (patrocinio). Regresa el premio nuevo.
    pub fn ampliar_premio(env: Env, reto: u32, de: Address, extra: i128) -> Result<i128, Error> {
        de.require_auth();
        let mut r = Self::reto(env.clone(), reto)?;
        if extra <= 0 {
            return Err(Error::MontoInvalido);
        }
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        token::Client::new(&env, &r.token).transfer(&de, env.current_contract_address(), &extra);
        r.premio += extra;
        poner(&env, reto, &r);
        PremioAmpliado { reto, extra, premio: r.premio }.publish(&env);
        Ok(r.premio)
    }

    pub fn reto(env: Env, id: u32) -> Result<Reto, Error> {
        env.storage().persistent().get(&Clave::Reto(id)).ok_or(Error::RetoNoExiste)
    }

    fn agentes(env: &Env) -> Address {
        env.storage().instance().get(&Clave::Agentes).unwrap()
    }
}

fn poner(env: &Env, id: u32, r: &Reto) {
    env.storage().persistent().set(&Clave::Reto(id), r);
    env.storage().persistent().extend_ttl(&Clave::Reto(id), TTL_UMBRAL, TTL_EXTENDER);
}

#[cfg(test)]
mod test;
