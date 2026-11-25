//! Matrícula: el pago de un curso queda en custodia (escrow) hasta que se entrega.
//!
//! El alumno paga al inscribirse y el dinero se queda en el contrato.
//! Cuando Budi marca el curso como terminado, `liberar` le pasa el pago a Budi.
//! Si el curso no se entrega antes del plazo, el alumno recupera su pago con
//! `reembolsar`. Otros contratos (como Bitácora) preguntan `inscrito` para
//! saber si alguien pagó un curso.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
    Symbol,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Curso {
    /// Precio en la unidad mínima del token (stroops si es XLM).
    pub precio: i128,
    /// Segundos que tiene Budi para entregar el curso antes de que proceda el reembolso.
    pub plazo: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Estado {
    /// Pagada y en custodia.
    EnCustodia = 0,
    /// Curso entregado: el pago ya es de Budi.
    Liberada = 1,
    /// No se entregó a tiempo: el pago regresó al alumno.
    Reembolsada = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inscripcion {
    pub alumno: Address,
    pub curso: Symbol,
    pub monto: i128,
    /// Momento (segundos unix) a partir del cual el alumno puede pedir reembolso.
    pub limite: u64,
    pub estado: Estado,
}

#[contracttype]
enum Clave {
    Admin,
    Token,
    Siguiente,
    Curso(Symbol),
    Inscripcion(u32),
    Activa(Address, Symbol),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CursoNoExiste = 1,
    PrecioInvalido = 2,
    YaInscrito = 3,
    InscripcionNoExiste = 4,
    NoEstaEnCustodia = 5,
    PlazoNoVencido = 6,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inscrito {
    #[topic]
    pub alumno: Address,
    #[topic]
    pub curso: Symbol,
    pub id: u32,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Liberado {
    #[topic]
    pub id: u32,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reembolsado {
    #[topic]
    pub id: u32,
    pub alumno: Address,
    pub monto: i128,
}

#[contract]
pub struct Matricula;

#[contractimpl]
impl Matricula {
    /// `admin` es Budi: crea cursos y libera pagos. `token` es el contrato del
    /// token con el que se cobra (en testnet, el de XLM nativo).
    pub fn __constructor(env: Env, admin: Address, token: Address) {
        env.storage().instance().set(&Clave::Admin, &admin);
        env.storage().instance().set(&Clave::Token, &token);
        env.storage().instance().set(&Clave::Siguiente, &1u32);
    }

    /// Crea o actualiza un curso. Solo el admin.
    pub fn crear_curso(env: Env, curso: Symbol, precio: i128, plazo: u64) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        if precio <= 0 {
            return Err(Error::PrecioInvalido);
        }
        let clave = Clave::Curso(curso);
        env.storage().persistent().set(&clave, &Curso { precio, plazo });
        env.storage()
            .persistent()
            .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        Ok(())
    }

    /// El alumno paga el curso y el pago queda en custodia en este contrato.
    /// Regresa el número de inscripción.
    pub fn inscribir(env: Env, alumno: Address, curso: Symbol) -> Result<u32, Error> {
        alumno.require_auth();
        let datos = Self::curso(env.clone(), curso.clone())?;
        let activa = Clave::Activa(alumno.clone(), curso.clone());
        if let Some(id) = env.storage().persistent().get::<_, u32>(&activa) {
            if Self::inscripcion(env.clone(), id)?.estado != Estado::Reembolsada {
                return Err(Error::YaInscrito);
            }
        }

        Self::token(&env).transfer(&alumno, &env.current_contract_address(), &datos.precio);

        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        let inscripcion = Inscripcion {
            alumno: alumno.clone(),
            curso: curso.clone(),
            monto: datos.precio,
            limite: env.ledger().timestamp() + datos.plazo,
            estado: Estado::EnCustodia,
        };
        Self::guardar(&env, id, &inscripcion);
        env.storage().persistent().set(&activa, &id);
        env.storage()
            .persistent()
            .extend_ttl(&activa, TTL_UMBRAL, TTL_EXTENDER);
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);

        Inscrito {
            alumno,
            curso,
            id,
            monto: datos.precio,
        }
        .publish(&env);
        Ok(id)
    }

    /// Budi entregó el curso: el pago en custodia pasa al admin. Solo el admin.
    pub fn liberar(env: Env, id: u32) -> Result<(), Error> {
        let admin = Self::admin(env.clone());
        admin.require_auth();
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        inscripcion.estado = Estado::Liberada;
        Self::guardar(&env, id, &inscripcion);
        Self::token(&env).transfer(&env.current_contract_address(), &admin, &inscripcion.monto);
        Liberado {
            id,
            monto: inscripcion.monto,
        }
        .publish(&env);
        Ok(())
    }

    /// Venció el plazo y el curso no se entregó: el alumno recupera su pago.
    pub fn reembolsar(env: Env, id: u32) -> Result<(), Error> {
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        inscripcion.alumno.require_auth();
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        if env.ledger().timestamp() < inscripcion.limite {
            return Err(Error::PlazoNoVencido);
        }
        inscripcion.estado = Estado::Reembolsada;
        Self::guardar(&env, id, &inscripcion);
        Self::token(&env).transfer(
            &env.current_contract_address(),
            &inscripcion.alumno,
            &inscripcion.monto,
        );
        Reembolsado {
            id,
            alumno: inscripcion.alumno,
            monto: inscripcion.monto,
        }
        .publish(&env);
        Ok(())
    }

    /// ¿`alumno` pagó `curso` (en custodia o ya entregado)?
    pub fn inscrito(env: Env, alumno: Address, curso: Symbol) -> bool {
        match env
            .storage()
            .persistent()
            .get::<_, u32>(&Clave::Activa(alumno, curso))
        {
            Some(id) => match Self::inscripcion(env, id) {
                Ok(i) => i.estado != Estado::Reembolsada,
                Err(_) => false,
            },
            None => false,
        }
    }

    pub fn inscripcion(env: Env, id: u32) -> Result<Inscripcion, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Inscripcion(id))
            .ok_or(Error::InscripcionNoExiste)
    }

    pub fn curso(env: Env, curso: Symbol) -> Result<Curso, Error> {
        env.storage()
            .persistent()
            .get(&Clave::Curso(curso))
            .ok_or(Error::CursoNoExiste)
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&Clave::Admin).unwrap()
    }

    fn token(env: &Env) -> token::Client<'_> {
        let id: Address = env.storage().instance().get(&Clave::Token).unwrap();
        token::Client::new(env, &id)
    }

    fn guardar(env: &Env, id: u32, inscripcion: &Inscripcion) {
        let clave = Clave::Inscripcion(id);
        env.storage().persistent().set(&clave, inscripcion);
        env.storage()
            .persistent()
            .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
    }
}

#[cfg(test)]
mod test;
