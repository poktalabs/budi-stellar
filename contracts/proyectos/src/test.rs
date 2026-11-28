#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Env, String};

fn s(env: &Env, t: &str) -> String {
    String::from_str(env, t)
}

#[test]
fn flujo_completo_del_proyecto() {
    let env = Env::default();
    env.mock_all_auths();
    let c = ProyectosClient::new(&env, &env.register(Proyectos, ()));
    let autor = Address::generate(&env);
    let mentor = Address::generate(&env);
    let par = Address::generate(&env);
    let colab = Address::generate(&env);
    let votante = Address::generate(&env);

    let id = c.registrar_proyecto(&autor, &mentor, &s(&env, "Boveda"), &s(&env, "github.com/a/b"));
    let h = c.agregar_hito_proyecto(&id, &autor, &s(&env, "Contrato"));
    c.completar_hito_proyecto(&id, &autor, &h);
    c.agregar_colaborador(&id, &autor, &colab);
    c.actualizar_repo(&id, &colab, &s(&env, "github.com/a/c"));
    c.quitar_colaborador(&id, &autor, &colab);
    assert!(c.try_actualizar_repo(&id, &colab, &s(&env, "x")).is_err());

    c.pedir_revision_par(&id, &autor, &par);
    c.aceptar_revision_par(&id, &par);
    c.pedir_cambios_proyecto(&id, &par, &s(&env, "faltan tests"));
    assert_eq!(c.estado_revision(&id, &par), CON_CAMBIOS);
    c.aprobar_proyecto_par(&id, &par);
    assert_eq!(c.proyecto(&id).aprobaciones, 1);
    c.certificar_proyecto(&id, &mentor);
    assert_eq!(c.proyecto(&id).estado, CERTIFICADO);
    assert_eq!(c.proyecto(&id).repo, s(&env, "github.com/a/c"));

    let d = c.abrir_demo_day(&mentor, &s(&env, "Demo day 1"));
    c.votar_demo(&d, &id, &votante);
    assert!(c.try_votar_demo(&d, &id, &votante).is_err());
    assert!(c.try_votar_demo(&d, &id, &autor).is_err());
    assert_eq!(c.votos_demo(&d, &id), 1);
    c.cerrar_demo_day(&d, &mentor);
    assert!(c.try_votar_demo(&d, &id, &par).is_err());

    c.archivar_proyecto(&id, &autor);
    assert_eq!(c.proyecto(&id).estado, ARCHIVADO);
}

#[test]
fn no_se_certifica_sin_aprobacion_ni_hitos() {
    let env = Env::default();
    env.mock_all_auths();
    let c = ProyectosClient::new(&env, &env.register(Proyectos, ()));
    let autor = Address::generate(&env);
    let mentor = Address::generate(&env);
    let id = c.registrar_proyecto(&autor, &mentor, &s(&env, "P"), &s(&env, "r"));
    assert!(c.try_certificar_proyecto(&id, &mentor).is_err());
    let h = c.agregar_hito_proyecto(&id, &autor, &s(&env, "h"));
    c.completar_hito_proyecto(&id, &autor, &h);
    assert!(c.try_certificar_proyecto(&id, &mentor).is_err());
    assert!(c.try_certificar_proyecto(&id, &autor).is_err());
}

#[test]
fn el_par_no_aprueba_sin_aceptar_y_el_autor_no_se_revisa() {
    let env = Env::default();
    env.mock_all_auths();
    let c = ProyectosClient::new(&env, &env.register(Proyectos, ()));
    let autor = Address::generate(&env);
    let par = Address::generate(&env);
    let id = c.registrar_proyecto(&autor, &autor, &s(&env, "P"), &s(&env, "r"));
    assert!(c.try_pedir_revision_par(&id, &autor, &autor).is_err());
    c.pedir_revision_par(&id, &autor, &par);
    assert!(c.try_aprobar_proyecto_par(&id, &par).is_err());
    assert!(c.try_pedir_revision_par(&id, &autor, &par).is_err());
}

#[test]
fn sin_firma_del_autor_no_se_registra() {
    let env = Env::default();
    let c = ProyectosClient::new(&env, &env.register(Proyectos, ()));
    let autor = Address::generate(&env);
    assert!(c.try_registrar_proyecto(&autor, &autor, &s(&env, "P"), &s(&env, "r")).is_err());
}
