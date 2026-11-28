extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{symbol_short, Event};

mod insignias {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/insignias.wasm");
}

const CORRECTAS: &str = "bca";

/// Examen como emisor de Insignias, igual que en la red.
fn caso() -> (Env, ExamenClient<'static>, insignias::Client<'static>) {
    let env = Env::default();
    let insignias_id = Address::generate(&env);
    let huella: BytesN<32> = env.crypto().sha256(&Bytes::from_slice(&env, CORRECTAS.as_bytes())).into();
    let admin = Address::generate(&env);
    let examen_id = env.register(Examen, (admin, insignias_id.clone(), symbol_short!("soroban1"), huella));
    env.register_at(&insignias_id, insignias::WASM, (examen_id.clone(),));
    let examen = ExamenClient::new(&env, &examen_id);
    let insignias = insignias::Client::new(&env, &insignias_id);
    (env, examen, insignias)
}

fn resp(env: &Env, r: &str) -> Bytes {
    Bytes::from_slice(env, r.as_bytes())
}

#[test]
fn con_las_respuestas_correctas_gana_la_insignia() {
    let (env, examen, insignias) = caso();
    env.mock_all_auths();
    let ana = Address::generate(&env);

    let id = examen.presentar(&ana, &resp(&env, CORRECTAS));

    assert_eq!(id, 1);
    assert!(insignias.tiene(&ana, &symbol_short!("soroban1")));
    assert_eq!(insignias.owner_of(&id), ana);
    assert_eq!(examen.aprobado(&ana), Some(1));
}

#[test]
fn con_una_respuesta_mal_no_gana_nada_y_puede_reintentar() {
    let (env, examen, insignias) = caso();
    env.mock_all_auths();
    let ana = Address::generate(&env);

    assert_eq!(examen.try_presentar(&ana, &resp(&env, "bcb")), Err(Ok(Error::Incorrecto)));
    assert!(!insignias.tiene(&ana, &symbol_short!("soroban1")));
    assert_eq!(examen.aprobado(&ana), None);

    assert_eq!(examen.presentar(&ana, &resp(&env, CORRECTAS)), 1);
}

#[test]
fn una_insignia_por_persona() {
    let (env, examen, _) = caso();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    examen.presentar(&ana, &resp(&env, CORRECTAS));
    assert_eq!(examen.try_presentar(&ana, &resp(&env, CORRECTAS)), Err(Ok(Error::YaAprobo)));
}

#[test]
#[should_panic]
fn sin_la_firma_del_alumno_no_presenta() {
    let (env, examen, _) = caso();
    let ana = Address::generate(&env);
    examen.presentar(&ana, &resp(&env, CORRECTAS));
}

#[test]
#[should_panic]
fn nadie_mas_puede_otorgar_la_insignia() {
    let (env, _, insignias) = caso();
    let ana = Address::generate(&env);
    insignias.otorgar(&ana, &symbol_short!("soroban1"));
}

#[test]
fn publica_el_examen_aprobado() {
    let (env, examen, _) = caso();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    examen.presentar(&ana, &resp(&env, CORRECTAS));
    let esperado = ExamenAprobado { alumno: ana, ruta: symbol_short!("soroban1"), insignia: 1 };
    assert_eq!(
        env.events().all().filter_by_contract(&examen.address),
        std::vec![esperado.to_xdr(&env, &examen.address)]
    );
}

#[test]
fn el_admin_pausa_reanuda_y_publica_una_nueva_edicion() {
    let (env, examen, insignias) = caso();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);

    examen.pausar_examen();
    assert!(examen.pausado());
    assert_eq!(examen.try_presentar(&ana, &resp(&env, CORRECTAS)), Err(Ok(Error::Pausado)));

    examen.reanudar_examen();
    assert!(!examen.pausado());
    assert_eq!(examen.presentar(&ana, &resp(&env, CORRECTAS)), 1);

    let nueva: BytesN<32> = env.crypto().sha256(&resp(&env, "aab")).into();
    examen.actualizar_examen(&nueva);
    assert_eq!(examen.try_presentar(&beto, &resp(&env, CORRECTAS)), Err(Ok(Error::Incorrecto)));
    assert_eq!(examen.presentar(&beto, &resp(&env, "aab")), 2);
    assert!(insignias.tiene(&ana, &symbol_short!("soroban1")));
}

#[test]
#[should_panic]
fn sin_la_firma_del_admin_no_se_pausa() {
    let (_env, examen, _) = caso();
    examen.pausar_examen();
}
