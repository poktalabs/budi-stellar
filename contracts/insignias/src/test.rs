extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger};
use soroban_sdk::{symbol_short, IntoVal};

fn setup() -> (Env, InsigniasClient<'static>, Address) {
    let env = Env::default();
    let emisor = Address::generate(&env);
    let id = env.register(Insignias, (emisor.clone(),));
    let client = InsigniasClient::new(&env, &id);
    (env, client, emisor)
}

#[test]
fn otorga_y_lee_insignias() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_795_000_000);
    let ana = Address::generate(&env);
    let ruta = symbol_short!("soroban1");

    assert!(!client.tiene(&ana, &ruta));
    assert_eq!(client.otorgar(&ana, &ruta), 1);

    assert!(client.tiene(&ana, &ruta));
    assert_eq!(client.owner_of(&1), ana);
    assert_eq!(client.balance(&ana), 1);
    let insignia = client.insignia(&1);
    assert_eq!(insignia.ruta, ruta);
    assert_eq!(insignia.otorgada_en, 1_795_000_000);
    assert!(!insignia.revocada);
    assert_eq!(client.insignias(&ana).len(), 1);
}

#[test]
fn ids_crecen_entre_personas_y_rutas() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);

    assert_eq!(client.otorgar(&ana, &symbol_short!("stellar")), 1);
    assert_eq!(client.otorgar(&beto, &symbol_short!("stellar")), 2);
    assert_eq!(client.otorgar(&ana, &symbol_short!("soroban1")), 3);
    assert_eq!(client.balance(&ana), 2);
    assert_eq!(client.balance(&beto), 1);
}

#[test]
fn solo_el_emisor_firma_al_otorgar() {
    let (env, client, emisor) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let ruta = symbol_short!("stellar");

    client.otorgar(&ana, &ruta);

    assert_eq!(
        env.auths(),
        std::vec![(
            emisor,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("otorgar"),
                    (ana, ruta).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

#[test]
#[should_panic]
fn sin_firma_del_emisor_falla() {
    let (env, client, _) = setup();
    let ana = Address::generate(&env);
    client.otorgar(&ana, &symbol_short!("stellar"));
}

#[test]
fn no_repite_la_misma_ruta() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let ruta = symbol_short!("stellar");

    client.otorgar(&ana, &ruta);
    assert_eq!(client.try_otorgar(&ana, &ruta), Err(Ok(Error::YaLaTiene)));
}

#[test]
fn revocar_deja_rastro() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let ruta = symbol_short!("stellar");
    client.otorgar(&ana, &ruta);

    client.revocar(&1, &String::from_str(&env, "Error al otorgar"));

    assert!(!client.tiene(&ana, &ruta));
    assert_eq!(client.balance(&ana), 0);
    assert!(client.insignia(&1).revocada);
    assert_eq!(client.insignias(&ana).len(), 1);
    assert_eq!(
        client.try_revocar(&1, &String::from_str(&env, "otra vez")),
        Err(Ok(Error::YaRevocada))
    );
    assert_eq!(
        client.try_revocar(&9, &String::from_str(&env, "no existe")),
        Err(Ok(Error::NoExiste))
    );
}

#[test]
fn coleccion_y_emisor() {
    let (env, client, emisor) = setup();
    assert_eq!(client.emisor(), emisor);
    assert_eq!(client.symbol(), String::from_str(&env, "BUDI"));
    assert_eq!(
        client.name(),
        String::from_str(&env, "Insignias Budi para Stellar")
    );
    assert_eq!(client.try_owner_of(&1), Err(Ok(Error::NoExiste)));
}
