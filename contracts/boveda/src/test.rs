extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::StellarAssetClient;

const BONO: i128 = 5555_0000000; // 5555 XLM en stroops

struct Caso {
    env: Env,
    budi: Address,
    token: token::Client<'static>,
    b: BovedaClient<'static>,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let budi = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &sac.address()).mint(&budi, &(BONO * 3));
    let hash: BytesN<32> = env.crypto().sha256(&Bytes::from_slice(&env, b"shipit")).into();
    let id = env.register(Boveda, (budi.clone(), sac.address(), hash));
    let b = BovedaClient::new(&env, &id);
    let token = token::Client::new(&env, &sac.address());
    Caso { env, budi, token, b }
}

fn codigo(env: &Env, s: &str) -> Bytes {
    Bytes::from_slice(env, s.as_bytes())
}

#[test]
fn depositar_llena_la_boveda() {
    let c = caso();
    assert_eq!(c.b.depositar(&c.budi, &BONO), BONO);
    assert_eq!(c.b.saldo(), BONO);
    assert_eq!(c.token.balance(&c.budi), BONO * 2);
}

#[test]
fn reclamar_con_el_codigo_se_lleva_todo() {
    let c = caso();
    c.b.depositar(&c.budi, &BONO);
    let baf = Address::generate(&c.env);
    assert_eq!(c.b.reclamar(&baf, &codigo(&c.env, "Ship It")), BONO);
    assert_eq!(c.token.balance(&baf), BONO);
    assert_eq!(c.b.saldo(), 0);
    assert_eq!(c.b.reclamado(&baf), BONO);
}

#[test]
fn el_codigo_no_distingue_mayusculas_ni_espacios() {
    let c = caso();
    for (i, s) in ["ship it", "SHIPIT", "  Ship  It \n"].iter().enumerate() {
        c.b.depositar(&c.budi, &(i as i128 + 1));
        let quien = Address::generate(&c.env);
        assert_eq!(c.b.reclamar(&quien, &codigo(&c.env, s)), i as i128 + 1);
    }
}

#[test]
fn codigo_incorrecto_no_abre() {
    let c = caso();
    c.b.depositar(&c.budi, &BONO);
    let quien = Address::generate(&c.env);
    assert_eq!(
        c.b.try_reclamar(&quien, &codigo(&c.env, "Ship")),
        Err(Ok(Error::CodigoIncorrecto))
    );
    assert_eq!(c.b.saldo(), BONO);
}

#[test]
fn cada_direccion_reclama_una_vez() {
    let c = caso();
    c.b.depositar(&c.budi, &BONO);
    let quien = Address::generate(&c.env);
    c.b.reclamar(&quien, &codigo(&c.env, "Ship It"));
    c.b.depositar(&c.budi, &BONO);
    assert_eq!(
        c.b.try_reclamar(&quien, &codigo(&c.env, "Ship It")),
        Err(Ok(Error::YaReclamaste))
    );
    assert_eq!(c.b.saldo(), BONO);
}

#[test]
fn boveda_vacia_no_paga() {
    let c = caso();
    let quien = Address::generate(&c.env);
    assert_eq!(
        c.b.try_reclamar(&quien, &codigo(&c.env, "Ship It")),
        Err(Ok(Error::BovedaVacia))
    );
}

#[test]
fn deposito_invalido() {
    let c = caso();
    assert_eq!(c.b.try_depositar(&c.budi, &0), Err(Ok(Error::MontoInvalido)));
}

#[test]
fn cerrar_regresa_el_saldo_al_admin() {
    let c = caso();
    c.b.depositar(&c.budi, &BONO);
    assert_eq!(c.b.cerrar(), BONO);
    assert_eq!(c.b.saldo(), 0);
    assert_eq!(c.token.balance(&c.budi), BONO * 3);
    assert_eq!(c.b.try_cerrar(), Err(Ok(Error::BovedaVacia)));
    let _ = &c.env;
}
