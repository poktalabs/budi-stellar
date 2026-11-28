extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};

/// Un registro de agentes mínimo para las pruebas: el agente 1 es de `Dueno`.
#[contract]
struct AgentesFalso;

#[contractimpl]
impl AgentesFalso {
    pub fn poner(env: Env, dueno: Address) {
        env.storage().instance().set(&1u32, &dueno);
    }
    pub fn dueno(env: Env, _id: u32) -> Address {
        env.storage().instance().get(&1u32).unwrap()
    }
}

struct Caso {
    env: Env,
    cta: CuentaClient<'static>,
    dueno: Address,
    token: Address,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let agentes = env.register(AgentesFalso, ());
    let dueno = Address::generate(&env);
    AgentesFalsoClient::new(&env, &agentes).poner(&dueno);
    let dir = env.register(Cuenta, (agentes, 1u32));
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &sac.address()).mint(&dir, &1_000);
    Caso { cta: CuentaClient::new(&env, &dir), env, dueno, token: sac.address() }
}

#[test]
fn revocar_gasto_deja_el_permiso_en_cero() {
    let c = caso();
    let servicio = Address::generate(&c.env);
    let hasta = c.env.ledger().sequence() + 100;
    c.cta.aprobar_gasto(&c.token, &servicio, &400, &hasta);
    let t = TokenClient::new(&c.env, &c.token);
    assert_eq!(t.allowance(&c.cta.address, &servicio), 400);
    c.cta.revocar_gasto(&c.token, &servicio);
    assert_eq!(c.env.auths()[0].0, c.dueno);
    assert_eq!(t.allowance(&c.cta.address, &servicio), 0);
}

#[test]
fn el_operador_paga_solo_dentro_del_tope() {
    let c = caso();
    let llave = Address::generate(&c.env);
    let ana = Address::generate(&c.env);
    c.cta.fijar_operador(&llave, &c.token, &100);
    assert_eq!(c.env.auths()[0].0, c.dueno);
    c.cta.pago_operador(&ana, &80);
    assert_eq!(c.env.auths()[0].0, llave);
    assert_eq!(TokenClient::new(&c.env, &c.token).balance(&ana), 80);
    assert_eq!(c.cta.try_pago_operador(&ana, &101), Err(Ok(Error::ExcedeTope)));
    assert_eq!(c.cta.try_pago_operador(&ana, &0), Err(Ok(Error::MontoInvalido)));
}

#[test]
fn quitar_operador_corta_sus_pagos() {
    let c = caso();
    let llave = Address::generate(&c.env);
    assert_eq!(c.cta.try_quitar_operador(), Err(Ok(Error::SinOperador)));
    c.cta.fijar_operador(&llave, &c.token, &100);
    c.cta.quitar_operador();
    assert_eq!(c.cta.operador(), None);
    assert_eq!(c.cta.try_pago_operador(&Address::generate(&c.env), &10), Err(Ok(Error::SinOperador)));
}

#[test]
fn rotar_la_llave_invalida_la_anterior() {
    let c = caso();
    let vieja = Address::generate(&c.env);
    let nueva = Address::generate(&c.env);
    c.cta.fijar_operador(&vieja, &c.token, &100);
    c.cta.fijar_operador(&nueva, &c.token, &50);
    let op = c.cta.operador().unwrap();
    assert_eq!(op.llave, nueva);
    assert_eq!(op.tope_por_pago, 50);
    assert_eq!(c.cta.try_fijar_operador(&nueva, &c.token, &0), Err(Ok(Error::MontoInvalido)));
}
