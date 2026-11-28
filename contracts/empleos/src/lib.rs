//! Empleos: una bolsa de trabajo donde las empresas contratan a egresados de
//! Budi cuyas insignias prueban sus habilidades.
//!
//! La empresa se registra y Budi la verifica, publica una vacante que pide una
//! insignia, los egresados se postulan, la empresa preselecciona, agenda la
//! entrevista y envía la oferta. Si el candidato acepta, la empresa deposita la
//! comisión de colocación en garantía: Budi la libera al terminar el periodo de
//! prueba, o la reembolsa si la contratación no se sostiene.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, token,
    Address, Env, String,
};

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

#[contracttype]
enum Clave {
    Admin,
    Token,
    Siguiente,
    Empresa(Address),
    Reportes(Address),
    Reporto(Address, Address),
    Vacante(u32),
    Postulacion(u32, Address),
    Garantia(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Empresa {
    pub nombre: String,
    pub verificada: bool,
}

/// Estado de una vacante: 0 abierta, 1 cubierta, 2 cerrada.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vacante {
    pub empresa: Address,
    pub titulo: String,
    pub insignia_requerida: u32,
    pub comision: i128,
    pub estado: u32,
    pub contratado: Option<Address>,
}

/// Estado: 0 postulado, 1 preseleccionado, 2 entrevista, 3 oferta, 4 aceptada, 5 rechazada.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Postulacion {
    pub enlace: String,
    pub estado: u32,
    pub entrevista: u64,
    pub salario: i128,
}

/// Estado: 0 en garantía, 1 liberada, 2 reembolsada.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Garantia {
    pub monto: i128,
    pub fin_prueba: u64,
    pub estado: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NoExiste = 1,
    YaRegistrada = 2,
    NoRegistrada = 3,
    NoVerificada = 4,
    VacanteNoAbierta = 5,
    YaPostulado = 6,
    NoPostulado = 7,
    EstadoInvalido = 8,
    MontoInvalido = 9,
    SinGarantia = 10,
    YaDepositada = 11,
    PruebaEnCurso = 12,
    GarantiaCerrada = 13,
    YaReporto = 14,
    AutoReporte = 15,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmpresaRegistrada {
    #[topic]
    pub empresa: Address,
    pub nombre: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmpresaVerificada {
    #[topic]
    pub empresa: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmpresaReportada {
    #[topic]
    pub empresa: Address,
    pub reportero: Address,
    pub motivo: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VacantePublicada {
    #[topic]
    pub id: u32,
    pub empresa: Address,
    pub insignia_requerida: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VacanteCerrada {
    #[topic]
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstadoPostulacion {
    #[topic]
    pub id: u32,
    #[topic]
    pub candidato: Address,
    pub estado: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostulacionRetirada {
    #[topic]
    pub id: u32,
    #[topic]
    pub candidato: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovimientoComision {
    #[topic]
    pub id: u32,
    pub estado: u32,
    pub monto: i128,
}

#[contract]
pub struct Empleos;

fn bump(env: &Env, clave: &Clave) {
    env.storage()
        .persistent()
        .extend_ttl(clave, TTL_UMBRAL, TTL_EXTENDER);
}

fn admin(env: &Env) -> Address {
    env.storage().instance().get(&Clave::Admin).unwrap()
}

fn token(env: &Env) -> token::Client<'_> {
    let id: Address = env.storage().instance().get(&Clave::Token).unwrap();
    token::Client::new(env, &id)
}

fn empresa_de(env: &Env, empresa: &Address) -> Empresa {
    env.storage()
        .persistent()
        .get(&Clave::Empresa(empresa.clone()))
        .unwrap_or_else(|| panic_with_error!(env, Error::NoRegistrada))
}

fn vacante_de(env: &Env, id: u32) -> Vacante {
    env.storage()
        .persistent()
        .get(&Clave::Vacante(id))
        .unwrap_or_else(|| panic_with_error!(env, Error::NoExiste))
}

fn guardar_vacante(env: &Env, id: u32, v: &Vacante) {
    let c = Clave::Vacante(id);
    env.storage().persistent().set(&c, v);
    bump(env, &c);
}

fn postulacion_de(env: &Env, id: u32, candidato: &Address) -> Postulacion {
    env.storage()
        .persistent()
        .get(&Clave::Postulacion(id, candidato.clone()))
        .unwrap_or_else(|| panic_with_error!(env, Error::NoPostulado))
}

fn guardar_postulacion(env: &Env, id: u32, candidato: &Address, p: &Postulacion) {
    let c = Clave::Postulacion(id, candidato.clone());
    env.storage().persistent().set(&c, p);
    bump(env, &c);
    EstadoPostulacion {
        id,
        candidato: candidato.clone(),
        estado: p.estado,
    }
    .publish(env);
}

/// La empresa dueña de la vacante firma y la postulación debe estar en `desde`.
fn avanzar(env: &Env, id: u32, candidato: &Address, desde: u32, hacia: u32) -> Postulacion {
    let v = vacante_de(env, id);
    v.empresa.require_auth();
    if v.estado != 0 {
        panic_with_error!(env, Error::VacanteNoAbierta);
    }
    let mut p = postulacion_de(env, id, candidato);
    if p.estado != desde {
        panic_with_error!(env, Error::EstadoInvalido);
    }
    p.estado = hacia;
    p
}

fn garantia_de(env: &Env, id: u32) -> Garantia {
    env.storage()
        .persistent()
        .get(&Clave::Garantia(id))
        .unwrap_or_else(|| panic_with_error!(env, Error::SinGarantia))
}

fn guardar_garantia(env: &Env, id: u32, g: &Garantia) {
    let c = Clave::Garantia(id);
    env.storage().persistent().set(&c, g);
    bump(env, &c);
    MovimientoComision {
        id,
        estado: g.estado,
        monto: g.monto,
    }
    .publish(env);
}

#[contractimpl]
impl Empleos {
    /// `admin` es Budi: verifica empresas y decide sobre la comisión.
    pub fn __constructor(env: Env, admin: Address, token: Address) {
        env.storage().instance().set(&Clave::Admin, &admin);
        env.storage().instance().set(&Clave::Token, &token);
        env.storage().instance().set(&Clave::Siguiente, &1u32);
    }

    pub fn registrar_empresa(env: Env, empresa: Address, nombre: String) {
        empresa.require_auth();
        let c = Clave::Empresa(empresa.clone());
        if env.storage().persistent().has(&c) {
            panic_with_error!(&env, Error::YaRegistrada);
        }
        env.storage().persistent().set(
            &c,
            &Empresa {
                nombre: nombre.clone(),
                verificada: false,
            },
        );
        bump(&env, &c);
        EmpresaRegistrada { empresa, nombre }.publish(&env);
    }

    /// Budi verifica a la empresa; solo las verificadas publican vacantes.
    pub fn verificar_empresa(env: Env, empresa: Address) {
        admin(&env).require_auth();
        let mut e = empresa_de(&env, &empresa);
        e.verificada = true;
        let c = Clave::Empresa(empresa.clone());
        env.storage().persistent().set(&c, &e);
        bump(&env, &c);
        EmpresaVerificada { empresa }.publish(&env);
    }

    /// Cualquiera, salvo la propia empresa, reporta a una empresa una vez.
    pub fn reportar_empresa(env: Env, reportero: Address, empresa: Address, motivo: String) {
        reportero.require_auth();
        if reportero == empresa {
            panic_with_error!(&env, Error::AutoReporte);
        }
        empresa_de(&env, &empresa);
        let ya = Clave::Reporto(empresa.clone(), reportero.clone());
        if env.storage().persistent().has(&ya) {
            panic_with_error!(&env, Error::YaReporto);
        }
        env.storage().persistent().set(&ya, &true);
        bump(&env, &ya);
        let c = Clave::Reportes(empresa.clone());
        let n: u32 = env.storage().persistent().get(&c).unwrap_or(0);
        env.storage().persistent().set(&c, &(n + 1));
        bump(&env, &c);
        EmpresaReportada {
            empresa,
            reportero,
            motivo,
        }
        .publish(&env);
    }

    pub fn publicar_vacante(
        env: Env,
        empresa: Address,
        titulo: String,
        insignia_requerida: u32,
        comision: i128,
    ) -> u32 {
        empresa.require_auth();
        if !empresa_de(&env, &empresa).verificada {
            panic_with_error!(&env, Error::NoVerificada);
        }
        if comision <= 0 {
            panic_with_error!(&env, Error::MontoInvalido);
        }
        let id: u32 = env.storage().instance().get(&Clave::Siguiente).unwrap();
        env.storage().instance().set(&Clave::Siguiente, &(id + 1));
        guardar_vacante(
            &env,
            id,
            &Vacante {
                empresa: empresa.clone(),
                titulo,
                insignia_requerida,
                comision,
                estado: 0,
                contratado: None,
            },
        );
        VacantePublicada {
            id,
            empresa,
            insignia_requerida,
        }
        .publish(&env);
        id
    }

    pub fn cerrar_vacante(env: Env, id: u32) {
        let mut v = vacante_de(&env, id);
        v.empresa.require_auth();
        if v.estado != 0 {
            panic_with_error!(&env, Error::VacanteNoAbierta);
        }
        v.estado = 2;
        guardar_vacante(&env, id, &v);
        VacanteCerrada { id }.publish(&env);
    }

    pub fn postular_vacante(env: Env, id: u32, candidato: Address, enlace: String) {
        candidato.require_auth();
        if vacante_de(&env, id).estado != 0 {
            panic_with_error!(&env, Error::VacanteNoAbierta);
        }
        if env
            .storage()
            .persistent()
            .has(&Clave::Postulacion(id, candidato.clone()))
        {
            panic_with_error!(&env, Error::YaPostulado);
        }
        guardar_postulacion(
            &env,
            id,
            &candidato,
            &Postulacion {
                enlace,
                estado: 0,
                entrevista: 0,
                salario: 0,
            },
        );
    }

    /// El candidato retira su postulación antes de aceptar una oferta.
    pub fn retirar_postulacion_vacante(env: Env, id: u32, candidato: Address) {
        candidato.require_auth();
        let p = postulacion_de(&env, id, &candidato);
        if p.estado >= 4 {
            panic_with_error!(&env, Error::EstadoInvalido);
        }
        env.storage()
            .persistent()
            .remove(&Clave::Postulacion(id, candidato.clone()));
        PostulacionRetirada { id, candidato }.publish(&env);
    }

    pub fn preseleccionar_candidato(env: Env, id: u32, candidato: Address) {
        let p = avanzar(&env, id, &candidato, 0, 1);
        guardar_postulacion(&env, id, &candidato, &p);
    }

    pub fn agendar_entrevista(env: Env, id: u32, candidato: Address, fecha: u64) {
        let mut p = avanzar(&env, id, &candidato, 1, 2);
        p.entrevista = fecha;
        guardar_postulacion(&env, id, &candidato, &p);
    }

    pub fn enviar_oferta(env: Env, id: u32, candidato: Address, salario: i128) {
        if salario <= 0 {
            panic_with_error!(&env, Error::MontoInvalido);
        }
        let mut p = avanzar(&env, id, &candidato, 2, 3);
        p.salario = salario;
        guardar_postulacion(&env, id, &candidato, &p);
    }

    /// Aceptar cubre la vacante.
    pub fn aceptar_oferta(env: Env, id: u32, candidato: Address) {
        candidato.require_auth();
        let mut v = vacante_de(&env, id);
        if v.estado != 0 {
            panic_with_error!(&env, Error::VacanteNoAbierta);
        }
        let mut p = postulacion_de(&env, id, &candidato);
        if p.estado != 3 {
            panic_with_error!(&env, Error::EstadoInvalido);
        }
        p.estado = 4;
        v.estado = 1;
        v.contratado = Some(candidato.clone());
        guardar_vacante(&env, id, &v);
        guardar_postulacion(&env, id, &candidato, &p);
    }

    pub fn rechazar_oferta(env: Env, id: u32, candidato: Address) {
        candidato.require_auth();
        let mut p = postulacion_de(&env, id, &candidato);
        if p.estado != 3 {
            panic_with_error!(&env, Error::EstadoInvalido);
        }
        p.estado = 5;
        guardar_postulacion(&env, id, &candidato, &p);
    }

    /// Con la vacante cubierta, la empresa deposita la comisión en garantía
    /// durante `prueba_segundos` de periodo de prueba.
    pub fn depositar_comision(env: Env, id: u32, prueba_segundos: u64) {
        let v = vacante_de(&env, id);
        v.empresa.require_auth();
        if v.estado != 1 {
            panic_with_error!(&env, Error::EstadoInvalido);
        }
        if env.storage().persistent().has(&Clave::Garantia(id)) {
            panic_with_error!(&env, Error::YaDepositada);
        }
        token(&env).transfer(&v.empresa, &env.current_contract_address(), &v.comision);
        guardar_garantia(
            &env,
            id,
            &Garantia {
                monto: v.comision,
                fin_prueba: env.ledger().timestamp() + prueba_segundos,
                estado: 0,
            },
        );
    }

    /// Terminado el periodo de prueba, Budi cobra la comisión.
    pub fn liberar_comision(env: Env, id: u32) {
        let a = admin(&env);
        a.require_auth();
        let mut g = garantia_de(&env, id);
        if g.estado != 0 {
            panic_with_error!(&env, Error::GarantiaCerrada);
        }
        if env.ledger().timestamp() < g.fin_prueba {
            panic_with_error!(&env, Error::PruebaEnCurso);
        }
        token(&env).transfer(&env.current_contract_address(), &a, &g.monto);
        g.estado = 1;
        guardar_garantia(&env, id, &g);
    }

    /// Si la contratación no se sostiene, Budi devuelve la comisión a la empresa.
    pub fn reembolsar_comision(env: Env, id: u32) {
        admin(&env).require_auth();
        let v = vacante_de(&env, id);
        let mut g = garantia_de(&env, id);
        if g.estado != 0 {
            panic_with_error!(&env, Error::GarantiaCerrada);
        }
        token(&env).transfer(&env.current_contract_address(), &v.empresa, &g.monto);
        g.estado = 2;
        guardar_garantia(&env, id, &g);
    }

    pub fn empresa(env: Env, empresa: Address) -> Empresa {
        empresa_de(&env, &empresa)
    }

    pub fn reportes_empresa(env: Env, empresa: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&Clave::Reportes(empresa))
            .unwrap_or(0)
    }

    pub fn vacante(env: Env, id: u32) -> Vacante {
        vacante_de(&env, id)
    }

    pub fn postulacion(env: Env, id: u32, candidato: Address) -> Postulacion {
        postulacion_de(&env, id, &candidato)
    }

    pub fn garantia(env: Env, id: u32) -> Garantia {
        garantia_de(&env, id)
    }
}

#[cfg(test)]
mod test;
