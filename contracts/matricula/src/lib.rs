//! Matrícula: un depósito de compromiso (skin in the game) para tomar un curso.
//!
//! El alumno deposita al inscribirse y el dinero se queda en el contrato.
//! Si termina el curso, Budi lo confirma con `completar` y el depósito regresa
//! completo al alumno. Si no lo termina antes del plazo, Budi se lo queda con
//! `cobrar`. Para probadores alfa, Budi reparte códigos de un solo uso: con
//! `canjear` el alumno entra sin depósito. Otros contratos (como Bitácora) preguntan `inscrito` para saber si
//! alguien está en el curso.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Bytes,
    BytesN, Env, Symbol, Vec,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Curso {
    /// Depósito en la unidad mínima del token (stroops si es XLM).
    pub precio: i128,
    /// Segundos que tiene el alumno para terminar el curso.
    pub plazo: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Estado {
    /// Depósito en custodia: el curso está en marcha.
    EnCustodia = 0,
    /// El alumno terminó el curso: su depósito regresó.
    Devuelta = 1,
    /// Venció el plazo sin terminar: Budi se quedó el depósito.
    Cobrada = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inscripcion {
    pub alumno: Address,
    pub curso: Symbol,
    pub monto: i128,
    /// Momento (segundos unix) en que vence el plazo para terminar el curso.
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
    /// sha256 del código de un solo uso -> curso al que da acceso.
    Codigo(BytesN<32>),
    /// Curso que ya no acepta inscripciones nuevas.
    Cerrado(Symbol),
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
    PlazoVencido = 7,
    CodigoInvalido = 8,
    MismoAlumno = 9,
    CursoCerrado = 10,
    PlazoInvalido = 11,
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
pub struct Abandonado {
    #[topic]
    pub id: u32,
    pub devuelto: i128,
    pub para_budi: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InscripcionTransferida {
    #[topic]
    pub id: u32,
    pub de: Address,
    pub a: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Devuelto {
    #[topic]
    pub id: u32,
    pub alumno: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cobrado {
    #[topic]
    pub id: u32,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlazoExtendido {
    #[topic]
    pub id: u32,
    pub limite: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InscripcionesCerradas {
    #[topic]
    pub curso: Symbol,
}

#[contract]
pub struct Matricula;

#[contractimpl]
impl Matricula {
    /// `admin` es Budi: crea cursos, confirma quién terminó y cobra lo vencido. `token` es el contrato del
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

    /// El alumno deposita y el depósito queda en custodia en este contrato.
    /// Regresa el número de inscripción.
    pub fn inscribir(env: Env, alumno: Address, curso: Symbol) -> Result<u32, Error> {
        alumno.require_auth();
        let datos = Self::curso(env.clone(), curso.clone())?;
        Self::abierto(&env, &curso)?;
        Self::sin_inscripcion_activa(&env, &alumno, &curso)?;
        Self::token(&env).transfer(&alumno, &env.current_contract_address(), &datos.precio);
        Ok(Self::alta(&env, alumno, curso, datos.precio, datos.plazo))
    }

    /// Guarda códigos de acceso gratis de un solo uso para `curso`. Se guarda el sha256
    /// de cada código, nunca el código. Solo el admin.
    pub fn crear_codigos(env: Env, curso: Symbol, hashes: Vec<BytesN<32>>) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        Self::curso(env.clone(), curso.clone())?;
        for h in hashes.iter() {
            let clave = Clave::Codigo(h);
            env.storage().persistent().set(&clave, &curso);
            env.storage()
                .persistent()
                .extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        }
        Ok(())
    }

    /// El alumno entra al curso con un código de un solo uso, sin depósito. El código
    /// se borra al usarse. Regresa el número de inscripción.
    pub fn canjear(env: Env, alumno: Address, codigo: Bytes) -> Result<u32, Error> {
        alumno.require_auth();
        let clave = Clave::Codigo(env.crypto().sha256(&codigo).into());
        let curso: Symbol = env
            .storage()
            .persistent()
            .get(&clave)
            .ok_or(Error::CodigoInvalido)?;
        let datos = Self::curso(env.clone(), curso.clone())?;
        Self::abierto(&env, &curso)?;
        Self::sin_inscripcion_activa(&env, &alumno, &curso)?;
        env.storage().persistent().remove(&clave);
        Ok(Self::alta(&env, alumno, curso, 0, datos.plazo))
    }

    /// El alumno terminó el curso a tiempo: su depósito regresa completo. Firma el admin,
    /// que es quien confirma que terminó.
    pub fn completar(env: Env, id: u32) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        if env.ledger().timestamp() >= inscripcion.limite {
            return Err(Error::PlazoVencido);
        }
        inscripcion.estado = Estado::Devuelta;
        Self::guardar(&env, id, &inscripcion);
        if inscripcion.monto > 0 {
            Self::token(&env).transfer(
                &env.current_contract_address(),
                &inscripcion.alumno,
                &inscripcion.monto,
            );
        }
        Devuelto {
            id,
            alumno: inscripcion.alumno,
            monto: inscripcion.monto,
        }
        .publish(&env);
        Ok(())
    }

    /// Venció el plazo y el alumno no terminó: Budi se queda el depósito. Firma el admin.
    pub fn cobrar(env: Env, id: u32) -> Result<(), Error> {
        let admin = Self::admin(env.clone());
        admin.require_auth();
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        if env.ledger().timestamp() < inscripcion.limite {
            return Err(Error::PlazoNoVencido);
        }
        inscripcion.estado = Estado::Cobrada;
        Self::guardar(&env, id, &inscripcion);
        if inscripcion.monto > 0 {
            Self::token(&env).transfer(&env.current_contract_address(), &admin, &inscripcion.monto);
        }
        Cobrado {
            id,
            monto: inscripcion.monto,
        }
        .publish(&env);
        Ok(())
    }

    /// ¿`alumno` está en `curso` (depósito en custodia o curso terminado)?
/// El alumno deja el curso antes del plazo: recupera la mitad de su depósito y la otra
    /// mitad se queda con Budi. Puede volver a inscribirse después. Firma el alumno.
    pub fn abandonar(env: Env, id: u32) -> Result<i128, Error> {
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        inscripcion.alumno.require_auth();
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        if env.ledger().timestamp() >= inscripcion.limite {
            return Err(Error::PlazoVencido);
        }
        let devuelto = inscripcion.monto / 2;
        let para_budi = inscripcion.monto - devuelto;
        inscripcion.estado = Estado::Cobrada;
        Self::guardar(&env, id, &inscripcion);
        let t = Self::token(&env);
        if devuelto > 0 {
            t.transfer(&env.current_contract_address(), &inscripcion.alumno, &devuelto);
        }
        if para_budi > 0 {
            t.transfer(&env.current_contract_address(), &Self::admin(env.clone()), &para_budi);
        }
        Abandonado { id, devuelto, para_budi }.publish(&env);
        Ok(devuelto)
    }

    /// El alumno le pasa su lugar (y su depósito en custodia) a otra persona que no
    /// esté inscrita en ese curso. Firma el alumno.
    pub fn transferir_inscripcion(env: Env, id: u32, a: Address) -> Result<(), Error> {
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        inscripcion.alumno.require_auth();
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        if inscripcion.alumno == a {
            return Err(Error::MismoAlumno);
        }
        Self::sin_inscripcion_activa(&env, &a, &inscripcion.curso)?;
        let de = inscripcion.alumno.clone();
        env.storage().persistent().remove(&Clave::Activa(de.clone(), inscripcion.curso.clone()));
        let activa = Clave::Activa(a.clone(), inscripcion.curso.clone());
        env.storage().persistent().set(&activa, &id);
        env.storage().persistent().extend_ttl(&activa, TTL_UMBRAL, TTL_EXTENDER);
        inscripcion.alumno = a.clone();
        Self::guardar(&env, id, &inscripcion);
        InscripcionTransferida { id, de, a }.publish(&env);
        Ok(())
    }


    /// Budi le da más tiempo a un alumno: suma `segundos` al límite de una inscripción en
    /// custodia (por ejemplo, por una incapacidad). Solo el admin. Regresa el nuevo límite.
    pub fn extender_plazo(env: Env, id: u32, segundos: u64) -> Result<u64, Error> {
        Self::admin(env.clone()).require_auth();
        if segundos == 0 {
            return Err(Error::PlazoInvalido);
        }
        let mut inscripcion = Self::inscripcion(env.clone(), id)?;
        if inscripcion.estado != Estado::EnCustodia {
            return Err(Error::NoEstaEnCustodia);
        }
        inscripcion.limite += segundos;
        Self::guardar(&env, id, &inscripcion);
        PlazoExtendido { id, limite: inscripcion.limite }.publish(&env);
        Ok(inscripcion.limite)
    }

    /// Budi cierra las inscripciones de `curso`: nadie nuevo entra (ni con depósito ni con
    /// código); quien ya está inscrito sigue igual. Solo el admin.
    pub fn cerrar_inscripciones(env: Env, curso: Symbol) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        Self::curso(env.clone(), curso.clone())?;
        let clave = Clave::Cerrado(curso.clone());
        env.storage().persistent().set(&clave, &true);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        InscripcionesCerradas { curso }.publish(&env);
        Ok(())
    }

    fn abierto(env: &Env, curso: &Symbol) -> Result<(), Error> {
        if env.storage().persistent().has(&Clave::Cerrado(curso.clone())) {
            return Err(Error::CursoCerrado);
        }
        Ok(())
    }

        pub fn inscrito(env: Env, alumno: Address, curso: Symbol) -> bool {
        match env
            .storage()
            .persistent()
            .get::<_, u32>(&Clave::Activa(alumno, curso))
        {
            Some(id) => match Self::inscripcion(env, id) {
                Ok(i) => i.estado != Estado::Cobrada,
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

    fn sin_inscripcion_activa(env: &Env, alumno: &Address, curso: &Symbol) -> Result<(), Error> {
        let activa = Clave::Activa(alumno.clone(), curso.clone());
        if let Some(id) = env.storage().persistent().get::<_, u32>(&activa) {
            if Self::inscripcion(env.clone(), id)?.estado != Estado::Cobrada {
                return Err(Error::YaInscrito);
            }
        }
        Ok(())
    }

    fn alta(env: &Env, alumno: Address, curso: Symbol, monto: i128, plazo: u64) -> u32 {
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        let inscripcion = Inscripcion {
            alumno: alumno.clone(),
            curso: curso.clone(),
            monto,
            limite: env.ledger().timestamp() + plazo,
            estado: Estado::EnCustodia,
        };
        Self::guardar(env, id, &inscripcion);
        let activa = Clave::Activa(alumno.clone(), curso.clone());
        env.storage().persistent().set(&activa, &id);
        env.storage()
            .persistent()
            .extend_ttl(&activa, TTL_UMBRAL, TTL_EXTENDER);
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        Inscrito {
            alumno,
            curso,
            id,
            monto,
        }
        .publish(env);
        id
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
