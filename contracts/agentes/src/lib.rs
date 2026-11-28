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
    String, Symbol, Vec,
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
    Cuenta(u32),
    Pausado(u32),
    Habilidades(u32),
    OperadorAgente(u32),
    Reportes(u32),
    Disponible(u32),
    TarifaAgente(u32),
    Idioma(u32),
    Ejemplos(u32),
    Contacto(u32),
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
    AgentePausado = 11,
    NoEstaPausado = 12,
    YaPausado = 13,
    YaTieneHabilidad = 14,
    SinHabilidad = 15,
    SinOperador = 16,
    MotivoVacio = 17,
    AutoReporte = 18,
    TarifaInvalida = 19,
    SinEjemplo = 20,
    YaTieneEjemplo = 21,
    SinContacto = 22,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperadorAgenteNombrado {
    #[topic]
    pub id: u32,
    pub operador: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperadorAgenteRetirado {
    #[topic]
    pub id: u32,
    pub operador: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteReportado {
    #[topic]
    pub id: u32,
    #[topic]
    pub quien: Address,
    pub motivo: String,
    pub reportes: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HabilidadAgregada {
    #[topic]
    pub id: u32,
    pub habilidad: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HabilidadQuitada {
    #[topic]
    pub id: u32,
    pub habilidad: Symbol,
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

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteQuemado {
    #[topic]
    pub id: u32,
    pub dueno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidacionDelegada {
    #[topic]
    pub trabajo: BytesN<32>,
    pub de: Address,
    pub a: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuentaVinculada {
    #[topic]
    pub id: u32,
    pub cuenta: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentePausadoEv {
    #[topic]
    pub id: u32,
    pub dueno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteReactivado {
    #[topic]
    pub id: u32,
    pub dueno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerfilActualizado {
    #[topic]
    pub id: u32,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisponibilidadFijada {
    #[topic]
    pub id: u32,
    pub disponible: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TarifaAgenteFijada {
    #[topic]
    pub id: u32,
    pub tarifa: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdiomaFijado {
    #[topic]
    pub id: u32,
    pub idioma: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EjemploAgregado {
    #[topic]
    pub id: u32,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EjemploQuitado {
    #[topic]
    pub id: u32,
    pub uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactoFijado {
    #[topic]
    pub id: u32,
    pub contacto: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactoQuitado {
    #[topic]
    pub id: u32,
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
        if Self::pausado(env.clone(), agente) {
            return Err(Error::AgentePausado);
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

    /// El dueño da de baja al agente para siempre. Su identidad desaparece (el id no se
    /// reusa); sus calificaciones quedan como historial y ya no recibe nuevas.
    pub fn quemar_agente(env: Env, id: u32) -> Result<(), Error> {
        let a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        env.storage().persistent().remove(&Clave::Agente(id));
        AgenteQuemado { id, dueno: a.dueno }.publish(&env);
        Ok(())
    }

    /// El validador pedido le pasa una validación pendiente a otro validador (que no sea
    /// el dueño del agente). Firma el validador actual.
    pub fn delegar_validacion(env: Env, trabajo: BytesN<32>, a: Address) -> Result<(), Error> {
        let mut v: Validacion = env
            .storage()
            .persistent()
            .get(&Clave::Validacion(trabajo.clone()))
            .ok_or(Error::ValidacionNoExiste)?;
        v.validador.require_auth();
        if v.respuesta.is_some() {
            return Err(Error::ValidacionYaExiste);
        }
        if Self::agente(env.clone(), v.agente)?.dueno == a || v.validador == a {
            return Err(Error::AutoCalificacion);
        }
        let de = v.validador.clone();
        v.validador = a.clone();
        poner(&env, &Clave::Validacion(trabajo.clone()), &v);
        ValidacionDelegada { trabajo, de, a }.publish(&env);
        Ok(())
    }

    /// El dueño vincula el agente con la cuenta (por ejemplo, su Cuenta de agente) que
    /// cobra por su trabajo. Así los pagos y la reputación apuntan al mismo agente.
    pub fn vincular_cuenta(env: Env, id: u32, cuenta: Address) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        poner(&env, &Clave::Cuenta(id), &cuenta);
        CuentaVinculada { id, cuenta }.publish(&env);
        Ok(())
    }

    /// El dueño pausa al agente: deja de aceptar calificaciones nuevas (por ejemplo,
    /// mientras lo corrige). Su historial sigue público.
    pub fn pausar_agente(env: Env, id: u32) -> Result<(), Error> {
        let a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        if Self::pausado(env.clone(), id) {
            return Err(Error::YaPausado);
        }
        poner(&env, &Clave::Pausado(id), &true);
        AgentePausadoEv { id, dueno: a.dueno }.publish(&env);
        Ok(())
    }

    /// El dueño reactiva un agente pausado.
    pub fn reactivar_agente(env: Env, id: u32) -> Result<(), Error> {
        let a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        if !Self::pausado(env.clone(), id) {
            return Err(Error::NoEstaPausado);
        }
        env.storage().persistent().remove(&Clave::Pausado(id));
        AgenteReactivado { id, dueno: a.dueno }.publish(&env);
        Ok(())
    }

    /// El dueño cambia la URI de la ficha del agente (qué hace, cómo contactarlo).
    pub fn actualizar_perfil(env: Env, id: u32, uri: String) -> Result<(), Error> {
        let mut a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        if uri.is_empty() {
            return Err(Error::UriVacia);
        }
        a.uri = uri.clone();
        poner(&env, &Clave::Agente(id), &a);
        PerfilActualizado { id, uri }.publish(&env);
        Ok(())
    }

    /// El dueño declara una habilidad del agente (por ejemplo `soroban`), para que quien
    /// contrata lo encuentre por habilidad. Declarar no prueba nada: la reputación por
    /// habilidad se gana con trabajo pagado en Reputación.
    pub fn agregar_habilidad(env: Env, id: u32, habilidad: Symbol) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        let mut hs = Self::habilidades(env.clone(), id);
        if hs.contains(&habilidad) {
            return Err(Error::YaTieneHabilidad);
        }
        hs.push_back(habilidad.clone());
        poner(&env, &Clave::Habilidades(id), &hs);
        HabilidadAgregada { id, habilidad }.publish(&env);
        Ok(())
    }

    /// El dueño retira una habilidad que el agente ya no ofrece.
    pub fn quitar_habilidad(env: Env, id: u32, habilidad: Symbol) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        let mut hs = Self::habilidades(env.clone(), id);
        let i = hs.first_index_of(&habilidad).ok_or(Error::SinHabilidad)?;
        hs.remove(i);
        poner(&env, &Clave::Habilidades(id), &hs);
        HabilidadQuitada { id, habilidad }.publish(&env);
        Ok(())
    }

    /// El dueño nombra a un operador que administra el agente en el día a día
    /// (por ejemplo, actualizar su ficha) sin poder transferirlo ni quemarlo.
    pub fn nombrar_operador_agente(env: Env, id: u32, operador: Address) -> Result<(), Error> {
        let a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        if operador == a.dueno {
            return Err(Error::MismoDueno);
        }
        poner(&env, &Clave::OperadorAgente(id), &operador);
        OperadorAgenteNombrado { id, operador }.publish(&env);
        Ok(())
    }

    /// El dueño retira al operador del agente.
    pub fn retirar_operador_agente(env: Env, id: u32) -> Result<(), Error> {
        let a = Self::agente(env.clone(), id)?;
        a.dueno.require_auth();
        let operador = Self::operador_agente(env.clone(), id).ok_or(Error::SinOperador)?;
        env.storage().persistent().remove(&Clave::OperadorAgente(id));
        OperadorAgenteRetirado { id, operador }.publish(&env);
        Ok(())
    }

    /// Cualquier persona reporta a un agente con un motivo (no al propio). El
    /// conteo de reportes es público y sirve para revisar al agente.
    pub fn reportar_agente(env: Env, id: u32, quien: Address, motivo: String) -> Result<u32, Error> {
        quien.require_auth();
        let a = Self::agente(env.clone(), id)?;
        if quien == a.dueno {
            return Err(Error::AutoReporte);
        }
        if motivo.is_empty() {
            return Err(Error::MotivoVacio);
        }
        let reportes = Self::reportes(env.clone(), id) + 1;
        poner(&env, &Clave::Reportes(id), &reportes);
        AgenteReportado { id, quien, motivo, reportes }.publish(&env);
        Ok(reportes)
    }

    /// El dueño marca si el agente acepta trabajo nuevo ahora (disponible para contratar).
    pub fn fijar_disponibilidad(env: Env, id: u32, disponible: bool) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        poner(&env, &Clave::Disponible(id), &disponible);
        DisponibilidadFijada { id, disponible }.publish(&env);
        Ok(())
    }

    /// El dueño publica la tarifa base del agente (en stroops por trabajo).
    pub fn fijar_tarifa_agente(env: Env, id: u32, tarifa: i128) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        if tarifa <= 0 {
            return Err(Error::TarifaInvalida);
        }
        poner(&env, &Clave::TarifaAgente(id), &tarifa);
        TarifaAgenteFijada { id, tarifa }.publish(&env);
        Ok(())
    }

    /// El dueño declara el idioma en que trabaja el agente (por ejemplo `es`).
    pub fn fijar_idioma(env: Env, id: u32, idioma: Symbol) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        poner(&env, &Clave::Idioma(id), &idioma);
        IdiomaFijado { id, idioma }.publish(&env);
        Ok(())
    }

    /// El dueño agrega a su portafolio un enlace a un trabajo de ejemplo.
    pub fn agregar_ejemplo(env: Env, id: u32, uri: String) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        if uri.is_empty() {
            return Err(Error::UriVacia);
        }
        let mut es = Self::ejemplos(env.clone(), id);
        if es.contains(&uri) {
            return Err(Error::YaTieneEjemplo);
        }
        es.push_back(uri.clone());
        poner(&env, &Clave::Ejemplos(id), &es);
        EjemploAgregado { id, uri }.publish(&env);
        Ok(())
    }

    /// El dueño quita un enlace del portafolio.
    pub fn quitar_ejemplo(env: Env, id: u32, uri: String) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        let mut es = Self::ejemplos(env.clone(), id);
        let i = es.first_index_of(&uri).ok_or(Error::SinEjemplo)?;
        es.remove(i);
        poner(&env, &Clave::Ejemplos(id), &es);
        EjemploQuitado { id, uri }.publish(&env);
        Ok(())
    }

    /// El dueño publica cómo contactar al agente (un enlace o un handle).
    pub fn fijar_contacto(env: Env, id: u32, contacto: String) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        if contacto.is_empty() {
            return Err(Error::UriVacia);
        }
        poner(&env, &Clave::Contacto(id), &contacto);
        ContactoFijado { id, contacto }.publish(&env);
        Ok(())
    }

    /// El dueño borra el contacto publicado.
    pub fn quitar_contacto(env: Env, id: u32) -> Result<(), Error> {
        Self::agente(env.clone(), id)?.dueno.require_auth();
        if Self::contacto(env.clone(), id).is_none() {
            return Err(Error::SinContacto);
        }
        env.storage().persistent().remove(&Clave::Contacto(id));
        ContactoQuitado { id }.publish(&env);
        Ok(())
    }

    pub fn disponible(env: Env, id: u32) -> bool {
        env.storage().persistent().get(&Clave::Disponible(id)).unwrap_or(false)
    }

    pub fn tarifa_agente(env: Env, id: u32) -> Option<i128> {
        env.storage().persistent().get(&Clave::TarifaAgente(id))
    }

    pub fn idioma(env: Env, id: u32) -> Option<Symbol> {
        env.storage().persistent().get(&Clave::Idioma(id))
    }

    pub fn ejemplos(env: Env, id: u32) -> Vec<String> {
        env.storage().persistent().get(&Clave::Ejemplos(id)).unwrap_or(Vec::new(&env))
    }

    pub fn contacto(env: Env, id: u32) -> Option<String> {
        env.storage().persistent().get(&Clave::Contacto(id))
    }

    pub fn operador_agente(env: Env, id: u32) -> Option<Address> {
        env.storage().persistent().get(&Clave::OperadorAgente(id))
    }

    pub fn reportes(env: Env, id: u32) -> u32 {
        env.storage().persistent().get(&Clave::Reportes(id)).unwrap_or(0)
    }

    pub fn habilidades(env: Env, id: u32) -> Vec<Symbol> {
        env.storage().persistent().get(&Clave::Habilidades(id)).unwrap_or(Vec::new(&env))
    }

    pub fn cuenta_vinculada(env: Env, id: u32) -> Option<Address> {
        env.storage().persistent().get(&Clave::Cuenta(id))
    }

    pub fn pausado(env: Env, id: u32) -> bool {
        env.storage().persistent().get(&Clave::Pausado(id)).unwrap_or(false)
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
