//! Reputación: la reputación por habilidad de un equipo humano + agente Budi.
//!
//! Reglas (las aceptadas para empezar, en `docs` del proyecto Budi):
//!
//! - **La insignia es la licencia.** Para ganar puntos en una habilidad, el humano
//!   del equipo necesita la insignia que la abre (contrato Insignias). La insignia
//!   nunca suma puntos.
//! - **Solo cuenta el trabajo pagado y entregado.** Un cliente contrata al equipo y
//!   su pago queda en custodia; el equipo entrega con una prueba; cuando el cliente
//!   acepta, el pago pasa al humano y el equipo gana puntos en esa habilidad.
//! - **Piso y tope.** Un trabajo por debajo del piso de la habilidad se paga pero no
//!   suma. Cada trabajo suma como máximo `max_por_trabajo`. Las evaluaciones de Budi
//!   suman como máximo `tope_evaluacion` en total, para que no carguen el puntaje.
//! - **El silencio no traba el pago.** El cliente tiene un plazo de revisión después
//!   de la entrega. Si no acepta ni responde en ese plazo, el humano cobra y el equipo
//!   suma como si hubiera aceptado.
//! - **Vigencia.** Los puntos vencen. Sin trabajo reciente, el puntaje baja solo.
//! - **Anular es aparte de vencer.** Budi puede anular los puntos de un trabajo
//!   (fraude o daño) con un motivo público.
//! - **El equipo es la pareja.** Un humano forma un equipo con un agente; la
//!   reputación es del equipo, no se transfiere, y un agente solo está en un equipo.
//!   Si el humano retira al agente, sus registros se quedan con el equipo.
//!
//! Primera versión, en testnet. La fórmula es pública aquí; las reglas pueden cambiar.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, String, Symbol, Vec,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días
/// Registros vigentes que guarda un equipo por habilidad.
pub const MAX_REGISTROS: u32 = 200;

/// Las reglas de una habilidad, que fija Budi.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Habilidad {
    /// La ruta de Insignias que abre esta habilidad.
    pub insignia: Symbol,
    /// Pago mínimo (unidad mínima del token) para que un trabajo sume puntos.
    pub piso: i128,
    /// Puntos máximos que suma un trabajo.
    pub max_por_trabajo: u32,
    /// Segundos que duran los puntos.
    pub vigencia: u64,
    /// Puntos máximos que pueden venir de evaluaciones de Budi, en total.
    pub tope_evaluacion: u32,
    /// Segundos que tiene el cliente para revisar una entrega. Después, el humano cobra.
    pub revision: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Equipo {
    pub humano: Address,
    pub agente: Address,
    pub activo: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Estado {
    /// El pago del cliente está en custodia y el equipo está trabajando.
    EnCustodia = 0,
    /// El equipo entregó; falta que el cliente acepte.
    Entregado = 1,
    /// El cliente aceptó, o no respondió en el plazo de revisión: el humano cobró y
    /// el equipo sumó puntos.
    Pagado = 2,
    /// Venció el plazo sin entrega: el cliente recuperó su pago.
    Reembolsado = 3,
    /// Budi anuló los puntos de este trabajo.
    Anulado = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trabajo {
    pub cliente: Address,
    pub equipo: u32,
    pub habilidad: Symbol,
    pub monto: i128,
    /// Momento (segundos unix) en que vence el plazo de entrega.
    pub limite: u64,
    /// Enlace o hash de lo entregado, para que Budi lo pueda revisar.
    pub prueba: String,
    /// Momento (segundos unix) en que vence la revisión del cliente; 0 antes de entregar.
    pub revisar_hasta: u64,
    pub puntos: u32,
    pub estado: Estado,
}

/// Una entrada del puntaje: un trabajo pagado (`trabajo` > 0) o una evaluación (`trabajo` = 0).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registro {
    pub trabajo: u32,
    pub puntos: u32,
    pub vence: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puntaje {
    /// Puntos vigentes: trabajos más evaluaciones (con su tope).
    pub puntos: u32,
    /// Trabajos pagados con puntos vigentes.
    pub trabajos: u32,
    /// Puntos vigentes que vienen de evaluaciones, ya con el tope.
    pub por_evaluacion: u32,
}

#[contracttype]
enum Clave {
    Admin,
    Token,
    Insignias,
    SiguienteEquipo,
    SiguienteTrabajo,
    Habilidad(Symbol),
    Equipo(u32),
    Agente(Address),
    Trabajo(u32),
    Registros(u32, Symbol),
}

/// Lo único que Reputación le pregunta a Insignias.
#[contractclient(name = "InsigniasClient")]
pub trait InsigniasApi {
    fn tiene(env: Env, titular: Address, ruta: Symbol) -> bool;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    HabilidadNoExiste = 1,
    ReglasInvalidas = 2,
    MontoInvalido = 3,
    EquipoNoExiste = 4,
    AgenteYaEnEquipo = 5,
    EquipoRetirado = 6,
    SinInsignia = 7,
    TrabajoNoExiste = 8,
    EstadoInvalido = 9,
    PlazoVencido = 10,
    PlazoNoVencido = 11,
    PuntosInvalidos = 12,
    MismaCuenta = 13,
    PruebaVacia = 14,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipoFormado {
    #[topic]
    pub humano: Address,
    pub id: u32,
    pub agente: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteRetirado {
    #[topic]
    pub equipo: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contratado {
    #[topic]
    pub equipo: u32,
    #[topic]
    pub habilidad: Symbol,
    pub id: u32,
    pub cliente: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entregado {
    #[topic]
    pub id: u32,
    pub prueba: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuntosGanados {
    #[topic]
    pub equipo: u32,
    #[topic]
    pub habilidad: Symbol,
    /// 0 si vienen de una evaluación de Budi.
    pub trabajo: u32,
    pub puntos: u32,
    pub vence: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reembolsado {
    #[topic]
    pub id: u32,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CobradoSinRespuesta {
    #[topic]
    pub id: u32,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuntosAnulados {
    #[topic]
    pub equipo: u32,
    #[topic]
    pub habilidad: Symbol,
    pub trabajo: u32,
    pub puntos: u32,
    pub motivo: String,
}

#[contract]
pub struct Reputacion;

#[contractimpl]
impl Reputacion {
    /// `admin` es Budi: forma equipos, fija reglas, evalúa y anula. `token` es el token
    /// con el que pagan los clientes. `insignias` es el contrato de insignias (licencias).
    pub fn __constructor(env: Env, admin: Address, token: Address, insignias: Address) {
        let s = env.storage().instance();
        s.set(&Clave::Admin, &admin);
        s.set(&Clave::Token, &token);
        s.set(&Clave::Insignias, &insignias);
        s.set(&Clave::SiguienteEquipo, &1u32);
        s.set(&Clave::SiguienteTrabajo, &1u32);
    }

    /// Crea o cambia las reglas de una habilidad. Solo el admin.
    pub fn definir_habilidad(env: Env, habilidad: Symbol, reglas: Habilidad) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        if reglas.piso <= 0
            || reglas.max_por_trabajo == 0
            || reglas.vigencia == 0
            || reglas.revision == 0
        {
            return Err(Error::ReglasInvalidas);
        }
        Self::poner(&env, &Clave::Habilidad(habilidad), &reglas);
        Ok(())
    }

    /// Forma un equipo humano + agente. Firman Budi (que lo emite), el humano y el agente.
    /// Un agente solo puede estar en un equipo, para siempre. Regresa el número de equipo.
    pub fn formar_equipo(env: Env, humano: Address, agente: Address) -> Result<u32, Error> {
        if humano == agente {
            return Err(Error::MismaCuenta);
        }
        Self::admin(env.clone()).require_auth();
        humano.require_auth();
        agente.require_auth();
        if env
            .storage()
            .persistent()
            .has(&Clave::Agente(agente.clone()))
        {
            return Err(Error::AgenteYaEnEquipo);
        }
        let id = Self::siguiente(&env, Clave::SiguienteEquipo);
        let equipo = Equipo {
            humano: humano.clone(),
            agente: agente.clone(),
            activo: true,
        };
        Self::poner(&env, &Clave::Equipo(id), &equipo);
        Self::poner(&env, &Clave::Agente(agente.clone()), &id);
        EquipoFormado { humano, id, agente }.publish(&env);
        Ok(id)
    }

    /// El humano retira a su agente. El equipo ya no toma trabajos nuevos, pero sus
    /// registros se quedan, y el agente no puede entrar a otro equipo.
    pub fn retirar(env: Env, equipo: u32) -> Result<(), Error> {
        let mut e = Self::equipo(env.clone(), equipo)?;
        e.humano.require_auth();
        if !e.activo {
            return Err(Error::EquipoRetirado);
        }
        e.activo = false;
        Self::poner(&env, &Clave::Equipo(equipo), &e);
        AgenteRetirado { equipo }.publish(&env);
        Ok(())
    }

    /// Un cliente contrata a un equipo para un trabajo en una habilidad. Su pago queda
    /// en custodia en este contrato. El humano del equipo necesita la insignia de la
    /// habilidad. Regresa el número de trabajo.
    pub fn contratar(
        env: Env,
        cliente: Address,
        equipo: u32,
        habilidad: Symbol,
        monto: i128,
        plazo: u64,
    ) -> Result<u32, Error> {
        cliente.require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        let reglas = Self::habilidad(env.clone(), habilidad.clone())?;
        let e = Self::equipo(env.clone(), equipo)?;
        if !e.activo {
            return Err(Error::EquipoRetirado);
        }
        if cliente == e.humano || cliente == e.agente {
            return Err(Error::MismaCuenta);
        }
        Self::con_insignia(&env, &e.humano, &reglas)?;
        Self::token(&env).transfer(&cliente, env.current_contract_address(), &monto);
        let id = Self::siguiente(&env, Clave::SiguienteTrabajo);
        let trabajo = Trabajo {
            cliente: cliente.clone(),
            equipo,
            habilidad: habilidad.clone(),
            monto,
            limite: env.ledger().timestamp() + plazo,
            prueba: String::from_str(&env, ""),
            revisar_hasta: 0,
            puntos: 0,
            estado: Estado::EnCustodia,
        };
        Self::poner(&env, &Clave::Trabajo(id), &trabajo);
        Contratado {
            equipo,
            habilidad,
            id,
            cliente,
            monto,
        }
        .publish(&env);
        Ok(id)
    }

    /// El humano del equipo entrega el trabajo, antes del plazo, con una prueba
    /// (un enlace o un hash) que Budi pueda revisar.
    pub fn entregar(env: Env, trabajo: u32, prueba: String) -> Result<(), Error> {
        let mut t = Self::trabajo(env.clone(), trabajo)?;
        Self::equipo(env.clone(), t.equipo)?.humano.require_auth();
        if t.estado != Estado::EnCustodia {
            return Err(Error::EstadoInvalido);
        }
        if env.ledger().timestamp() >= t.limite {
            return Err(Error::PlazoVencido);
        }
        if prueba.is_empty() {
            return Err(Error::PruebaVacia);
        }
        let reglas = Self::habilidad(env.clone(), t.habilidad.clone())?;
        t.prueba = prueba.clone();
        t.revisar_hasta = env.ledger().timestamp() + reglas.revision;
        t.estado = Estado::Entregado;
        Self::poner(&env, &Clave::Trabajo(trabajo), &t);
        Entregado {
            id: trabajo,
            prueba,
        }
        .publish(&env);
        Ok(())
    }

    /// El cliente acepta lo entregado: el pago pasa al humano del equipo y el equipo
    /// suma puntos en la habilidad si el pago llega al piso y el humano todavía tiene
    /// la insignia. Regresa los puntos.
    pub fn aceptar(env: Env, trabajo: u32) -> Result<u32, Error> {
        let t = Self::trabajo(env.clone(), trabajo)?;
        t.cliente.require_auth();
        if t.estado != Estado::Entregado {
            return Err(Error::EstadoInvalido);
        }
        Self::pagar(&env, trabajo, t)
    }

    /// El cliente no aceptó ni respondió en el plazo de revisión: el humano cobra el
    /// pago en custodia y el equipo suma como si el cliente hubiera aceptado. Firma el
    /// humano. Regresa los puntos.
    pub fn cobrar_sin_respuesta(env: Env, trabajo: u32) -> Result<u32, Error> {
        let t = Self::trabajo(env.clone(), trabajo)?;
        Self::equipo(env.clone(), t.equipo)?.humano.require_auth();
        if t.estado != Estado::Entregado {
            return Err(Error::EstadoInvalido);
        }
        if env.ledger().timestamp() < t.revisar_hasta {
            return Err(Error::PlazoNoVencido);
        }
        let monto = t.monto;
        let puntos = Self::pagar(&env, trabajo, t)?;
        CobradoSinRespuesta { id: trabajo, monto }.publish(&env);
        Ok(puntos)
    }

    /// Paga al humano un trabajo entregado y suma sus puntos (si llega al piso y el
    /// humano todavía tiene la insignia).
    fn pagar(env: &Env, trabajo: u32, mut t: Trabajo) -> Result<u32, Error> {
        let env = env.clone();
        let e = Self::equipo(env.clone(), t.equipo)?;
        let reglas = Self::habilidad(env.clone(), t.habilidad.clone())?;
        let por_monto = t.monto / reglas.piso;
        // Si Budi le revocó la insignia en medio del trabajo, se paga pero no suma.
        let puntos = if Self::con_insignia(&env, &e.humano, &reglas).is_err() {
            0
        } else if por_monto >= reglas.max_por_trabajo as i128 {
            reglas.max_por_trabajo
        } else {
            por_monto as u32
        };
        t.puntos = puntos;
        t.estado = Estado::Pagado;
        Self::poner(&env, &Clave::Trabajo(trabajo), &t);
        Self::token(&env).transfer(&env.current_contract_address(), &e.humano, &t.monto);
        if puntos > 0 {
            Self::sumar(&env, t.equipo, &t.habilidad, &reglas, trabajo, puntos);
        }
        Ok(puntos)
    }

    /// Venció el plazo y el equipo no entregó: el cliente recupera su pago. Firma el cliente.
    pub fn reembolsar(env: Env, trabajo: u32) -> Result<(), Error> {
        let mut t = Self::trabajo(env.clone(), trabajo)?;
        t.cliente.require_auth();
        if t.estado != Estado::EnCustodia {
            return Err(Error::EstadoInvalido);
        }
        if env.ledger().timestamp() < t.limite {
            return Err(Error::PlazoNoVencido);
        }
        t.estado = Estado::Reembolsado;
        Self::poner(&env, &Clave::Trabajo(trabajo), &t);
        Self::token(&env).transfer(&env.current_contract_address(), &t.cliente, &t.monto);
        Reembolsado {
            id: trabajo,
            monto: t.monto,
        }
        .publish(&env);
        Ok(())
    }

    /// Budi suma puntos por una evaluación calificada (la otra forma de renovar).
    /// Cada evaluación da como máximo `tope_evaluacion` puntos, y en el puntaje las
    /// evaluaciones nunca pasan de ese tope en total. Solo el admin.
    pub fn evaluar(env: Env, equipo: u32, habilidad: Symbol, puntos: u32) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        let reglas = Self::habilidad(env.clone(), habilidad.clone())?;
        if puntos == 0 || puntos > reglas.tope_evaluacion {
            return Err(Error::PuntosInvalidos);
        }
        let e = Self::equipo(env.clone(), equipo)?;
        if !e.activo {
            return Err(Error::EquipoRetirado);
        }
        Self::con_insignia(&env, &e.humano, &reglas)?;
        Self::sumar(&env, equipo, &habilidad, &reglas, 0, puntos);
        Ok(())
    }

    /// Budi anula los puntos de un trabajo pagado (fraude o daño), con un motivo
    /// público. Es aparte del vencimiento. El pago al humano no se revierte aquí.
    pub fn anular(env: Env, trabajo: u32, motivo: String) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        let mut t = Self::trabajo(env.clone(), trabajo)?;
        if t.estado != Estado::Pagado {
            return Err(Error::EstadoInvalido);
        }
        let clave = Clave::Registros(t.equipo, t.habilidad.clone());
        let registros = Self::registros(&env, t.equipo, &t.habilidad);
        let mut quedan = Vec::new(&env);
        for r in registros.iter() {
            if r.trabajo != trabajo {
                quedan.push_back(r);
            }
        }
        Self::poner(&env, &clave, &quedan);
        let puntos = t.puntos;
        t.puntos = 0;
        t.estado = Estado::Anulado;
        Self::poner(&env, &Clave::Trabajo(trabajo), &t);
        PuntosAnulados {
            equipo: t.equipo,
            habilidad: t.habilidad,
            trabajo,
            puntos,
            motivo,
        }
        .publish(&env);
        Ok(())
    }

    /// El puntaje vigente de un equipo en una habilidad.
    pub fn puntaje(env: Env, equipo: u32, habilidad: Symbol) -> Puntaje {
        let ahora = env.ledger().timestamp();
        let tope = Self::habilidad(env.clone(), habilidad.clone())
            .map(|r| r.tope_evaluacion)
            .unwrap_or(0);
        let (mut por_trabajo, mut trabajos, mut por_evaluacion) = (0u32, 0u32, 0u32);
        for r in Self::registros(&env, equipo, &habilidad).iter() {
            if r.vence <= ahora {
                continue;
            }
            if r.trabajo == 0 {
                por_evaluacion = por_evaluacion.saturating_add(r.puntos);
            } else {
                por_trabajo = por_trabajo.saturating_add(r.puntos);
                trabajos += 1;
            }
        }
        let por_evaluacion = por_evaluacion.min(tope);
        Puntaje {
            puntos: por_trabajo.saturating_add(por_evaluacion),
            trabajos,
            por_evaluacion,
        }
    }

    pub fn equipo(env: Env, id: u32) -> Result<Equipo, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Equipo(id))
            .ok_or(Error::EquipoNoExiste)
    }

    /// El equipo de un agente, si tiene.
    pub fn equipo_de(env: Env, agente: Address) -> Option<u32> {
        env.storage().persistent().get(&Clave::Agente(agente))
    }

    pub fn trabajo(env: Env, id: u32) -> Result<Trabajo, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Trabajo(id))
            .ok_or(Error::TrabajoNoExiste)
    }

    pub fn habilidad(env: Env, habilidad: Symbol) -> Result<Habilidad, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Habilidad(habilidad))
            .ok_or(Error::HabilidadNoExiste)
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&Clave::Admin).unwrap()
    }

    fn token(env: &Env) -> token::Client<'_> {
        let id: Address = env.storage().instance().get(&Clave::Token).unwrap();
        token::Client::new(env, &id)
    }

    fn con_insignia(env: &Env, humano: &Address, reglas: &Habilidad) -> Result<(), Error> {
        let id: Address = env.storage().instance().get(&Clave::Insignias).unwrap();
        if InsigniasClient::new(env, &id).tiene(humano, &reglas.insignia) {
            Ok(())
        } else {
            Err(Error::SinInsignia)
        }
    }

    fn registros(env: &Env, equipo: u32, habilidad: &Symbol) -> Vec<Registro> {
        env.storage()
            .persistent()
            .get(&Clave::Registros(equipo, habilidad.clone()))
            .unwrap_or(Vec::new(env))
    }

    /// Agrega un registro y de paso tira los vencidos, para que la lista no crezca sin fin.
    fn sumar(
        env: &Env,
        equipo: u32,
        habilidad: &Symbol,
        reglas: &Habilidad,
        trabajo: u32,
        puntos: u32,
    ) {
        let ahora = env.ledger().timestamp();
        let mut vigentes = Vec::new(env);
        for r in Self::registros(env, equipo, habilidad).iter() {
            if r.vence > ahora {
                vigentes.push_back(r);
            }
        }
        // Si se llena, se va el más viejo.
        if vigentes.len() >= MAX_REGISTROS {
            vigentes.pop_front();
        }
        let vence = ahora + reglas.vigencia;
        vigentes.push_back(Registro {
            trabajo,
            puntos,
            vence,
        });
        Self::poner(env, &Clave::Registros(equipo, habilidad.clone()), &vigentes);
        PuntosGanados {
            equipo,
            habilidad: habilidad.clone(),
            trabajo,
            puntos,
            vence,
        }
        .publish(env);
    }

    fn siguiente(env: &Env, clave: Clave) -> u32 {
        let id: u32 = env.storage().instance().get(&clave).unwrap_or(1);
        env.storage().instance().set(&clave, &(id + 1));
        env.storage()
            .instance()
            .extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        id
    }

    fn poner<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, clave: &Clave, valor: &V) {
        env.storage().persistent().set(clave, valor);
        env.storage()
            .persistent()
            .extend_ttl(clave, TTL_UMBRAL, TTL_EXTENDER);
    }
}

#[cfg(test)]
mod test;
