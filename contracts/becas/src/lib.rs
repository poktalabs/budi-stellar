//! Becas: fondos de becas para el curso de Budi, pagados en XLM por patrocinadores.
//!
//! Un patrocinador crea un fondo y deja el dinero en custodia, fija los criterios y
//! nombra a un comité. Los alumnos solicitan una beca; el comité la aprueba (el monto
//! queda reservado) o la rechaza, y libera el dinero por tramos conforme el alumno
//! avanza. Al cerrar el fondo, lo que nadie tiene reservado vuelve al patrocinador.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, token,
    Address, Env, String, Symbol, Vec,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fondo {
    pub patrocinador: Address,
    pub token: Address,
    /// Dinero del fondo que ninguna beca tiene reservado.
    pub disponible: i128,
    pub criterios: String,
    pub comite: Vec<Address>,
    pub cerrado: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EstadoBeca {
    Solicitada = 0,
    Aprobada = 1,
    Rechazada = 2,
    Suspendida = 3,
    Retirada = 4,
    Pagada = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beca {
    pub fondo: u32,
    pub alumno: Address,
    pub monto: i128,
    pub tramos: u32,
    pub tramos_pagados: u32,
    pub liberado: i128,
    pub motivo: String,
    pub estado: EstadoBeca,
}

#[contracttype]
enum Clave {
    SiguienteFondo,
    SiguienteBeca,
    Fondo(u32),
    Beca(u32),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    MontoInvalido = 1,
    FondoNoExiste = 2,
    BecaNoExiste = 3,
    NoEsComite = 4,
    YaEsComite = 5,
    FondoCerrado = 6,
    FondoAbierto = 7,
    EstadoInvalido = 8,
    SaldoInsuficiente = 9,
    MismoPatrocinador = 10,
}

/// Cada cambio en un fondo o una beca: qué pasó, sobre qué id, quién lo hizo y cuánto.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Movimiento {
    #[topic]
    pub accion: Symbol,
    #[topic]
    pub id: u32,
    pub quien: Address,
    pub monto: i128,
}

fn emitir(env: &Env, accion: Symbol, id: u32, quien: &Address, monto: i128) {
    Movimiento { accion, id, quien: quien.clone(), monto }.publish(env);
}

fn leer_fondo(env: &Env, id: u32) -> Result<Fondo, Error> {
    env.storage().persistent().get(&Clave::Fondo(id)).ok_or(Error::FondoNoExiste)
}

fn guardar_fondo(env: &Env, id: u32, f: &Fondo) {
    let k = Clave::Fondo(id);
    env.storage().persistent().set(&k, f);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

fn leer_beca(env: &Env, id: u32) -> Result<Beca, Error> {
    env.storage().persistent().get(&Clave::Beca(id)).ok_or(Error::BecaNoExiste)
}

fn guardar_beca(env: &Env, id: u32, b: &Beca) {
    let k = Clave::Beca(id);
    env.storage().persistent().set(&k, b);
    env.storage().persistent().extend_ttl(&k, TTL_UMBRAL, TTL_EXTENDER);
}

fn siguiente(env: &Env, k: Clave) -> u32 {
    let id: u32 = env.storage().instance().get(&k).unwrap_or(1);
    env.storage().instance().set(&k, &(id + 1));
    env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
    id
}

fn exigir_comite(f: &Fondo, miembro: &Address) -> Result<(), Error> {
    miembro.require_auth();
    if f.comite.contains(miembro) { Ok(()) } else { Err(Error::NoEsComite) }
}

#[contract]
pub struct Becas;

#[contractimpl]
impl Becas {
    /// El patrocinador crea un fondo y deposita el monto inicial.
    pub fn crear_fondo_beca(env: Env, patrocinador: Address, token: Address, monto: i128) -> Result<u32, Error> {
        patrocinador.require_auth();
        if monto <= 0 { return Err(Error::MontoInvalido); }
        token::Client::new(&env, &token).transfer(&patrocinador, env.current_contract_address(), &monto);
        let id = siguiente(&env, Clave::SiguienteFondo);
        let f = Fondo {
            patrocinador: patrocinador.clone(),
            token,
            disponible: monto,
            criterios: String::from_str(&env, ""),
            comite: Vec::new(&env),
            cerrado: false,
        };
        guardar_fondo(&env, id, &f);
        emitir(&env, symbol_short!("creado"), id, &patrocinador, monto);
        Ok(id)
    }

    /// Cualquiera aporta más dinero a un fondo abierto.
    pub fn aportar_fondo_beca(env: Env, fondo: u32, de: Address, monto: i128) -> Result<(), Error> {
        de.require_auth();
        if monto <= 0 { return Err(Error::MontoInvalido); }
        let mut f = leer_fondo(&env, fondo)?;
        if f.cerrado { return Err(Error::FondoCerrado); }
        token::Client::new(&env, &f.token).transfer(&de, env.current_contract_address(), &monto);
        f.disponible += monto;
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("aporte"), fondo, &de, monto);
        Ok(())
    }

    /// El patrocinador publica los criterios de la beca (un texto o un enlace).
    pub fn fijar_criterios_beca(env: Env, fondo: u32, criterios: String) -> Result<(), Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        f.criterios = criterios;
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("criterio"), fondo, &f.patrocinador, 0);
        Ok(())
    }

    /// El patrocinador suma a alguien al comité que decide las becas.
    pub fn agregar_comite(env: Env, fondo: u32, miembro: Address) -> Result<(), Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        if f.comite.contains(&miembro) { return Err(Error::YaEsComite); }
        f.comite.push_back(miembro.clone());
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("comite"), fondo, &miembro, 0);
        Ok(())
    }

    /// El patrocinador quita a alguien del comité.
    pub fn quitar_comite(env: Env, fondo: u32, miembro: Address) -> Result<(), Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        let i = f.comite.first_index_of(&miembro).ok_or(Error::NoEsComite)?;
        f.comite.remove(i);
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("sin_comit"), fondo, &miembro, 0);
        Ok(())
    }

    /// Un alumno pide una beca de `monto`, pagada en `tramos` partes.
    pub fn solicitar_beca(env: Env, fondo: u32, alumno: Address, monto: i128, tramos: u32, motivo: String) -> Result<u32, Error> {
        alumno.require_auth();
        if monto <= 0 || tramos == 0 { return Err(Error::MontoInvalido); }
        let f = leer_fondo(&env, fondo)?;
        if f.cerrado { return Err(Error::FondoCerrado); }
        let id = siguiente(&env, Clave::SiguienteBeca);
        let b = Beca { fondo, alumno: alumno.clone(), monto, tramos, tramos_pagados: 0, liberado: 0, motivo, estado: EstadoBeca::Solicitada };
        guardar_beca(&env, id, &b);
        emitir(&env, symbol_short!("solicitud"), id, &alumno, monto);
        Ok(id)
    }

    /// El alumno retira su solicitud antes de que el comité decida.
    pub fn retirar_solicitud_beca(env: Env, beca: u32) -> Result<(), Error> {
        let mut b = leer_beca(&env, beca)?;
        b.alumno.require_auth();
        if b.estado != EstadoBeca::Solicitada { return Err(Error::EstadoInvalido); }
        b.estado = EstadoBeca::Retirada;
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("retirada"), beca, &b.alumno, 0);
        Ok(())
    }

    /// Un miembro del comité aprueba la beca; el monto queda reservado del fondo.
    pub fn aprobar_beca(env: Env, beca: u32, miembro: Address) -> Result<(), Error> {
        let mut b = leer_beca(&env, beca)?;
        let mut f = leer_fondo(&env, b.fondo)?;
        exigir_comite(&f, &miembro)?;
        if b.estado != EstadoBeca::Solicitada { return Err(Error::EstadoInvalido); }
        if f.disponible < b.monto { return Err(Error::SaldoInsuficiente); }
        f.disponible -= b.monto;
        b.estado = EstadoBeca::Aprobada;
        guardar_fondo(&env, b.fondo, &f);
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("aprobada"), beca, &miembro, b.monto);
        Ok(())
    }

    /// Un miembro del comité rechaza la solicitud.
    pub fn rechazar_beca(env: Env, beca: u32, miembro: Address) -> Result<(), Error> {
        let mut b = leer_beca(&env, beca)?;
        let f = leer_fondo(&env, b.fondo)?;
        exigir_comite(&f, &miembro)?;
        if b.estado != EstadoBeca::Solicitada { return Err(Error::EstadoInvalido); }
        b.estado = EstadoBeca::Rechazada;
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("rechazo"), beca, &miembro, 0);
        Ok(())
    }

    /// El comité libera el siguiente tramo al alumno; el último tramo paga el resto.
    pub fn liberar_tramo_beca(env: Env, beca: u32, miembro: Address) -> Result<i128, Error> {
        let mut b = leer_beca(&env, beca)?;
        let f = leer_fondo(&env, b.fondo)?;
        exigir_comite(&f, &miembro)?;
        if b.estado != EstadoBeca::Aprobada { return Err(Error::EstadoInvalido); }
        let pago = if b.tramos_pagados + 1 == b.tramos { b.monto - b.liberado } else { b.monto / b.tramos as i128 };
        token::Client::new(&env, &f.token).transfer(&env.current_contract_address(), &b.alumno, &pago);
        b.tramos_pagados += 1;
        b.liberado += pago;
        if b.tramos_pagados == b.tramos { b.estado = EstadoBeca::Pagada; }
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("tramo"), beca, &b.alumno, pago);
        Ok(pago)
    }

    /// El comité suspende una beca aprobada (por ejemplo, si el alumno deja de avanzar).
    pub fn suspender_beca(env: Env, beca: u32, miembro: Address) -> Result<(), Error> {
        let mut b = leer_beca(&env, beca)?;
        let f = leer_fondo(&env, b.fondo)?;
        exigir_comite(&f, &miembro)?;
        if b.estado != EstadoBeca::Aprobada { return Err(Error::EstadoInvalido); }
        b.estado = EstadoBeca::Suspendida;
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("suspend"), beca, &miembro, 0);
        Ok(())
    }

    /// El comité reanuda una beca suspendida.
    pub fn reanudar_beca(env: Env, beca: u32, miembro: Address) -> Result<(), Error> {
        let mut b = leer_beca(&env, beca)?;
        let f = leer_fondo(&env, b.fondo)?;
        exigir_comite(&f, &miembro)?;
        if b.estado != EstadoBeca::Suspendida { return Err(Error::EstadoInvalido); }
        b.estado = EstadoBeca::Aprobada;
        guardar_beca(&env, beca, &b);
        emitir(&env, symbol_short!("reanuda"), beca, &miembro, 0);
        Ok(())
    }

    /// El patrocinador cierra el fondo: no entran solicitudes ni aportes nuevos.
    pub fn cerrar_fondo_beca(env: Env, fondo: u32) -> Result<(), Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        if f.cerrado { return Err(Error::FondoCerrado); }
        f.cerrado = true;
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("cerrado"), fondo, &f.patrocinador, 0);
        Ok(())
    }

    /// Con el fondo cerrado, el patrocinador recupera lo que ninguna beca reservó.
    pub fn devolver_sobrante_beca(env: Env, fondo: u32) -> Result<i128, Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        if !f.cerrado { return Err(Error::FondoAbierto); }
        let monto = f.disponible;
        if monto <= 0 { return Err(Error::SaldoInsuficiente); }
        token::Client::new(&env, &f.token).transfer(&env.current_contract_address(), &f.patrocinador, &monto);
        f.disponible = 0;
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("sobrante"), fondo, &f.patrocinador, monto);
        Ok(monto)
    }

    /// El patrocinador cede el fondo a otro patrocinador; firman los dos.
    pub fn transferir_patrocinio(env: Env, fondo: u32, nuevo: Address) -> Result<(), Error> {
        let mut f = leer_fondo(&env, fondo)?;
        f.patrocinador.require_auth();
        nuevo.require_auth();
        if f.patrocinador == nuevo { return Err(Error::MismoPatrocinador); }
        f.patrocinador = nuevo.clone();
        guardar_fondo(&env, fondo, &f);
        emitir(&env, symbol_short!("patrocin"), fondo, &nuevo, 0);
        Ok(())
    }

    pub fn fondo(env: Env, fondo: u32) -> Result<Fondo, Error> {
        leer_fondo(&env, fondo)
    }

    pub fn beca(env: Env, beca: u32) -> Result<Beca, Error> {
        leer_beca(&env, beca)
    }
}

#[cfg(test)]
mod test;
