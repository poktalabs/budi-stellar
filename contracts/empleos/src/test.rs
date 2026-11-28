#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

struct Ctx<'a> {
    env: Env,
    admin: Address,
    empresa: Address,
    candidato: Address,
    c: EmpleosClient<'a>,
    tok: TokenClient<'a>,
}

fn ctx<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let empresa = Address::generate(&env);
    let candidato = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(&env, &sac.address()).mint(&empresa, &1_000);
    let id = env.register(Empleos, (admin.clone(), sac.address()));
    let c = EmpleosClient::new(&env, &id);
    let tok = TokenClient::new(&env, &sac.address());
    Ctx { env, admin, empresa, candidato, c, tok }
}

fn s(env: &Env, t: &str) -> String {
    String::from_str(env, t)
}

fn hasta_oferta(x: &Ctx) -> u32 {
    x.c.registrar_empresa(&x.empresa, &s(&x.env, "Acme"));
    x.c.verificar_empresa(&x.empresa);
    let id = x.c.publicar_vacante(&x.empresa, &s(&x.env, "Dev Soroban"), &1, &100);
    x.c.postular_vacante(&id, &x.candidato, &s(&x.env, "https://cv"));
    x.c.preseleccionar_candidato(&id, &x.candidato);
    x.c.agendar_entrevista(&id, &x.candidato, &1_000);
    x.c.enviar_oferta(&id, &x.candidato, &50_000);
    id
}

#[test]
fn contratacion_completa_y_comision_liberada_tras_la_prueba() {
    let x = ctx();
    let id = hasta_oferta(&x);
    assert_eq!(x.c.postulacion(&id, &x.candidato).estado, 3);
    x.c.aceptar_oferta(&id, &x.candidato);
    assert_eq!(x.c.vacante(&id).estado, 1);
    x.c.depositar_comision(&id, &60);
    assert_eq!(x.tok.balance(&x.empresa), 900);
    assert!(x.c.try_liberar_comision(&id).is_err());
    x.env.ledger().with_mut(|l| l.timestamp += 61);
    x.c.liberar_comision(&id);
    assert_eq!(x.tok.balance(&x.admin), 100);
    assert_eq!(x.c.garantia(&id).estado, 1);
}

#[test]
fn reembolso_si_la_contratacion_no_se_sostiene() {
    let x = ctx();
    let id = hasta_oferta(&x);
    x.c.aceptar_oferta(&id, &x.candidato);
    x.c.depositar_comision(&id, &60);
    x.c.reembolsar_comision(&id);
    assert_eq!(x.tok.balance(&x.empresa), 1_000);
    assert!(x.c.try_liberar_comision(&id).is_err());
}

#[test]
fn rechazar_retirar_y_cerrar() {
    let x = ctx();
    let id = hasta_oferta(&x);
    x.c.rechazar_oferta(&id, &x.candidato);
    assert_eq!(x.c.postulacion(&id, &x.candidato).estado, 5);
    let otro = Address::generate(&x.env);
    x.c.postular_vacante(&id, &otro, &s(&x.env, "https://otro"));
    x.c.retirar_postulacion_vacante(&id, &otro);
    assert!(x.c.try_postulacion(&id, &otro).is_err());
    x.c.cerrar_vacante(&id);
    assert!(x
        .c
        .try_postular_vacante(&id, &otro, &s(&x.env, "x"))
        .is_err());
}

#[test]
fn sin_verificar_no_publica_y_los_reportes_cuentan_una_vez() {
    let x = ctx();
    x.c.registrar_empresa(&x.empresa, &s(&x.env, "Acme"));
    assert!(x
        .c
        .try_publicar_vacante(&x.empresa, &s(&x.env, "t"), &1, &10)
        .is_err());
    x.c.reportar_empresa(&x.candidato, &x.empresa, &s(&x.env, "no paga"));
    assert!(x
        .c
        .try_reportar_empresa(&x.candidato, &x.empresa, &s(&x.env, "otra vez"))
        .is_err());
    assert!(x
        .c
        .try_reportar_empresa(&x.empresa, &x.empresa, &s(&x.env, "yo"))
        .is_err());
    assert_eq!(x.c.reportes_empresa(&x.empresa), 1);
}

#[test]
#[should_panic]
fn solo_la_empresa_avanza_al_candidato() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let empresa = Address::generate(&env);
    let candidato = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let id = env.register(Empleos, (admin.clone(), sac.address()));
    let c = EmpleosClient::new(&env, &id);
    env.mock_all_auths();
    c.registrar_empresa(&empresa, &String::from_str(&env, "Acme"));
    c.verificar_empresa(&empresa);
    let v = c.publicar_vacante(&empresa, &String::from_str(&env, "t"), &1, &10);
    c.postular_vacante(&v, &candidato, &String::from_str(&env, "cv"));
    env.set_auths(&[]);
    c.preseleccionar_candidato(&v, &candidato);
}
