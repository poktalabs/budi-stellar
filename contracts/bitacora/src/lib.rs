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
    /// Número del hito que el builder fijó arriba de su bitácora.
    Fijado(Address),
    /// Quien firmó `configurar`; solo esa cuenta cambia o quita el acceso.
    Admin,
    /// El agente que el builder autorizó para anotar hitos por él.
    Agente(Address),
    /// La cuenta nueva a la que el builder ofreció pasar su bitácora.
    Oferta(Address),
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
    SinHitoFijado = 7,
    BitacoraVacia = 8,
    SinConfigurar = 9,
    LoteVacio = 10,
    AgenteNoVinculado = 11,
    SinAgente = 12,
    DestinoConHitos = 13,
    SinOferta = 14,
    MismaCuenta = 15,
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

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoFijado {
    #[topic]
    pub builder: Address,
    pub numero: u32,
    pub hito: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoDesfijado {
    #[topic]
    pub builder: Address,
    pub numero: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitacoraVaciada {
    #[topic]
    pub builder: Address,
    pub borrados: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccesoCambiado {
    #[topic]
    pub admin: Address,
    pub matricula: Address,
    pub curso: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccesoQuitado {
    #[topic]
    pub admin: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoMovido {
    #[topic]
    pub builder: Address,
    pub de: u32,
    pub a: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteVinculado {
    #[topic]
    pub builder: Address,
    pub agente: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgenteDesvinculado {
    #[topic]
    pub builder: Address,
    pub agente: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitacoraOfrecida {
    #[topic]
    pub builder: Address,
    pub nuevo: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitacoraTransferida {
    #[topic]
    pub de: Address,
    #[topic]
    pub a: Address,
    pub hitos: u32,
}

#[contract]
pub struct Bitacora;

impl Bitacora {
    fn anotar(env: &Env, builder: &Address, hito: String) -> Result<u32, Error> {
        if let Some(acceso) = Self::acceso(env.clone()) {
            if !MatriculaClient::new(env, &acceso.matricula).inscrito(builder, &acceso.curso) {
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
            .unwrap_or(Vec::new(env));
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
            builder: builder.clone(),
            numero,
            hito,
        }
        .publish(env);
        Ok(numero)
    }

    fn admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&Clave::Admin)
            .ok_or(Error::SinConfigurar)
    }
}

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
        if env.storage().instance().has(&Clave::Acceso) || env.storage().instance().has(&Clave::Admin) {
            return Err(Error::YaConfigurada);
        }
        env.storage().instance().set(&Clave::Admin, &admin);
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
        Self::anotar(&env, &builder, hito)
    }

    /// Registra varios hitos de una vez, en orden, con una sola firma del builder.
    /// Cada hito pasa las mismas reglas que `registrar`; si uno falla, no se guarda
    /// ninguno. Regresa el número del último.
    pub fn registrar_lote(env: Env, builder: Address, hitos: Vec<String>) -> Result<u32, Error> {
        builder.require_auth();
        if hitos.is_empty() {
            return Err(Error::LoteVacio);
        }
        let mut numero = 0;
        for hito in hitos.iter() {
            numero = Self::anotar(&env, &builder, hito)?;
        }
        Ok(numero)
    }

    /// El admin de `configurar` cambia el curso (o el contrato Matrícula) que hay que
    /// pagar para registrar.
    pub fn cambiar_acceso(env: Env, matricula: Address, curso: Symbol) -> Result<(), Error> {
        let admin = Self::admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(
            &Clave::Acceso,
            &Acceso { matricula: matricula.clone(), curso: curso.clone() },
        );
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        AccesoCambiado { admin, matricula, curso }.publish(&env);
        Ok(())
    }

    /// El admin abre la bitácora a todos otra vez (por ejemplo, un taller gratuito).
    pub fn quitar_acceso(env: Env) -> Result<(), Error> {
        let admin = Self::admin(&env)?;
        admin.require_auth();
        if !env.storage().instance().has(&Clave::Acceso) {
            return Err(Error::SinConfigurar);
        }
        env.storage().instance().remove(&Clave::Acceso);
        AccesoQuitado { admin }.publish(&env);
        Ok(())
    }

    /// El builder autoriza a su agente de Budi a anotar hitos por él (el agente escribe
    /// el código; el hito queda en la bitácora del builder). Uno a la vez: vincular otro
    /// reemplaza al anterior.
    pub fn vincular_agente(env: Env, builder: Address, agente: Address) -> Result<(), Error> {
        builder.require_auth();
        if agente == builder {
            return Err(Error::MismaCuenta);
        }
        let clave = Clave::Agente(builder.clone());
        env.storage().persistent().set(&clave, &agente);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        AgenteVinculado { builder, agente }.publish(&env);
        Ok(())
    }

    /// El builder le quita el permiso a su agente. Regresa el agente que tenía.
    pub fn desvincular_agente(env: Env, builder: Address) -> Result<Address, Error> {
        builder.require_auth();
        let clave = Clave::Agente(builder.clone());
        let agente: Address = env.storage().persistent().get(&clave).ok_or(Error::SinAgente)?;
        env.storage().persistent().remove(&clave);
        AgenteDesvinculado { builder, agente: agente.clone() }.publish(&env);
        Ok(agente)
    }

    /// El agente vinculado anota un hito en la bitácora de `builder`. Firma el agente,
    /// no el builder; las reglas (Matrícula, largo, tope) son las mismas.
    pub fn registrar_por_agente(
        env: Env,
        agente: Address,
        builder: Address,
        hito: String,
    ) -> Result<u32, Error> {
        agente.require_auth();
        if Self::agente_de(env.clone(), builder.clone()) != Some(agente) {
            return Err(Error::AgenteNoVinculado);
        }
        Self::anotar(&env, &builder, hito)
    }

    /// El agente que puede anotar por `builder`, si tiene uno.
    pub fn agente_de(env: Env, builder: Address) -> Option<Address> {
        env.storage().persistent().get(&Clave::Agente(builder))
    }

    /// Mueve el hito `de` a la posición `a` (ambos empiezan en 1); el hito fijado sigue
    /// siendo el mismo aunque cambie de número. Solo el propio builder firma.
    pub fn mover_hito(env: Env, builder: Address, de: u32, a: u32) -> Result<(), Error> {
        builder.require_auth();
        let clave = Clave::Hitos(builder.clone());
        let mut hitos = Self::hitos(env.clone(), builder.clone());
        let n = hitos.len();
        if de == 0 || de > n || a == 0 || a > n {
            return Err(Error::HitoInexistente);
        }
        let hito = hitos.get_unchecked(de - 1);
        hitos.remove(de - 1);
        hitos.insert(a - 1, hito);
        env.storage().persistent().set(&clave, &hitos);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        let fijo = Clave::Fijado(builder.clone());
        if let Some(f) = env.storage().persistent().get::<_, u32>(&fijo) {
            let nuevo = if f == de {
                a
            } else if de < a && f > de && f <= a {
                f - 1
            } else if de > a && f >= a && f < de {
                f + 1
            } else {
                f
            };
            env.storage().persistent().set(&fijo, &nuevo);
        }
        HitoMovido { builder, de, a }.publish(&env);
        Ok(())
    }

    /// Primer paso para pasar la bitácora a otra cuenta (wallet nueva o perdida): el
    /// builder la ofrece a `nuevo`. Nada se mueve hasta que `nuevo` la acepte.
    pub fn ofrecer_bitacora(env: Env, builder: Address, nuevo: Address) -> Result<(), Error> {
        builder.require_auth();
        if nuevo == builder {
            return Err(Error::MismaCuenta);
        }
        if Self::contar(env.clone(), builder.clone()) == 0 {
            return Err(Error::BitacoraVacia);
        }
        let clave = Clave::Oferta(builder.clone());
        env.storage().persistent().set(&clave, &nuevo);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        BitacoraOfrecida { builder, nuevo }.publish(&env);
        Ok(())
    }

    /// Segundo paso: `nuevo` firma y recibe los hitos (y el fijado) de `builder`, que
    /// queda vacía. Solo si `nuevo` aún no tiene hitos; el agente no pasa con ella.
    /// Regresa cuántos hitos recibió.
    pub fn aceptar_bitacora(env: Env, nuevo: Address, builder: Address) -> Result<u32, Error> {
        nuevo.require_auth();
        let oferta = Clave::Oferta(builder.clone());
        if env.storage().persistent().get::<_, Address>(&oferta) != Some(nuevo.clone()) {
            return Err(Error::SinOferta);
        }
        if Self::contar(env.clone(), nuevo.clone()) > 0 {
            return Err(Error::DestinoConHitos);
        }
        let hitos = Self::hitos(env.clone(), builder.clone());
        let destino = Clave::Hitos(nuevo.clone());
        env.storage().persistent().set(&destino, &hitos);
        env.storage().persistent().extend_ttl(&destino, TTL_UMBRAL, TTL_EXTENDER);
        if let Some(f) = Self::fijado(env.clone(), builder.clone()) {
            let fijo = Clave::Fijado(nuevo.clone());
            env.storage().persistent().set(&fijo, &f);
            env.storage().persistent().extend_ttl(&fijo, TTL_UMBRAL, TTL_EXTENDER);
        }
        env.storage().persistent().remove(&Clave::Hitos(builder.clone()));
        env.storage().persistent().remove(&Clave::Fijado(builder.clone()));
        env.storage().persistent().remove(&Clave::Agente(builder.clone()));
        env.storage().persistent().remove(&oferta);
        let total = hitos.len();
        BitacoraTransferida { de: builder, a: nuevo, hitos: total }.publish(&env);
        Ok(total)
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
        let fijo = Clave::Fijado(builder.clone());
        if let Some(f) = env.storage().persistent().get::<_, u32>(&fijo) {
            if f == numero {
                env.storage().persistent().remove(&fijo);
            } else if f > numero {
                env.storage().persistent().set(&fijo, &(f - 1));
            }
        }
        HitoBorrado { builder, numero, hito }.publish(&env);
        Ok(())
    }

    /// El builder fija su hito `numero` (empieza en 1) como el más importante, el que su
    /// perfil muestra primero. Fijar otro reemplaza al anterior. Solo el propio builder firma.
    pub fn fijar_hito(env: Env, builder: Address, numero: u32) -> Result<(), Error> {
        builder.require_auth();
        let hitos = Self::hitos(env.clone(), builder.clone());
        if numero == 0 || numero > hitos.len() {
            return Err(Error::HitoInexistente);
        }
        let clave = Clave::Fijado(builder.clone());
        env.storage().persistent().set(&clave, &numero);
        env.storage().persistent().extend_ttl(&clave, TTL_UMBRAL, TTL_EXTENDER);
        HitoFijado { builder, numero, hito: hitos.get_unchecked(numero - 1) }.publish(&env);
        Ok(())
    }

    /// El builder quita el hito fijado; su bitácora vuelve a mostrarse en orden. Solo el
    /// propio builder firma. Regresa el número que estaba fijado.
    pub fn desfijar_hito(env: Env, builder: Address) -> Result<u32, Error> {
        builder.require_auth();
        let clave = Clave::Fijado(builder.clone());
        let numero: u32 = env.storage().persistent().get(&clave).ok_or(Error::SinHitoFijado)?;
        env.storage().persistent().remove(&clave);
        HitoDesfijado { builder, numero }.publish(&env);
        Ok(numero)
    }

    /// El builder borra toda su bitácora de una vez (y el hito fijado, si había) para empezar
    /// de cero. Solo el propio builder firma. Regresa cuántos hitos se borraron.
    pub fn vaciar_bitacora(env: Env, builder: Address) -> Result<u32, Error> {
        builder.require_auth();
        let borrados = Self::contar(env.clone(), builder.clone());
        if borrados == 0 {
            return Err(Error::BitacoraVacia);
        }
        env.storage().persistent().remove(&Clave::Hitos(builder.clone()));
        env.storage().persistent().remove(&Clave::Fijado(builder.clone()));
        BitacoraVaciada { builder, borrados }.publish(&env);
        Ok(borrados)
    }

    /// El número del hito fijado de `builder`, si fijó uno.
    pub fn fijado(env: Env, builder: Address) -> Option<u32> {
        env.storage().persistent().get(&Clave::Fijado(builder))
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
