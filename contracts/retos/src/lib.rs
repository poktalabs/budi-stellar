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
    TienePostulantes = 8,
    NoEstaCancelado = 9,
    MismoCreador = 10,
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

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostulacionRetirada {
    #[topic]
    pub reto: u32,
    pub agente: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremioDividido {
    #[topic]
    pub reto: u32,
    pub agentes: Vec<u32>,
    pub cada_uno: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetoEditado {
    #[topic]
    pub reto: u32,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetoReabierto {
    #[topic]
    pub reto: u32,
    pub premio: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetoTraspasado {
    #[topic]
    pub reto: u32,
    pub de: Address,
    pub a: Address,
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

    /// El dueño de un agente lo baja de un reto abierto.
    pub fn retirar_postulacion(env: Env, reto: u32, agente: u32) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        AgentesClient::new(&env, &Self::agentes(&env)).dueno(&agente).require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        let i = r.postulantes.first_index_of(agente).ok_or(Error::NoPostulado)?;
        r.postulantes.remove(i);
        poner(&env, reto, &r);
        PostulacionRetirada { reto, agente }.publish(&env);
        Ok(())
    }

    /// El creador saca a un agente de su reto (no cumple lo pedido). Firma el creador.
    pub fn rechazar_postulacion(env: Env, reto: u32, agente: u32) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        let i = r.postulantes.first_index_of(agente).ok_or(Error::NoPostulado)?;
        r.postulantes.remove(i);
        poner(&env, reto, &r);
        PostulacionRetirada { reto, agente }.publish(&env);
        Ok(())
    }

    /// El creador reparte el premio en partes iguales entre varios agentes postulados
    /// (cada parte va a la cuenta de su agente; el sobrante de la división, al primero).
    pub fn dividir_premio(env: Env, reto: u32, ganadores: Vec<u32>) -> Result<i128, Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        let n = ganadores.len() as i128;
        if n < 2 {
            return Err(Error::NoPostulado);
        }
        for g in ganadores.iter() {
            if !r.postulantes.contains(g) {
                return Err(Error::NoPostulado);
            }
        }
        let agentes = Self::agentes(&env);
        let cuentas: Address = env.storage().instance().get(&Clave::Cuentas).unwrap();
        let cada_uno = r.premio / n;
        let sobra = r.premio - cada_uno * n;
        r.estado = EstadoReto::Pagado;
        r.ganador = ganadores.get_unchecked(0);
        poner(&env, reto, &r);
        let t = token::Client::new(&env, &r.token);
        for (i, g) in ganadores.iter().enumerate() {
            let cuenta = CuentasClient::new(&env, &cuentas).cuenta_de(&agentes, &g);
            let monto = if i == 0 { cada_uno + sobra } else { cada_uno };
            t.transfer(&env.current_contract_address(), &cuenta, &monto);
        }
        PremioDividido { reto, agentes: ganadores, cada_uno }.publish(&env);
        Ok(cada_uno)
    }

    /// El creador corrige la descripción (uri) de un reto abierto mientras nadie se ha postulado,
    /// para no cambiarle las reglas a quien ya trabaja en él.
    pub fn editar_reto(env: Env, reto: u32, uri: String) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        if !r.postulantes.is_empty() {
            return Err(Error::TienePostulantes);
        }
        if uri.is_empty() {
            return Err(Error::UriVacia);
        }
        r.uri = uri.clone();
        poner(&env, reto, &r);
        RetoEditado { reto, uri }.publish(&env);
        Ok(())
    }

    /// El creador reabre un reto cancelado depositando de nuevo un premio. Las postulaciones
    /// anteriores se borran: cada agente vuelve a postularse.
    pub fn reabrir_reto(env: Env, reto: u32, premio: i128) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        if r.estado != EstadoReto::Cancelado {
            return Err(Error::NoEstaCancelado);
        }
        if premio <= 0 {
            return Err(Error::MontoInvalido);
        }
        token::Client::new(&env, &r.token).transfer(&r.creador, env.current_contract_address(), &premio);
        r.premio = premio;
        r.estado = EstadoReto::Abierto;
        r.postulantes = Vec::new(&env);
        poner(&env, reto, &r);
        RetoReabierto { reto, premio }.publish(&env);
        Ok(())
    }

    /// El creador cede un reto abierto a otra persona (por ejemplo, a la empresa que lo patrocina):
    /// firman los dos, y desde ahí el nuevo creador elige ganador o cancela y recupera el premio.
    pub fn traspasar_reto(env: Env, reto: u32, nuevo: Address) -> Result<(), Error> {
        let mut r = Self::reto(env.clone(), reto)?;
        r.creador.require_auth();
        nuevo.require_auth();
        if r.estado != EstadoReto::Abierto {
            return Err(Error::RetoCerrado);
        }
        if nuevo == r.creador {
            return Err(Error::MismoCreador);
        }
        let de = r.creador.clone();
        r.creador = nuevo.clone();
        poner(&env, reto, &r);
        RetoTraspasado { reto, de, a: nuevo }.publish(&env);
        Ok(())
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
