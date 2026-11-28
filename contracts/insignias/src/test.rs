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

#[test]
fn el_titular_renuncia_y_puede_volver_a_ganarla() {
    let (env, c, _emisor) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let ruta = symbol_short!("soroban");
    let id = c.otorgar(&ana, &ruta);
    c.renunciar(&id);
    assert!(!c.tiene(&ana, &ruta));
    assert_eq!(c.balance(&ana), 0);
    assert_eq!(c.try_insignia(&id), Err(Ok(Error::NoExiste)));
    assert_eq!(c.try_renunciar(&id), Err(Ok(Error::NoExiste)));
    let nueva = c.otorgar(&ana, &ruta);
    assert!(c.tiene(&ana, &ruta));
    assert!(nueva > id);
}

#[test]
fn el_emisor_fija_la_uri_y_cada_insignia_apunta_a_su_metadata() {
    let (env, c, emisor) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let id = c.otorgar(&ana, &symbol_short!("soroban"));
    for _ in 0..11 {
        c.otorgar(&Address::generate(&env), &symbol_short!("soroban"));
    }
    c.fijar_uri(&String::from_str(&env, "https://stellar.budi.lat/insignias/"));
    assert_eq!(env.auths()[0].0, emisor);
    assert_eq!(c.token_uri(&id), String::from_str(&env, "https://stellar.budi.lat/insignias/1.json"));
    assert_eq!(c.token_uri(&12), String::from_str(&env, "https://stellar.budi.lat/insignias/12.json"));
    assert_eq!(c.try_token_uri(&99), Err(Ok(Error::NoExiste)));
}

#[test]
fn otorga_en_lote_a_una_cohorte_todo_o_nada() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ruta = symbol_short!("soroban1");
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);
    let ids = client.otorgar_lote(&soroban_sdk::vec![&env, ana.clone(), beto.clone()], &ruta);
    assert_eq!(ids, soroban_sdk::vec![&env, 1u32, 2u32]);
    assert!(client.tiene(&ana, &ruta) && client.tiene(&beto, &ruta));
    // Si alguien ya la tiene, no se otorga a nadie.
    let caro = Address::generate(&env);
    let r = client.try_otorgar_lote(&soroban_sdk::vec![&env, caro.clone(), ana.clone()], &ruta);
    assert_eq!(r, Err(Ok(Error::YaLaTiene)));
    assert!(!client.tiene(&caro, &ruta));
}

#[test]
fn cambia_el_emisor_con_la_firma_de_los_dos() {
    let (env, client, emisor) = setup();
    env.mock_all_auths();
    let nuevo = Address::generate(&env);
    client.cambiar_emisor(&nuevo);
    let firmas: std::vec::Vec<Address> = env.auths().iter().map(|(a, _)| a.clone()).collect();
    assert!(firmas.contains(&emisor) && firmas.contains(&nuevo));
    assert_eq!(client.emisor(), nuevo);
}

#[test]
fn suspender_y_restaurar_una_insignia() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let ruta = symbol_short!("soroban1");
    client.otorgar(&ana, &ruta);
    let motivo = String::from_str(&env, "revision");
    assert_eq!(client.try_restaurar_insignia(&1), Err(Ok(Error::NoSuspendida)));
    client.suspender_insignia(&1, &motivo);
    assert!(!client.tiene(&ana, &ruta));
    assert_eq!(client.try_suspender_insignia(&1, &motivo), Err(Ok(Error::YaSuspendida)));
    client.restaurar_insignia(&1);
    assert!(client.tiene(&ana, &ruta));
}

#[test]
fn el_emisor_fija_el_nivel_de_una_insignia() {
    let (env, client, _) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let id = client.otorgar(&ana, &symbol_short!("soroban1"));
    assert_eq!(client.nivel(&id), 1);
    client.fijar_nivel(&id, &2);
    assert_eq!(client.nivel(&id), 2);
    assert_eq!(client.try_fijar_nivel(&id, &4), Err(Ok(Error::NivelInvalido)));
    assert_eq!(client.try_fijar_nivel(&99, &2), Err(Ok(Error::NoExiste)));
}
