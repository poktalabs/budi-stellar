//! Cohortes: una generación en vivo del curso de IA + blockchain de Budi.
//!
//! El mentor abre la cohorte, los alumnos se unen, el mentor pasa lista en
//! cada sesión y califica las tareas, y el alumno se gradúa solo cuando cumple
//! la asistencia mínima y las tareas aprobadas que la cohorte pide.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address,
    Env, String, Vec,
};

const TTL_UMBRAL: u32 = 17_280; // ~1 día en ledgers de 5 s
const TTL_EXTENDER: u32 = 518_400; // ~30 días

#[contracttype]
enum Clave {
    Siguiente,
    Cohorte(u32),
    Alumno(u32, Address),
    Asistio(u32, Address, u32),
    Tarea(u32, Address, u32),
    Cerrada(u32),
    Comentor(u32, Address),
    Inicio(u32),
    Pausada(u32),
    Anuncio(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cohorte {
    pub mentor: Address,
    pub nombre: String,
    pub sesiones: u32,
    pub asistencia_minima: u32,
    pub tareas_minimas: u32,
    pub alumnos: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Progreso {
    pub asistencias: u32,
    pub tareas_aprobadas: u32,
    pub graduado: bool,
}

/// Estado de una tarea entregada: 0 entregada, 1 aprobada, 2 por corregir.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tarea {
    pub enlace: String,
    pub estado: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NoExiste = 1,
    YaInscrito = 2,
    NoInscrito = 3,
    SesionInvalida = 4,
    YaAsistio = 5,
    SinTarea = 6,
    NoCumple = 7,
    YaGraduado = 8,
    ConfigInvalida = 9,
    YaAprobada = 10,
    Cerrada = 11,
    NoAprobada = 12,
    NoComentor = 13,
    YaComentor = 14,
    Pausada = 15,
    NoPausada = 16,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohorteAbierta {
    #[topic]
    pub id: u32,
    pub mentor: Address,
    pub nombre: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorAsignado {
    #[topic]
    pub id: u32,
    pub antes: Address,
    pub ahora: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlumnoUnido {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlumnoSalio {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsistenciaMarcada {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
    pub sesion: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TareaEntregada {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
    pub tarea: u32,
    pub enlace: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TareaCalificada {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
    pub tarea: u32,
    pub aprobada: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlumnoGraduado {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohorteCerrada {
    #[topic]
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SesionAgregada {
    #[topic]
    pub id: u32,
    pub sesiones: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaltaJustificada {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
    pub sesion: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TareaReabierta {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
    pub tarea: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlumnoExpulsado {
    #[topic]
    pub id: u32,
    #[topic]
    pub alumno: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohorteRenombrada {
    #[topic]
    pub id: u32,
    pub nombre: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequisitosCambiados {
    #[topic]
    pub id: u32,
    pub asistencia_minima: u32,
    pub tareas_minimas: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComentorAgregado {
    #[topic]
    pub id: u32,
    #[topic]
    pub comentor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComentorQuitado {
    #[topic]
    pub id: u32,
    #[topic]
    pub comentor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsistenciaLote {
    #[topic]
    pub id: u32,
    pub sesion: u32,
    pub marcados: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FechaInicioFijada {
    #[topic]
    pub id: u32,
    pub inicio: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohortePausada {
    #[topic]
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohorteReanudada {
    #[topic]
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CohorteAnunciada {
    #[topic]
    pub id: u32,
    pub mensaje: String,
}

#[contract]
pub struct Cohortes;

fn leer(env: &Env, id: u32) -> Cohorte {
    let k = Clave::Cohorte(id);
    let c: Cohorte = env
        .storage()
        .persistent()
        .get(&k)
        .unwrap_or_else(|| panic_with_error!(env, Error::NoExiste));
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
    c
}

fn guardar(env: &Env, id: u32, c: &Cohorte) {
    let k = Clave::Cohorte(id);
    env.storage().persistent().set(&k, c);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

fn progreso_de(env: &Env, id: u32, alumno: &Address) -> Progreso {
    let k = Clave::Alumno(id, alumno.clone());
    let p: Progreso = env
        .storage()
        .persistent()
        .get(&k)
        .unwrap_or_else(|| panic_with_error!(env, Error::NoInscrito));
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
    p
}

fn guardar_progreso(env: &Env, id: u32, alumno: &Address, p: &Progreso) {
    let k = Clave::Alumno(id, alumno.clone());
    env.storage().persistent().set(&k, p);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

fn poner(env: &Env, k: &Clave) {
    env.storage().persistent().set(k, &true);
    env.storage().persistent().extend_ttl(k, TTL_UMBRAL, TTL_EXTENDER);
}

fn no_pausada(env: &Env, id: u32) {
    if env.storage().persistent().has(&Clave::Pausada(id)) {
        panic_with_error!(env, Error::Pausada);
    }
}

/// Marca una asistencia sin revisar firmas (quien llama ya las revisó).
fn asistir(env: &Env, c: &Cohorte, id: u32, alumno: &Address, sesion: u32) {
    if sesion == 0 || sesion > c.sesiones {
        panic_with_error!(env, Error::SesionInvalida);
    }
    let mut p = progreso_de(env, id, alumno);
    let k = Clave::Asistio(id, alumno.clone(), sesion);
    if env.storage().persistent().has(&k) {
        panic_with_error!(env, Error::YaAsistio);
    }
    poner(env, &k);
    p.asistencias += 1;
    guardar_progreso(env, id, alumno, &p);
}

#[contractimpl]
impl Cohortes {
    /// El mentor abre una cohorte con su número de sesiones y lo que pide para graduarse.
    pub fn abrir_cohorte(
        env: Env,
        mentor: Address,
        nombre: String,
        sesiones: u32,
        asistencia_minima: u32,
        tareas_minimas: u32,
    ) -> u32 {
        mentor.require_auth();
        if sesiones == 0 || asistencia_minima > sesiones {
            panic_with_error!(&env, Error::ConfigInvalida);
        }
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
        let c = Cohorte {
            mentor: mentor.clone(),
            nombre: nombre.clone(),
            sesiones,
            asistencia_minima,
            tareas_minimas,
            alumnos: 0,
        };
        guardar(&env, id, &c);
        CohorteAbierta { id, mentor, nombre }.publish(&env);
        id
    }

    /// El mentor actual le pasa la cohorte a otro mentor.
    pub fn asignar_mentor(env: Env, id: u32, nuevo: Address) {
        let mut c = leer(&env, id);
        c.mentor.require_auth();
        let antes = c.mentor.clone();
        c.mentor = nuevo.clone();
        guardar(&env, id, &c);
        MentorAsignado { id, antes, ahora: nuevo }.publish(&env);
    }

    /// El alumno se une a la cohorte.
    pub fn unirse_a_cohorte(env: Env, id: u32, alumno: Address) {
        alumno.require_auth();
        let mut c = leer(&env, id);
        no_pausada(&env, id);
        if env.storage().persistent().has(&Clave::Cerrada(id)) {
            panic_with_error!(&env, Error::Cerrada);
        }
        if env.storage().persistent().has(&Clave::Alumno(id, alumno.clone())) {
            panic_with_error!(&env, Error::YaInscrito);
        }
        guardar_progreso(
            &env,
            id,
            &alumno,
            &Progreso { asistencias: 0, tareas_aprobadas: 0, graduado: false },
        );
        c.alumnos += 1;
        guardar(&env, id, &c);
        AlumnoUnido { id, alumno }.publish(&env);
    }

    /// El alumno deja la cohorte (no puede salir si ya se graduó).
    pub fn salir_de_cohorte(env: Env, id: u32, alumno: Address) {
        alumno.require_auth();
        let mut c = leer(&env, id);
        let p = progreso_de(&env, id, &alumno);
        if p.graduado {
            panic_with_error!(&env, Error::YaGraduado);
        }
        env.storage().persistent().remove(&Clave::Alumno(id, alumno.clone()));
        c.alumnos -= 1;
        guardar(&env, id, &c);
        AlumnoSalio { id, alumno }.publish(&env);
    }

    /// El mentor pasa lista: el alumno asistió a la sesión `sesion` (1..=sesiones).
    pub fn marcar_asistencia(env: Env, id: u32, alumno: Address, sesion: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        if sesion == 0 || sesion > c.sesiones {
            panic_with_error!(&env, Error::SesionInvalida);
        }
        let mut p = progreso_de(&env, id, &alumno);
        let k = Clave::Asistio(id, alumno.clone(), sesion);
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::YaAsistio);
        }
        env.storage().persistent().set(&k, &true);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        p.asistencias += 1;
        guardar_progreso(&env, id, &alumno, &p);
        AsistenciaMarcada { id, alumno, sesion }.publish(&env);
    }

    /// El alumno entrega (o reentrega) una tarea con el enlace a su trabajo.
    pub fn entregar_tarea(env: Env, id: u32, alumno: Address, tarea: u32, enlace: String) {
        alumno.require_auth();
        leer(&env, id);
        no_pausada(&env, id);
        progreso_de(&env, id, &alumno);
        let k = Clave::Tarea(id, alumno.clone(), tarea);
        if let Some(t) = env.storage().persistent().get::<_, Tarea>(&k) {
            if t.estado == 1 {
                panic_with_error!(&env, Error::YaAprobada);
            }
        }
        env.storage().persistent().set(&k, &Tarea { enlace: enlace.clone(), estado: 0 });
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        TareaEntregada { id, alumno, tarea, enlace }.publish(&env);
    }

    /// El mentor aprueba la tarea o la regresa para corregir.
    pub fn calificar_tarea(env: Env, id: u32, alumno: Address, tarea: u32, aprobada: bool) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Tarea(id, alumno.clone(), tarea);
        let mut t: Tarea = env
            .storage()
            .persistent()
            .get(&k)
            .unwrap_or_else(|| panic_with_error!(&env, Error::SinTarea));
        if t.estado == 1 {
            panic_with_error!(&env, Error::YaAprobada);
        }
        t.estado = if aprobada { 1 } else { 2 };
        env.storage().persistent().set(&k, &t);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        if aprobada {
            let mut p = progreso_de(&env, id, &alumno);
            p.tareas_aprobadas += 1;
            guardar_progreso(&env, id, &alumno, &p);
        }
        TareaCalificada { id, alumno, tarea, aprobada }.publish(&env);
    }

    /// El alumno se gradúa si cumple la asistencia mínima y las tareas aprobadas.
    pub fn graduar(env: Env, id: u32, alumno: Address) {
        alumno.require_auth();
        let c = leer(&env, id);
        let mut p = progreso_de(&env, id, &alumno);
        if p.graduado {
            panic_with_error!(&env, Error::YaGraduado);
        }
        if p.asistencias < c.asistencia_minima || p.tareas_aprobadas < c.tareas_minimas {
            panic_with_error!(&env, Error::NoCumple);
        }
        p.graduado = true;
        guardar_progreso(&env, id, &alumno, &p);
        AlumnoGraduado { id, alumno }.publish(&env);
    }

    /// El mentor cierra la cohorte: nadie nuevo puede unirse; los inscritos siguen y se gradúan.
    pub fn cerrar_cohorte(env: Env, id: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Cerrada(id);
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::Cerrada);
        }
        env.storage().persistent().set(&k, &true);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        CohorteCerrada { id }.publish(&env);
    }

    /// El mentor agrega una sesión más al calendario de la cohorte.
    pub fn agregar_sesion(env: Env, id: u32) -> u32 {
        let mut c = leer(&env, id);
        c.mentor.require_auth();
        c.sesiones += 1;
        guardar(&env, id, &c);
        SesionAgregada { id, sesiones: c.sesiones }.publish(&env);
        c.sesiones
    }

    /// El mentor justifica una falta: la sesión cuenta como asistencia.
    pub fn justificar_falta(env: Env, id: u32, alumno: Address, sesion: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        if sesion == 0 || sesion > c.sesiones {
            panic_with_error!(&env, Error::SesionInvalida);
        }
        let mut p = progreso_de(&env, id, &alumno);
        let k = Clave::Asistio(id, alumno.clone(), sesion);
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::YaAsistio);
        }
        env.storage().persistent().set(&k, &false);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        p.asistencias += 1;
        guardar_progreso(&env, id, &alumno, &p);
        FaltaJustificada { id, alumno, sesion }.publish(&env);
    }

    /// El mentor reabre una tarea aprobada para que el alumno la rehaga (antes de graduarse).
    pub fn reabrir_tarea(env: Env, id: u32, alumno: Address, tarea: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let mut p = progreso_de(&env, id, &alumno);
        if p.graduado {
            panic_with_error!(&env, Error::YaGraduado);
        }
        let k = Clave::Tarea(id, alumno.clone(), tarea);
        let mut t: Tarea = env
            .storage()
            .persistent()
            .get(&k)
            .unwrap_or_else(|| panic_with_error!(&env, Error::SinTarea));
        if t.estado != 1 {
            panic_with_error!(&env, Error::NoAprobada);
        }
        t.estado = 2;
        env.storage().persistent().set(&k, &t);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        p.tareas_aprobadas -= 1;
        guardar_progreso(&env, id, &alumno, &p);
        TareaReabierta { id, alumno, tarea }.publish(&env);
    }

    /// El mentor saca a un alumno no graduado de la cohorte.
    pub fn expulsar_de_cohorte(env: Env, id: u32, alumno: Address) {
        let mut c = leer(&env, id);
        c.mentor.require_auth();
        let p = progreso_de(&env, id, &alumno);
        if p.graduado {
            panic_with_error!(&env, Error::YaGraduado);
        }
        env.storage().persistent().remove(&Clave::Alumno(id, alumno.clone()));
        c.alumnos -= 1;
        guardar(&env, id, &c);
        AlumnoExpulsado { id, alumno }.publish(&env);
    }

    /// El mentor cambia el nombre de la cohorte.
    pub fn renombrar_cohorte(env: Env, id: u32, nombre: String) {
        let mut c = leer(&env, id);
        c.mentor.require_auth();
        c.nombre = nombre.clone();
        guardar(&env, id, &c);
        CohorteRenombrada { id, nombre }.publish(&env);
    }

    /// El mentor ajusta lo que pide la cohorte para graduarse.
    pub fn cambiar_requisitos(env: Env, id: u32, asistencia_minima: u32, tareas_minimas: u32) {
        let mut c = leer(&env, id);
        c.mentor.require_auth();
        if asistencia_minima > c.sesiones {
            panic_with_error!(&env, Error::ConfigInvalida);
        }
        c.asistencia_minima = asistencia_minima;
        c.tareas_minimas = tareas_minimas;
        guardar(&env, id, &c);
        RequisitosCambiados { id, asistencia_minima, tareas_minimas }.publish(&env);
    }

    /// El mentor suma un co-mentor que también puede pasar lista.
    pub fn agregar_comentor(env: Env, id: u32, comentor: Address) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Comentor(id, comentor.clone());
        if env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::YaComentor);
        }
        poner(&env, &k);
        ComentorAgregado { id, comentor }.publish(&env);
    }

    /// El mentor quita a un co-mentor.
    pub fn quitar_comentor(env: Env, id: u32, comentor: Address) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Comentor(id, comentor.clone());
        if !env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::NoComentor);
        }
        env.storage().persistent().remove(&k);
        ComentorQuitado { id, comentor }.publish(&env);
    }

    /// Un co-mentor pasa lista de un alumno en una sesión.
    pub fn marcar_asistencia_comentor(env: Env, id: u32, comentor: Address, alumno: Address, sesion: u32) {
        comentor.require_auth();
        let c = leer(&env, id);
        if !env.storage().persistent().has(&Clave::Comentor(id, comentor.clone())) {
            panic_with_error!(&env, Error::NoComentor);
        }
        asistir(&env, &c, id, &alumno, sesion);
        AsistenciaMarcada { id, alumno, sesion }.publish(&env);
    }

    /// El mentor pasa lista de varios alumnos en una sola transacción.
    pub fn marcar_asistencia_lote(env: Env, id: u32, alumnos: Vec<Address>, sesion: u32) -> u32 {
        let c = leer(&env, id);
        c.mentor.require_auth();
        for a in alumnos.iter() {
            asistir(&env, &c, id, &a, sesion);
        }
        let marcados = alumnos.len();
        AsistenciaLote { id, sesion, marcados }.publish(&env);
        marcados
    }

    /// El mentor fija la fecha de inicio (marca de tiempo Unix en segundos).
    pub fn fijar_fecha_inicio(env: Env, id: u32, inicio: u64) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Inicio(id);
        env.storage().persistent().set(&k, &inicio);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        FechaInicioFijada { id, inicio }.publish(&env);
    }

    /// El mentor pausa la cohorte: nadie se une ni entrega tareas hasta reanudar.
    pub fn pausar_cohorte(env: Env, id: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        no_pausada(&env, id);
        poner(&env, &Clave::Pausada(id));
        CohortePausada { id }.publish(&env);
    }

    /// El mentor reanuda una cohorte pausada.
    pub fn reanudar_cohorte(env: Env, id: u32) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Pausada(id);
        if !env.storage().persistent().has(&k) {
            panic_with_error!(&env, Error::NoPausada);
        }
        env.storage().persistent().remove(&k);
        CohorteReanudada { id }.publish(&env);
    }

    /// El mentor publica un aviso para la cohorte (queda el último).
    pub fn anunciar_cohorte(env: Env, id: u32, mensaje: String) {
        let c = leer(&env, id);
        c.mentor.require_auth();
        let k = Clave::Anuncio(id);
        env.storage().persistent().set(&k, &mensaje);
        env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
        CohorteAnunciada { id, mensaje }.publish(&env);
    }

    pub fn es_comentor(env: Env, id: u32, comentor: Address) -> bool {
        env.storage().persistent().has(&Clave::Comentor(id, comentor))
    }

    pub fn fecha_inicio(env: Env, id: u32) -> u64 {
        env.storage().persistent().get(&Clave::Inicio(id)).unwrap_or(0)
    }

    pub fn anuncio(env: Env, id: u32) -> Option<String> {
        env.storage().persistent().get(&Clave::Anuncio(id))
    }

    pub fn cohorte(env: Env, id: u32) -> Cohorte {
        leer(&env, id)
    }

    pub fn progreso(env: Env, id: u32, alumno: Address) -> Progreso {
        progreso_de(&env, id, &alumno)
    }
}

#[cfg(test)]
mod test;
