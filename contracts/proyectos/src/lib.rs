//! Proyectos: el proyecto final de quien aprende en Budi, con revisión entre pares.
//!
//! Quien aprende registra su proyecto con el enlace al repo y sus hitos, pide
//! revisores entre sus pares, los pares aprueban o piden cambios, el mentor
//! certifica cuando hay al menos una aprobación y todos los hitos están
//! completos, y en el demo day la comunidad vota por los proyectos.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address,
    Env, String,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

/// Estados del proyecto.
pub const ACTIVO: u32 = 0;
pub const CERTIFICADO: u32 = 1;
pub const ARCHIVADO: u32 = 2;

/// Estados de una revisión entre pares.
pub const PEDIDA: u32 = 0;
pub const ACEPTADA: u32 = 1;
pub const APROBADA: u32 = 2;
pub const CON_CAMBIOS: u32 = 3;

#[contracttype]
enum Clave {
    Siguiente,
    SiguienteDemo,
    Proyecto(u32),
    Hito(u32, u32),
    Revision(u32, Address),
    Colaborador(u32, Address),
    Demo(u32),
    Voto(u32, Address),
    Votos(u32, u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proyecto {
    pub autor: Address,
    pub mentor: Address,
    pub nombre: String,
    pub repo: String,
    pub estado: u32,
    pub hitos: u32,
    pub hitos_completos: u32,
    pub aprobaciones: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemoDay {
    pub organizador: Address,
    pub nombre: String,
    pub abierto: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NoExiste = 1,
    NoEsAutor = 2,
    NoEsMentor = 3,
    NoActivo = 4,
    HitoNoExiste = 5,
    HitoYaCompleto = 6,
    RevisorInvalido = 7,
    RevisionYaPedida = 8,
    EstadoRevision = 9,
    FaltanRequisitos = 10,
    DemoNoExiste = 11,
    DemoCerrado = 12,
    NoEsOrganizador = 13,
    YaVoto = 14,
    VotoPropio = 15,
    NoAutorizado = 16,
    YaEsColaborador = 17,
    NoEsColaborador = 18,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProyectoRegistrado { #[topic] pub id: u32, pub autor: Address, pub nombre: String }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoAgregado { #[topic] pub id: u32, pub hito: u32, pub titulo: String }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitoCompletado { #[topic] pub id: u32, pub hito: u32 }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionPedida { #[topic] pub id: u32, pub revisor: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionAceptada { #[topic] pub id: u32, pub revisor: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProyectoAprobadoPar { #[topic] pub id: u32, pub revisor: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CambiosPedidos { #[topic] pub id: u32, pub revisor: Address, pub nota: String }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProyectoCertificado { #[topic] pub id: u32, pub mentor: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemoAbierto { #[topic] pub demo: u32, pub organizador: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemoCerradoEv { #[topic] pub demo: u32 }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VotoDemo { #[topic] pub demo: u32, pub proyecto: u32, pub votante: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColaboradorAgregado { #[topic] pub id: u32, pub colaborador: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColaboradorQuitado { #[topic] pub id: u32, pub colaborador: Address }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProyectoArchivado { #[topic] pub id: u32 }
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoActualizado { #[topic] pub id: u32, pub repo: String }

#[contract]
pub struct Proyectos;

fn leer(env: &Env, id: u32) -> Proyecto {
    let k = Clave::Proyecto(id);
    match env.storage().persistent().get(&k) {
        Some(p) => {
            env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
            p
        }
        None => panic_with_error!(env, Error::NoExiste),
    }
}

fn guardar<K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>, V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, k: &K, v: &V) {
    env.storage().persistent().set(k, v);
    env.storage().persistent().extend_ttl(k, TTL_UMBRAL, TTL_EXTENDER);
}

fn activo_del_autor(env: &Env, id: u32, autor: &Address) -> Proyecto {
    autor.require_auth();
    let p = leer(env, id);
    if p.autor != *autor {
        panic_with_error!(env, Error::NoEsAutor);
    }
    if p.estado != ACTIVO {
        panic_with_error!(env, Error::NoActivo);
    }
    p
}

fn revision(env: &Env, id: u32, revisor: &Address) -> u32 {
    match env.storage().persistent().get(&Clave::Revision(id, revisor.clone())) {
        Some(e) => e,
        None => panic_with_error!(env, Error::EstadoRevision),
    }
}

fn demo(env: &Env, d: u32) -> DemoDay {
    match env.storage().persistent().get(&Clave::Demo(d)) {
        Some(x) => x,
        None => panic_with_error!(env, Error::DemoNoExiste),
    }
}

#[contractimpl]
impl Proyectos {
    /// Quien aprende registra su proyecto final; `mentor` es quien lo certificará.
    pub fn registrar_proyecto(env: Env, autor: Address, mentor: Address, nombre: String, repo: String) -> u32 {
        autor.require_auth();
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        let p = Proyecto { autor: autor.clone(), mentor, nombre: nombre.clone(), repo, estado: ACTIVO, hitos: 0, hitos_completos: 0, aprobaciones: 0 };
        guardar(&env, &Clave::Proyecto(id), &p);
        ProyectoRegistrado { id, autor, nombre }.publish(&env);
        id
    }

    /// El autor agrega un hito; devuelve su número (desde 1).
    pub fn agregar_hito_proyecto(env: Env, id: u32, autor: Address, titulo: String) -> u32 {
        let mut p = activo_del_autor(&env, id, &autor);
        p.hitos += 1;
        let h = p.hitos;
        guardar(&env, &Clave::Hito(id, h), &false);
        guardar(&env, &Clave::Proyecto(id), &p);
        HitoAgregado { id, hito: h, titulo }.publish(&env);
        h
    }

    /// El autor marca un hito como completo.
    pub fn completar_hito_proyecto(env: Env, id: u32, autor: Address, hito: u32) {
        let mut p = activo_del_autor(&env, id, &autor);
        let k = Clave::Hito(id, hito);
        let hecho: bool = match env.storage().persistent().get(&k) {
            Some(x) => x,
            None => panic_with_error!(&env, Error::HitoNoExiste),
        };
        if hecho {
            panic_with_error!(&env, Error::HitoYaCompleto);
        }
        guardar(&env, &k, &true);
        p.hitos_completos += 1;
        guardar(&env, &Clave::Proyecto(id), &p);
        HitoCompletado { id, hito }.publish(&env);
    }

    /// El autor pide a un par que revise su proyecto.
    pub fn pedir_revision_par(env: Env, id: u32, autor: Address, revisor: Address) {
        let p = activo_del_autor(&env, id, &autor);
        if revisor == p.autor {
            panic_with_error!(&env, Error::RevisorInvalido);
        }
        let k = Clave::Revision(id, revisor.clone());
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::RevisionYaPedida);
        }
        guardar(&env, &k, &PEDIDA);
        RevisionPedida { id, revisor }.publish(&env);
    }

    /// El par acepta revisar.
    pub fn aceptar_revision_par(env: Env, id: u32, revisor: Address) {
        revisor.require_auth();
        leer(&env, id);
        if revision(&env, id, &revisor) != PEDIDA {
            panic_with_error!(&env, Error::EstadoRevision);
        }
        guardar(&env, &Clave::Revision(id, revisor.clone()), &ACEPTADA);
        RevisionAceptada { id, revisor }.publish(&env);
    }

    /// El par aprueba el proyecto (tras aceptar la revisión, o tras pedir cambios).
    pub fn aprobar_proyecto_par(env: Env, id: u32, revisor: Address) {
        revisor.require_auth();
        let mut p = leer(&env, id);
        if p.estado != ACTIVO {
            panic_with_error!(&env, Error::NoActivo);
        }
        let e = revision(&env, id, &revisor);
        if e != ACEPTADA && e != CON_CAMBIOS {
            panic_with_error!(&env, Error::EstadoRevision);
        }
        guardar(&env, &Clave::Revision(id, revisor.clone()), &APROBADA);
        p.aprobaciones += 1;
        guardar(&env, &Clave::Proyecto(id), &p);
        ProyectoAprobadoPar { id, revisor }.publish(&env);
    }

    /// El par pide cambios con una nota; si ya había aprobado, retira su aprobación.
    pub fn pedir_cambios_proyecto(env: Env, id: u32, revisor: Address, nota: String) {
        revisor.require_auth();
        let mut p = leer(&env, id);
        if p.estado != ACTIVO {
            panic_with_error!(&env, Error::NoActivo);
        }
        let e = revision(&env, id, &revisor);
        if e == PEDIDA || e == CON_CAMBIOS {
            panic_with_error!(&env, Error::EstadoRevision);
        }
        if e == APROBADA {
            p.aprobaciones -= 1;
            guardar(&env, &Clave::Proyecto(id), &p);
        }
        guardar(&env, &Clave::Revision(id, revisor.clone()), &CON_CAMBIOS);
        CambiosPedidos { id, revisor, nota }.publish(&env);
    }

    /// El mentor certifica: hace falta al menos una aprobación de un par y todos los hitos completos.
    pub fn certificar_proyecto(env: Env, id: u32, mentor: Address) {
        mentor.require_auth();
        let mut p = leer(&env, id);
        if p.mentor != mentor {
            panic_with_error!(&env, Error::NoEsMentor);
        }
        if p.estado != ACTIVO {
            panic_with_error!(&env, Error::NoActivo);
        }
        if p.aprobaciones == 0 || p.hitos == 0 || p.hitos_completos < p.hitos {
            panic_with_error!(&env, Error::FaltanRequisitos);
        }
        p.estado = CERTIFICADO;
        guardar(&env, &Clave::Proyecto(id), &p);
        ProyectoCertificado { id, mentor }.publish(&env);
    }

    /// Abre un demo day; devuelve su número.
    pub fn abrir_demo_day(env: Env, organizador: Address, nombre: String) -> u32 {
        organizador.require_auth();
        let d: u32 = env.storage().instance().get(&Clave::SiguienteDemo).unwrap_or(1);
        env.storage().instance().set(&Clave::SiguienteDemo, &(d + 1));
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        guardar(&env, &Clave::Demo(d), &DemoDay { organizador: organizador.clone(), nombre, abierto: true });
        DemoAbierto { demo: d, organizador }.publish(&env);
        d
    }

    /// Una persona vota una vez por demo day por un proyecto que no es suyo.
    pub fn votar_demo(env: Env, demo_id: u32, proyecto: u32, votante: Address) {
        votante.require_auth();
        if !demo(&env, demo_id).abierto {
            panic_with_error!(&env, Error::DemoCerrado);
        }
        let p = leer(&env, proyecto);
        if p.autor == votante {
            panic_with_error!(&env, Error::VotoPropio);
        }
        if p.estado == ARCHIVADO {
            panic_with_error!(&env, Error::NoActivo);
        }
        let kv = Clave::Voto(demo_id, votante.clone());
        if env.storage().persistent().has(&kv) {
            panic_with_error!(&env, Error::YaVoto);
        }
        guardar(&env, &kv, &proyecto);
        let kt = Clave::Votos(demo_id, proyecto);
        let n: u32 = env.storage().persistent().get(&kt).unwrap_or(0);
        guardar(&env, &kt, &(n + 1));
        VotoDemo { demo: demo_id, proyecto, votante }.publish(&env);
    }

    /// El organizador cierra la votación.
    pub fn cerrar_demo_day(env: Env, demo_id: u32, organizador: Address) {
        organizador.require_auth();
        let mut d = demo(&env, demo_id);
        if d.organizador != organizador {
            panic_with_error!(&env, Error::NoEsOrganizador);
        }
        if !d.abierto {
            panic_with_error!(&env, Error::DemoCerrado);
        }
        d.abierto = false;
        guardar(&env, &Clave::Demo(demo_id), &d);
        DemoCerradoEv { demo: demo_id }.publish(&env);
    }

    /// El autor suma un colaborador, que podrá actualizar el repo.
    pub fn agregar_colaborador(env: Env, id: u32, autor: Address, colaborador: Address) {
        activo_del_autor(&env, id, &autor);
        let k = Clave::Colaborador(id, colaborador.clone());
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::YaEsColaborador);
        }
        guardar(&env, &k, &true);
        ColaboradorAgregado { id, colaborador }.publish(&env);
    }

    /// El autor quita a un colaborador.
    pub fn quitar_colaborador(env: Env, id: u32, autor: Address, colaborador: Address) {
        activo_del_autor(&env, id, &autor);
        let k = Clave::Colaborador(id, colaborador.clone());
        if !env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::NoEsColaborador);
        }
        env.storage().persistent().remove(&k);
        ColaboradorQuitado { id, colaborador }.publish(&env);
    }

    /// El autor archiva su proyecto (deja de recibir revisiones y votos).
    pub fn archivar_proyecto(env: Env, id: u32, autor: Address) {
        autor.require_auth();
        let mut p = leer(&env, id);
        if p.autor != autor {
            panic_with_error!(&env, Error::NoEsAutor);
        }
        if p.estado == ARCHIVADO {
            panic_with_error!(&env, Error::NoActivo);
        }
        p.estado = ARCHIVADO;
        guardar(&env, &Clave::Proyecto(id), &p);
        ProyectoArchivado { id }.publish(&env);
    }

    /// El autor o un colaborador cambia el enlace al repo.
    pub fn actualizar_repo(env: Env, id: u32, quien: Address, repo: String) {
        quien.require_auth();
        let mut p = leer(&env, id);
        if p.estado == ARCHIVADO {
            panic_with_error!(&env, Error::NoActivo);
        }
        if p.autor != quien && !env.storage().persistent().has(&Clave::Colaborador(id, quien.clone())) {
            panic_with_error!(&env, Error::NoAutorizado);
        }
        p.repo = repo.clone();
        guardar(&env, &Clave::Proyecto(id), &p);
        RepoActualizado { id, repo }.publish(&env);
    }

    pub fn proyecto(env: Env, id: u32) -> Proyecto {
        leer(&env, id)
    }

    pub fn estado_revision(env: Env, id: u32, revisor: Address) -> u32 {
        revision(&env, id, &revisor)
    }

    pub fn votos_demo(env: Env, demo_id: u32, proyecto: u32) -> u32 {
        env.storage().persistent().get(&Clave::Votos(demo_id, proyecto)).unwrap_or(0)
    }
}

mod test;
