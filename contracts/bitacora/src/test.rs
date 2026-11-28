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

#[test]
fn corrige_un_hito_con_la_firma_del_builder() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    client.registrar(&builder, &String::from_str(&env, "Primer contrato"));
    client.registrar(&builder, &String::from_str(&env, "Pruebas en verde"));
    let nuevo = String::from_str(&env, "Mi primer contrato en Stellar");

    client.corregir(&builder, &1, &nuevo);

    assert_eq!(
        env.auths(),
        std::vec![(
            builder.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "corregir"),
                    (builder.clone(), 1u32, nuevo.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
    assert_eq!(
        client.hitos(&builder),
        vec![&env, nuevo, String::from_str(&env, "Pruebas en verde")]
    );
    assert_eq!(client.contar(&builder), 2);
}

#[test]
#[should_panic]
fn sin_firma_no_corrige() {
    let (env, client) = setup();
    let builder = Address::generate(&env);
    env.mock_all_auths();
    client.registrar(&builder, &String::from_str(&env, "Hito"));
    env.set_auths(&[]);
    client.corregir(&builder, &1, &String::from_str(&env, "Otro"));
}

#[test]
fn no_corrige_un_hito_que_no_existe_ni_con_texto_invalido() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Hito");
    assert_eq!(client.try_corregir(&builder, &1, &hito), Err(Ok(Error::HitoInexistente)));
    client.registrar(&builder, &hito);
    assert_eq!(client.try_corregir(&builder, &0, &hito), Err(Ok(Error::HitoInexistente)));
    assert_eq!(client.try_corregir(&builder, &2, &hito), Err(Ok(Error::HitoInexistente)));
    assert_eq!(
        client.try_corregir(&builder, &1, &String::from_str(&env, "")),
        Err(Ok(Error::HitoVacio))
    );
    let largo = "x".repeat(MAX_HITO as usize + 1);
    assert_eq!(
        client.try_corregir(&builder, &1, &String::from_str(&env, &largo)),
        Err(Ok(Error::HitoMuyLargo))
    );
    assert_eq!(client.hitos(&builder), vec![&env, hito]);
}

#[test]
fn publica_la_correccion_con_el_texto_de_antes() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let antes = String::from_str(&env, "Demo");
    let ahora = String::from_str(&env, "Demo en testnet");
    client.registrar(&builder, &antes);

    client.corregir(&builder, &1, &ahora);

    let esperado = HitoCorregido { builder: builder.clone(), numero: 1, antes, ahora };
    assert_eq!(env.events().all(), std::vec![esperado.to_xdr(&env, &client.address)]);
}

mod con_matricula {
    use super::*;
    use matricula::{Matricula, MatriculaClient as Escrow};
    use soroban_sdk::token::StellarAssetClient;

    fn caso() -> (Env, BitacoraClient<'static>, Escrow<'static>, Address) {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let alumno = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
        StellarAssetClient::new(&env, &sac.address()).mint(&alumno, &1_000);
        let escrow = Escrow::new(&env, &env.register(Matricula, (admin.clone(), sac.address())));
        escrow.crear_curso(&symbol_short!("soroban"), &100, &3600);
        client.configurar(&admin, &escrow.address, &symbol_short!("soroban"));
        (env, client, escrow, alumno)
    }

    #[test]
    fn sin_pagar_no_registra() {
        let (env, client, _, alumno) = caso();
        assert_eq!(
            client.try_registrar(&alumno, &String::from_str(&env, "Hola")),
            Err(Ok(Error::NoInscrito))
        );
    }

    #[test]
    fn con_el_curso_pagado_registra() {
        let (env, client, escrow, alumno) = caso();
        escrow.inscribir(&alumno, &symbol_short!("soroban"));
        assert_eq!(client.registrar(&alumno, &String::from_str(&env, "Pagué")), 1);
    }

    #[test]
    fn se_configura_una_sola_vez() {
        let (env, client, escrow, _) = caso();
        let otro = Address::generate(&env);
        assert_eq!(
            client.try_configurar(&otro, &escrow.address, &symbol_short!("rust")),
            Err(Ok(Error::YaConfigurada))
        );
    }
}
