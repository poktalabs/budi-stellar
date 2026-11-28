extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::symbol_short;

fn caso() -> (Env, AgentesClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(Agentes, ());
    let dueno = Address::generate(&env);
    (env.clone(), AgentesClient::new(&env, &id), dueno)
}

fn s(env: &Env, t: &str) -> String {
    String::from_str(env, t)
}

#[test]
fn registra_y_transfiere_un_agente() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "https://budi.lat/agentes/budi.json"));
    assert_eq!(id, 1);
    assert_eq!(c.registrar_agente(&dueno, &s(&env, "ipfs://otro")), 2);
    let nueva = Address::generate(&env);
    c.transferir_agente(&id, &nueva);
    assert_eq!(c.dueno(&id), nueva);
    assert_eq!(c.try_transferir_agente(&id, &nueva), Err(Ok(Error::MismoDueno)));
    assert_eq!(c.try_registrar_agente(&dueno, &s(&env, "")), Err(Ok(Error::UriVacia)));
}

#[test]
#[should_panic]
fn solo_el_dueno_transfiere() {
    let env = Env::default();
    let c = AgentesClient::new(&env, &env.register(Agentes, ()));
    let dueno = Address::generate(&env);
    env.mock_all_auths();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    env.set_auths(&[]);
    c.transferir_agente(&id, &Address::generate(&env));
}

#[test]
fn las_calificaciones_hacen_el_promedio_y_se_pueden_revocar() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);
    assert_eq!(c.dar_feedback(&ana, &id, &90, &symbol_short!("codigo"), &s(&env, "pr/1")), 0);
    assert_eq!(c.dar_feedback(&beto, &id, &70, &symbol_short!("codigo"), &s(&env, "pr/2")), 1);
    assert_eq!(c.resumen(&id), Resumen { calificaciones: 2, suma: 160 });
    assert_eq!(c.try_revocar_feedback(&ana, &id, &1), Err(Ok(Error::NoEsTuFeedback)));
    c.revocar_feedback(&beto, &id, &1);
    assert_eq!(c.resumen(&id), Resumen { calificaciones: 1, suma: 90 });
    assert_eq!(c.try_revocar_feedback(&beto, &id, &1), Err(Ok(Error::YaRevocado)));
}

#[test]
fn nadie_se_califica_a_si_mismo_ni_pasa_de_100() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    assert_eq!(c.try_dar_feedback(&dueno, &id, &100, &symbol_short!("x"), &s(&env, "u")), Err(Ok(Error::AutoCalificacion)));
    let ana = Address::generate(&env);
    assert_eq!(c.try_dar_feedback(&ana, &id, &101, &symbol_short!("x"), &s(&env, "u")), Err(Ok(Error::PuntajeInvalido)));
}

#[test]
fn el_dueno_responde_una_calificacion() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    let ana = Address::generate(&env);
    c.dar_feedback(&ana, &id, &40, &symbol_short!("codigo"), &s(&env, "pr/1"));
    c.responder_feedback(&id, &0, &s(&env, "Corregido en pr/3"));
    assert_eq!(c.feedback(&id, &0).respuesta, s(&env, "Corregido en pr/3"));
}

#[test]
fn solo_el_validador_pedido_responde() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    let validador = Address::generate(&env);
    let trabajo = BytesN::from_array(&env, &[7; 32]);
    c.pedir_validacion(&id, &validador, &trabajo);
    assert_eq!(c.try_pedir_validacion(&id, &validador, &trabajo), Err(Ok(Error::ValidacionYaExiste)));
    c.responder_validacion(&trabajo, &85);
    assert_eq!(env.auths()[0].0, validador);
    let v = c.validacion(&trabajo).unwrap();
    assert_eq!(v.respuesta, Some(85));
    assert_eq!(c.try_responder_validacion(&BytesN::from_array(&env, &[1; 32]), &1), Err(Ok(Error::ValidacionNoExiste)));
}

#[test]
fn quemar_un_agente_lo_da_de_baja() {
    let (env, c, dueno) = caso();
    let id = c.registrar_agente(&dueno, &s(&env, "uri"));
    let ana = Address::generate(&env);
    c.dar_feedback(&ana, &id, &90, &symbol_short!("codigo"), &s(&env, "pr/1"));
    c.quemar_agente(&id);
    assert_eq!(c.try_agente(&id), Err(Ok(Error::AgenteNoExiste)));
    assert_eq!(c.try_dar_feedback(&ana, &id, &90, &symbol_short!("x"), &s(&env, "u")), Err(Ok(Error::AgenteNoExiste)));
    assert_eq!(c.feedback(&id, &0).puntaje, 90);
    assert_eq!(c.registrar_agente(&dueno, &s(&env, "uri")), id + 1);
}
