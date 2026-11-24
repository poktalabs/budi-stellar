extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _};
use soroban_sdk::{symbol_short, vec, Event, IntoVal};

fn setup() -> (Env, BitacoraClient<'static>) {
    let env = Env::default();
    let id = env.register(Bitacora, ());
    let client = BitacoraClient::new(&env, &id);
    (env, client)
}

#[test]
fn registra_y_lee_hitos_en_orden() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);

    assert_eq!(client.contar(&builder), 0);
    assert_eq!(
        client.registrar(&builder, &String::from_str(&env, "Repo público")),
        1
    );
    assert_eq!(
        client.registrar(&builder, &String::from_str(&env, "Contrato con pruebas")),
        2
    );

    assert_eq!(
        client.hitos(&builder),
        vec![
            &env,
            String::from_str(&env, "Repo público"),
            String::from_str(&env, "Contrato con pruebas")
        ]
    );
    assert_eq!(client.contar(&builder), 2);
}

#[test]
fn pide_la_firma_del_builder() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Despegue");

    client.registrar(&builder, &hito);

    assert_eq!(
        env.auths(),
        std::vec![(
            builder.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("registrar"),
                    (builder.clone(), hito.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

#[test]
#[should_panic]
fn sin_firma_no_registra() {
    let (env, client) = setup();
    let builder = Address::generate(&env);
    client.registrar(&builder, &String::from_str(&env, "Sin firma"));
}

#[test]
fn rechaza_hito_vacio_o_largo() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);

    assert_eq!(
        client.try_registrar(&builder, &String::from_str(&env, "")),
        Err(Ok(Error::HitoVacio))
    );
    let largo = "x".repeat(MAX_HITO as usize + 1);
    assert_eq!(
        client.try_registrar(&builder, &String::from_str(&env, &largo)),
        Err(Ok(Error::HitoMuyLargo))
    );
    assert_eq!(client.contar(&builder), 0);
}

#[test]
fn la_bitacora_tiene_tope() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "hito");
    for _ in 0..MAX_HITOS {
        client.registrar(&builder, &hito);
    }
    assert_eq!(
        client.try_registrar(&builder, &hito),
        Err(Ok(Error::BitacoraLlena))
    );
}

#[test]
fn cada_builder_tiene_su_bitacora() {
    let (env, client) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);

    client.registrar(&ana, &String::from_str(&env, "Hito de Ana"));
    assert_eq!(client.contar(&ana), 1);
    assert_eq!(client.contar(&beto), 0);
}

#[test]
fn publica_un_evento_por_hito() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Demo");

    client.registrar(&builder, &hito);

    let esperado = HitoRegistrado {
        builder: builder.clone(),
        numero: 1,
        hito,
    };
    assert_eq!(
        env.events().all(),
        std::vec![esperado.to_xdr(&env, &client.address)]
    );
}
