//! Mentorías: el mercado de mentorías 1:1 de Budi para quien aprende IA + blockchain.
//!
//! El mentor se registra con su tarifa y publica horarios. El alumno reserva un
//! horario y su pago queda en garantía en este contrato. Si la sesión ocurre, el
//! alumno la confirma y el mentor cobra; si el mentor no llega, el alumno recupera
//! su dinero. Después el alumno califica y puede dejar propina.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, token,
    Address, Env, String, Vec,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

/// Estados de una reserva.
pub const RESERVADA: u32 = 0;
pub const CONFIRMADA: u32 = 1;
pub const CANCELADA: u32 = 2;
pub const NO_ASISTIO: u32 = 3;
pub const PAGADA: u32 = 4;
pub const REEMBOLSADA: u32 = 5;

#[contracttype]
enum Clave {
    Siguiente,
    Mentor(Address),
    Horario(Address, u64),
    Reserva(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mentor {
    pub token: Address,
    pub tarifa: i128,
    pub pausado: bool,
    pub activo: bool,
    pub temas: Vec<String>,
    pub calificaciones: u32,
    pub suma_estrellas: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reserva {
    pub mentor: Address,
    pub alumno: Address,
    pub inicio: u64,
    pub monto: i128,
    pub estado: u32,
    pub calificada: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    YaRegistrado = 1,
    NoEsMentor = 2,
    TarifaInvalida = 3,
    MentorNoDisponible = 4,
    HorarioNoDisponible = 5,
    NoExisteReserva = 6,
    EstadoInvalido = 7,
    NoEsElAlumno = 8,
    NoEsElMentor = 9,
    TodaviaNoEmpieza = 10,
    YaEmpezo = 11,
    EstrellasInvalidas = 12,
    YaCalificada = 13,
    MontoInvalido = 14,
    HorarioRepetido = 15,
}

#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorRegistrado { #[topic] pub mentor: Address, pub tarifa: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct HorarioPublicado { #[topic] pub mentor: Address, pub inicio: u64 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentoriaReservada { #[topic] pub id: u32, pub mentor: Address, pub alumno: Address, pub monto: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentoriaConfirmada { #[topic] pub id: u32 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservaCancelada { #[topic] pub id: u32, pub reembolso: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorNoAsistio { #[topic] pub id: u32, pub reembolso: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagoLiberado { #[topic] pub id: u32, pub monto: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorCalificado { #[topic] pub mentor: Address, pub id: u32, pub estrellas: u32 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorPausado { #[topic] pub mentor: Address, pub pausado: bool }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct TarifaCambiada { #[topic] pub mentor: Address, pub tarifa: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentoriaReprogramada { #[topic] pub id: u32, pub antes: u64, pub ahora: u64 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentoriaReembolsada { #[topic] pub id: u32, pub monto: i128 }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemaAgregado { #[topic] pub mentor: Address, pub tema: String }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorRetirado { #[topic] pub mentor: Address }
#[contractevent] #[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropinaDada { #[topic] pub id: u32, pub monto: i128 }

#[contract]
pub struct Mentorias;

#[contractimpl]
impl Mentorias {
    /// El mentor se registra con el token en que cobra y su tarifa por sesión.
    pub fn registrar_mentor(env: Env, mentor: Address, token: Address, tarifa: i128) {
        mentor.require_auth();
        if tarifa <= 0 { panic_with_error!(&env, Error::TarifaInvalida); }
        if env.storage().persistent().has(&Clave::Mentor(mentor.clone())) {
            panic_with_error!(&env, Error::YaRegistrado);
        }
        let m = Mentor { token, tarifa, pausado: false, activo: true, temas: Vec::new(&env), calificaciones: 0, suma_estrellas: 0 };
        guardar_mentor(&env, &mentor, &m);
        MentorRegistrado { mentor, tarifa }.publish(&env);
    }

    /// El mentor publica un horario libre (marca de tiempo de inicio, en segundos).
    pub fn publicar_horario(env: Env, mentor: Address, inicio: u64) {
        mentor.require_auth();
        let m = leer_mentor(&env, &mentor);
        if !m.activo { panic_with_error!(&env, Error::MentorNoDisponible); }
        let k = Clave::Horario(mentor.clone(), inicio);
        if env.storage().persistent().has(&k) { panic_with_error!(&env, Error::HorarioRepetido); }
        poner(&env, &k, &true);
        HorarioPublicado { mentor, inicio }.publish(&env);
    }

    /// El alumno reserva un horario publicado; la tarifa queda en garantía.
    pub fn reservar_mentoria(env: Env, alumno: Address, mentor: Address, inicio: u64) -> u32 {
        alumno.require_auth();
        let m = leer_mentor(&env, &mentor);
        if !m.activo || m.pausado { panic_with_error!(&env, Error::MentorNoDisponible); }
        tomar_horario(&env, &mentor, inicio);
        token::Client::new(&env, &m.token).transfer(&alumno, &env.current_contract_address(), &m.tarifa);
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap_or(1);
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        let r = Reserva { mentor: mentor.clone(), alumno: alumno.clone(), inicio, monto: m.tarifa, estado: RESERVADA, calificada: false };
        guardar_reserva(&env, id, &r);
        MentoriaReservada { id, mentor, alumno, monto: m.tarifa }.publish(&env);
        id
    }

    /// El alumno confirma que la sesión ocurrió.
    pub fn confirmar_mentoria(env: Env, alumno: Address, id: u32) {
        alumno.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != RESERVADA { panic_with_error!(&env, Error::EstadoInvalido); }
        r.estado = CONFIRMADA;
        guardar_reserva(&env, id, &r);
        MentoriaConfirmada { id }.publish(&env);
    }

    /// El mentor cobra una sesión que el alumno ya confirmó.
    pub fn liberar_pago_mentor(env: Env, mentor: Address, id: u32) {
        mentor.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.mentor != mentor { panic_with_error!(&env, Error::NoEsElMentor); }
        if r.estado != CONFIRMADA { panic_with_error!(&env, Error::EstadoInvalido); }
        let m = leer_mentor(&env, &mentor);
        token::Client::new(&env, &m.token).transfer(&env.current_contract_address(), &mentor, &r.monto);
        r.estado = PAGADA;
        guardar_reserva(&env, id, &r);
        PagoLiberado { id, monto: r.monto }.publish(&env);
    }

    /// El alumno cancela antes de que empiece: recupera su pago y el horario se libera.
    pub fn cancelar_reserva(env: Env, alumno: Address, id: u32) {
        alumno.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != RESERVADA { panic_with_error!(&env, Error::EstadoInvalido); }
        if env.ledger().timestamp() >= r.inicio { panic_with_error!(&env, Error::YaEmpezo); }
        devolver(&env, &r);
        poner(&env, &Clave::Horario(r.mentor.clone(), r.inicio), &true);
        r.estado = CANCELADA;
        guardar_reserva(&env, id, &r);
        ReservaCancelada { id, reembolso: r.monto }.publish(&env);
    }

    /// Ya pasó la hora y el mentor no llegó: el alumno lo marca y recupera su pago.
    pub fn marcar_no_asistio(env: Env, alumno: Address, id: u32) {
        alumno.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != RESERVADA { panic_with_error!(&env, Error::EstadoInvalido); }
        if env.ledger().timestamp() < r.inicio { panic_with_error!(&env, Error::TodaviaNoEmpieza); }
        devolver(&env, &r);
        r.estado = NO_ASISTIO;
        guardar_reserva(&env, id, &r);
        MentorNoAsistio { id, reembolso: r.monto }.publish(&env);
    }

    /// El alumno califica de 1 a 5 una sesión pagada, una sola vez.
    pub fn calificar_mentor(env: Env, alumno: Address, id: u32, estrellas: u32) {
        alumno.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != PAGADA { panic_with_error!(&env, Error::EstadoInvalido); }
        if r.calificada { panic_with_error!(&env, Error::YaCalificada); }
        if estrellas < 1 || estrellas > 5 { panic_with_error!(&env, Error::EstrellasInvalidas); }
        let mut m = leer_mentor(&env, &r.mentor);
        m.calificaciones += 1;
        m.suma_estrellas += estrellas;
        guardar_mentor(&env, &r.mentor, &m);
        r.calificada = true;
        guardar_reserva(&env, id, &r);
        MentorCalificado { mentor: r.mentor, id, estrellas }.publish(&env);
    }

    /// El mentor deja de recibir reservas nuevas por un tiempo (o vuelve).
    pub fn pausar_mentor(env: Env, mentor: Address, pausado: bool) {
        mentor.require_auth();
        let mut m = leer_mentor(&env, &mentor);
        m.pausado = pausado;
        guardar_mentor(&env, &mentor, &m);
        MentorPausado { mentor, pausado }.publish(&env);
    }

    /// El mentor cambia su tarifa; las reservas hechas conservan su monto.
    pub fn cambiar_tarifa_mentor(env: Env, mentor: Address, tarifa: i128) {
        mentor.require_auth();
        if tarifa <= 0 { panic_with_error!(&env, Error::TarifaInvalida); }
        let mut m = leer_mentor(&env, &mentor);
        m.tarifa = tarifa;
        guardar_mentor(&env, &mentor, &m);
        TarifaCambiada { mentor, tarifa }.publish(&env);
    }

    /// El alumno mueve su reserva a otro horario publicado del mismo mentor.
    pub fn reprogramar_mentoria(env: Env, alumno: Address, id: u32, nuevo_inicio: u64) {
        alumno.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != RESERVADA { panic_with_error!(&env, Error::EstadoInvalido); }
        if env.ledger().timestamp() >= r.inicio { panic_with_error!(&env, Error::YaEmpezo); }
        tomar_horario(&env, &r.mentor, nuevo_inicio);
        poner(&env, &Clave::Horario(r.mentor.clone(), r.inicio), &true);
        let antes = r.inicio;
        r.inicio = nuevo_inicio;
        guardar_reserva(&env, id, &r);
        MentoriaReprogramada { id, antes, ahora: nuevo_inicio }.publish(&env);
    }

    /// El mentor devuelve por su cuenta el pago de una reserva aún no cobrada.
    pub fn reembolsar_mentoria(env: Env, mentor: Address, id: u32) {
        mentor.require_auth();
        let mut r = leer_reserva(&env, id);
        if r.mentor != mentor { panic_with_error!(&env, Error::NoEsElMentor); }
        if r.estado != RESERVADA && r.estado != CONFIRMADA { panic_with_error!(&env, Error::EstadoInvalido); }
        devolver(&env, &r);
        r.estado = REEMBOLSADA;
        guardar_reserva(&env, id, &r);
        MentoriaReembolsada { id, monto: r.monto }.publish(&env);
    }

    /// El mentor agrega un tema que enseña (por ejemplo "Soroban" o "agentes").
    pub fn agregar_tema_mentor(env: Env, mentor: Address, tema: String) {
        mentor.require_auth();
        let mut m = leer_mentor(&env, &mentor);
        m.temas.push_back(tema.clone());
        guardar_mentor(&env, &mentor, &m);
        TemaAgregado { mentor, tema }.publish(&env);
    }

    /// El mentor se retira del mercado: no acepta horarios ni reservas nuevas.
    pub fn retirar_mentor(env: Env, mentor: Address) {
        mentor.require_auth();
        let mut m = leer_mentor(&env, &mentor);
        m.activo = false;
        guardar_mentor(&env, &mentor, &m);
        MentorRetirado { mentor }.publish(&env);
    }

    /// El alumno deja propina al mentor después de una sesión pagada.
    pub fn dar_propina_mentor(env: Env, alumno: Address, id: u32, monto: i128) {
        alumno.require_auth();
        if monto <= 0 { panic_with_error!(&env, Error::MontoInvalido); }
        let r = leer_reserva(&env, id);
        if r.alumno != alumno { panic_with_error!(&env, Error::NoEsElAlumno); }
        if r.estado != PAGADA { panic_with_error!(&env, Error::EstadoInvalido); }
        let m = leer_mentor(&env, &r.mentor);
        token::Client::new(&env, &m.token).transfer(&alumno, &r.mentor, &monto);
        PropinaDada { id, monto }.publish(&env);
    }

    pub fn mentor(env: Env, mentor: Address) -> Mentor { leer_mentor(&env, &mentor) }
    pub fn reserva(env: Env, id: u32) -> Reserva { leer_reserva(&env, id) }
    pub fn horario_libre(env: Env, mentor: Address, inicio: u64) -> bool {
        env.storage().persistent().get(&Clave::Horario(mentor, inicio)).unwrap_or(false)
    }
}

fn poner(env: &Env, k: &Clave, v: &bool) {
    env.storage().persistent().set(k, v);
    env.storage().persistent().extend_ttl(k, TTL_UMBRAL, TTL_EXTENDER);
}

fn tomar_horario(env: &Env, mentor: &Address, inicio: u64) {
    let k = Clave::Horario(mentor.clone(), inicio);
    let libre: bool = env.storage().persistent().get(&k).unwrap_or(false);
    if !libre { panic_with_error!(env, Error::HorarioNoDisponible); }
    poner(env, &k, &false);
}

fn devolver(env: &Env, r: &Reserva) {
    let m = leer_mentor(env, &r.mentor);
    token::Client::new(env, &m.token).transfer(&env.current_contract_address(), &r.alumno, &r.monto);
}

fn leer_mentor(env: &Env, mentor: &Address) -> Mentor {
    env.storage().persistent().get(&Clave::Mentor(mentor.clone()))
        .unwrap_or_else(|| panic_with_error!(env, Error::NoEsMentor))
}

fn guardar_mentor(env: &Env, mentor: &Address, m: &Mentor) {
    let k = Clave::Mentor(mentor.clone());
    env.storage().persistent().set(&k, m);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

fn leer_reserva(env: &Env, id: u32) -> Reserva {
    env.storage().persistent().get(&Clave::Reserva(id))
        .unwrap_or_else(|| panic_with_error!(env, Error::NoExisteReserva))
}

fn guardar_reserva(env: &Env, id: u32, r: &Reserva) {
    let k = Clave::Reserva(id);
    env.storage().persistent().set(&k, r);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

#[cfg(test)]
mod test;
