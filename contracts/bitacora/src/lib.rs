//! Bitácora: cada builder registra sus hitos onchain.
//!
//! Es el contrato de ejemplo que Budi abre en cada sesión nueva: corto,
//! con autorización, almacenamiento y eventos, para que el entorno pruebe
//! todo el ciclo (compilar, probar, desplegar) desde el primer minuto.
//!
//! Sin configurar, cualquiera registra. Con `configurar`, solo registra quien
//! pagó el curso en el contrato Matrícula (escrow del pago del curso).
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    Env, String, Symbol, Vec,
};

/// Un hito no puede pasar de este largo (en bytes).
pub const MAX_HITO: u32 = 140;
/// Un builder guarda hasta este número de hitos.
pub const MAX_HITOS: u32 = 100;

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
enum Clave {
    Hitos(Address),
    Acceso,
}

/// Qué curso de qué contrato Matrícula hay que haber pagado para registrar.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Acceso {
    pub matricula: Address,
    pub curso: Symbol,
}

/// Lo único que Bitácora le pregunta a Matrícula.
#[contractclient(name = "MatriculaClient")]
pub trait MatriculaApi {
    fn inscrito(env: Env, alumno: Address, curso: Symbol) -> bool;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    HitoVacio = 1,
    HitoMuyLargo = 2,
    BitacoraLlena = 3,
    NoInscrito = 4,
    YaConfigurada = 5,
    HitoInexistente = 6,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoRegistrado {
    #[topic]
    pub builder: Address,
    pub numero: u32,
    pub hito: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoCorregido {
    #[topic]
    pub builder: Address,
    pub numero: u32,
    pub antes: String,
    pub ahora: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoBorrado {
    #[topic]
    pub builder: Address,
    pub numero: u32,
    pub hito: String,
}

#[contract]
pub struct Bitacora;

#[contractimpl]
impl Bitacora {
    /// Pide que quien registre haya pagado `curso` en `matricula`. Se hace una
    /// sola vez, firmada por `admin`, justo después de desplegar.
    pub fn configurar(
        env: Env,
        admin: Address,
        matricula: Address,
        curso: Symbol,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&Clave::Acceso) {
            return Err(Error::YaConfigurada);
        }
        env.storage()
            .instance()
            .set(&Clave::Acceso, &Acceso { matricula, curso });
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        Ok(())
    }

    /// El curso que hay que pagar para registrar, si hay uno.
    pub fn acceso(env: Env) -> Option<Acceso> {
        env.storage().instance().get(&Clave::Acceso)
    }

    /// Registra un hito para `builder`. Solo el propio builder puede firmar.
    /// Regresa el número del hito (empieza en 1).
    pub fn registrar(env: Env, builder: Address, hito: String) -> Result<u32, Error> {
        builder.require_auth();
        if let Some(acceso) = Self::acceso(env.clone()) {
            if !MatriculaClient::new(&env, &acceso.matricula).inscrito(&builder, &acceso.curso) {
                return Err(Error::NoInscrito);
            }
        }
        if hito.is_empty() {
            return Err(Error::HitoVacio);
        }
        if hito.len() > MAX_HITO {
            return Err(Error::HitoMuyLargo);
        }
        let clave = Clave::Hitos(builder.clone());
        let mut hitos: Vec<String> = env
            .storage()
            .persistent()
            .get(&clave)
            .unwrap_or(Vec::new(&env));
        if hitos.len() >= MAX_HITOS {
            return Err(Error::BitacoraLlena);
        }
        hitos.push_back(hito.clone());
        let numero = hitos.len();
        env.storage().persistent().set(&clave, &hitos);
        env.storage()
            .persistent()
            .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        HitoRegistrado {
            builder,
            numero,
            hito,
        }
        .publish(&env);
        Ok(numero)
    }

    /// Corrige el hito `numero` (empieza en 1) de `builder`. Solo el propio builder
    /// firma; el texto nuevo pasa las mismas reglas que al registrar, y el evento
    /// guarda el texto de antes para que la corrección quede a la vista.
    pub fn corregir(env: Env, builder: Address, numero: u32, hito: String) -> Result<(), Error> {
        builder.require_auth();
        if hito.is_empty() {
            return Err(Error::HitoVacio);
        }
        if hito.len() > MAX_HITO {
            return Err(Error::HitoMuyLargo);
        }
        let clave = Clave::Hitos(builder.clone());
        let mut hitos: Vec<String> = env
            .storage()
            .persistent()
            .get(&clave)
            .unwrap_or(Vec::new(&env));
        if numero == 0 || numero > hitos.len() {
            return Err(Error::HitoInexistente);
        }
        let antes = hitos.get_unchecked(numero - 1);
        hitos.set(numero - 1, hito.clone());
        env.storage().persistent().set(&clave, &hitos);
        env.storage()
            .persistent()
            .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        HitoCorregido {
            builder,
            numero,
            antes,
            ahora: hito,
        }
        .publish(&env);
        Ok(())
    }

    /// Borra el hito `numero` de `builder`; los siguientes recorren su número. Solo el
    /// propio builder firma, y el evento guarda el texto borrado.
    pub fn borrar(env: Env, builder: Address, numero: u32) -> Result<(), Error> {
        builder.require_auth();
        let clave = Clave::Hitos(builder.clone());
        let mut hitos: Vec<String> = env.storage().persistent().get(&clave).unwrap_or(Vec::new(&env));
        if numero == 0 || numero > hitos.len() {
            return Err(Error::HitoInexistente);
        }
        let hito = hitos.get_unchecked(numero - 1);
        hitos.remove(numero - 1);
        env.storage().persistent().set(&clave, &hitos);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        HitoBorrado { builder, numero, hito }.publish(&env);
        Ok(())
    }

    /// Todos los hitos de `builder`, del primero al último.
    pub fn hitos(env: Env, builder: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&Clave::Hitos(builder))
            .unwrap_or(Vec::new(&env))
    }

    /// Cuántos hitos lleva `builder`.
    pub fn contar(env: Env, builder: Address) -> u32 {
        Self::hitos(env, builder).len()
    }
}

#[cfg(test)]
mod test;
