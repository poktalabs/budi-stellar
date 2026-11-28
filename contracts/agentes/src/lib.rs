//! Agentes: identidad, reputación y validación de agentes de IA, en la línea de
//! ERC-8004 (Trustless Agents), simplificado para Soroban.
//!
//! - **Identidad.** Cada agente es un registro con dueño y una URI a su ficha
//!   (qué hace, cómo contactarlo). El dueño lo puede transferir.
//! - **Reputación.** Un cliente califica a un agente de 0 a 100 con una etiqueta y
//!   una URI a la evidencia. El dueño no se califica a sí mismo. El cliente puede
//!   revocar su calificación y el dueño puede responderla. El promedio es público.
//! - **Validación.** El dueño pide a un validador que revise un trabajo (por su
//!   hash); solo ese validador responde, de 0 a 100.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
    String, Symbol,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Agente {
    pub dueno: Address,
    pub uri: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feedback {
    pub cliente: Address,
    pub puntaje: u32,
    pub etiqueta: Symbol,
    pub uri: String,
    pub revocado: bool,
    pub respuesta: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resumen {
    pub calificaciones: u32,
    pub suma: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validacion {
    pub agente: u32,
    pub validador: Address,
    /// 0 a 100; `None` mientras el validador no responde.
    pub respuesta: Option<u32>,
}

#[contracttype]
enum Clave {
    Siguiente,
    Agente(u32),
    Feedbacks(u32),
    Feedback(u32, u32),
    Resumen(u32),
    Validacion(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AgenteNoExiste = 1,
    UriVacia = 2,
    PuntajeInvalido = 3,
    AutoCalificacion = 4,
    FeedbackNoExiste = 5,
    YaRevocado = 6,
    NoEsTuFeedback = 7,
    ValidacionYaExiste = 8,
    ValidacionNoExiste = 9,
    MismoDueno = 10,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteRegistrado {
    #[topic]
    pub dueno: Address,
    pub id: u32,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteTransferido {
    #[topic]
    pub id: u32,
    pub de: Address,
    pub a: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackDado {
    #[topic]
    pub agente: u32,
    #[topic]
    pub cliente: Address,
    pub indice: u32,
    pub puntaje: u32,
    pub etiqueta: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackRevocado {
    #[topic]
    pub agente: u32,
    pub indice: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackRespondido {
    #[topic]
    pub agente: u32,
    pub indice: u32,
    pub respuesta: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidacionPedida {
    #[topic]
    pub agente: u32,
    #[topic]
    pub validador: Address,
    pub trabajo: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidacionRespondida {
    #[topic]
    pub agente: u32,
    pub trabajo: BytesN<32>,
    pub respuesta: u32,
}

#[contract]
pub struct Agentes;

#[contractimpl]
impl Agentes {
    /// Registra un agente nuevo a nombre de `dueno`, con la URI de su ficha. Regresa su id.
    pub fn registrar_agente(env: Env, dueno: Address, uri: String) -> Result<u32, Error> {
        dueno.require_auth();
        if uri.is_empty() {
            return Err(Error::UriVacia);
        }
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        poner(&env, &Clave::Agente(id), &Agente { dueno: dueno.clone(), uri: uri.clone() });
        AgenteRegistrado { dueno, id, uri }.publish(&env);
        Ok(id)
    }

    /// El dueño le pasa el agente a otra cuenta (con su reputación).
    pub fn transferir_agente(env: Env, id: u32, a: Address) -> Result<(), Error> {
        let mut agente = Self::agente(env.clone(), id)?;
        agente.dueno.require_auth();
        if agente.dueno == a {
            return Err(Error::MismoDueno);
        }
        let de = agente.dueno.clone();
        agente.dueno = a.clone();
        poner(&env, &Clave::Agente(id), &agente);
        AgenteTransferido { id, de, a }.publish(&env);
        Ok(())
    }

    /// Un cliente califica al agente de 0 a 100, con una etiqueta y la URI de la
    /// evidencia. El dueño no se califica. Regresa el índice de la calificación.
    pub fn dar_feedback(
        env: Env,
        cliente: Address,
        agente: u32,
        puntaje: u32,
        etiqueta: Symbol,
        uri: String,
    ) -> Result<u32, Error> {
        cliente.require_auth();
        let a = Self::agente(env.clone(), agente)?;
        if a.dueno == cliente {
            return Err(Error::AutoCalificacion);
        }
        if puntaje > 100 {
            return Err(Error::PuntajeInvalido);
        }
        let indice: u32 = env.storage().persistent().get(&Clave::Feedbacks(agente)).unwrap_or(0);
        poner(&env, &Clave::Feedbacks(agente), &(indice + 1));
        poner(
            &env,
            &Clave::Feedback(agente, indice),
            &Feedback {
                cliente: cliente.clone(),
                puntaje,
                etiqueta: etiqueta.clone(),
                uri,
                revocado: false,
                respuesta: String::from_str(&env, ""),
            },
        );
        let mut r = Self::resumen(env.clone(), agente);
        r.calificaciones += 1;
        r.suma += puntaje;
        poner(&env, &Clave::Resumen(agente), &r);
        FeedbackDado { agente, cliente, indice, puntaje, etiqueta }.publish(&env);
        Ok(indice)
    }

    /// El cliente retira su calificación; deja de contar en el promedio.
    pub fn revocar_feedback(env: Env, cliente: Address, agente: u32, indice: u32) -> Result<(), Error> {
        cliente.require_auth();
        let mut f = Self::feedback(env.clone(), agente, indice)?;
        if f.cliente != cliente {
            return Err(Error::NoEsTuFeedback);
        }
        if f.revocado {
            return Err(Error::YaRevocado);
        }
        f.revocado = true;
        poner(&env, &Clave::Feedback(agente, indice), &f);
        let mut r = Self::resumen(env.clone(), agente);
        r.calificaciones -= 1;
        r.suma -= f.puntaje;
        poner(&env, &Clave::Resumen(agente), &r);
        FeedbackRevocado { agente, indice }.publish(&env);
        Ok(())
    }

    /// El dueño del agente responde una calificación (por ejemplo, con lo que corrigió).
    pub fn responder_feedback(env: Env, agente: u32, indice: u32, respuesta: String) -> Result<(), Error> {
        Self::agente(env.clone(), agente)?.dueno.require_auth();
        if respuesta.is_empty() {
            return Err(Error::UriVacia);
        }
        let mut f = Self::feedback(env.clone(), agente, indice)?;
        f.respuesta = respuesta.clone();
        poner(&env, &Clave::Feedback(agente, indice), &f);
        FeedbackRespondido { agente, indice, respuesta }.publish(&env);
        Ok(())
    }

    /// El dueño pide a `validador` que revise el trabajo con hash `trabajo`.
    pub fn pedir_validacion(env: Env, agente: u32, validador: Address, trabajo: BytesN<32>) -> Result<(), Error> {
        let a = Self::agente(env.clone(), agente)?;
        a.dueno.require_auth();
        if a.dueno == validador {
            return Err(Error::AutoCalificacion);
        }
        if env.storage().persistent().has(&Clave::Validacion(trabajo.clone())) {
            return Err(Error::ValidacionYaExiste);
        }
        poner(&env, &Clave::Validacion(trabajo.clone()), &Validacion { agente, validador: validador.clone(), respuesta: None });
        ValidacionPedida { agente, validador, trabajo }.publish(&env);
        Ok(())
    }

    /// Solo el validador pedido responde, de 0 a 100.
    pub fn responder_validacion(env: Env, trabajo: BytesN<32>, respuesta: u32) -> Result<(), Error> {
        let mut v: Validacion = env
            .storage()
            .persistent()
            .get(&Clave::Validacion(trabajo.clone()))
            .ok_or(Error::ValidacionNoExiste)?;
        v.validador.require_auth();
        if respuesta > 100 {
            return Err(Error::PuntajeInvalido);
        }
        v.respuesta = Some(respuesta);
        poner(&env, &Clave::Validacion(trabajo.clone()), &v);
        ValidacionRespondida { agente: v.agente, trabajo, respuesta }.publish(&env);
        Ok(())
    }

    pub fn agente(env: Env, id: u32) -> Result<Agente, Error> {
        env.storage().persistent().get(&Clave::Agente(id)).ok_or(Error::AgenteNoExiste)
    }

    pub fn dueno(env: Env, id: u32) -> Result<Address, Error> {
        Ok(Self::agente(env, id)?.dueno)
    }

    pub fn feedback(env: Env, agente: u32, indice: u32) -> Result<Feedback, Error> {
        env.storage().persistent().get(&Clave::Feedback(agente, indice)).ok_or(Error::FeedbackNoExiste)
    }

    pub fn resumen(env: Env, agente: u32) -> Resumen {
        env.storage().persistent().get(&Clave::Resumen(agente)).unwrap_or(Resumen { calificaciones: 0, suma: 0 })
    }

    pub fn validacion(env: Env, trabajo: BytesN<32>) -> Option<Validacion> {
        env.storage().persistent().get(&Clave::Validacion(trabajo))
    }
}

fn poner<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, clave: &Clave, valor: &V) {
    env.storage().persistent().set(clave, valor);
    env.storage().persistent().extend_ttl(clave, TTL_UMBRAL, TTL_EXTENDER);
}

#[cfg(test)]
mod test;
