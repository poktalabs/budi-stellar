#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Env, String};

fn preparar() -> (Env, CohortesClient<'static>, Address, Address, u32) {
    let env = Env::default();
    env.mock_all_auths();
    let id_c = env.register(Cohortes, ());
    let c = CohortesClient::new(&env, &id_c);
    let mentor = Address::generate(&env);
    let alumno = Address::generate(&env);
    let id = c.abrir_cohorte(&mentor, &String::from_str(&env, "Generacion 1"), &4, &3, &1);
    c.unirse_a_cohorte(&id, &alumno);
    (env, c, mentor, alumno, id)
}

#[test]
fn abre_cohorte_y_se_une_un_alumno() {
    let (_env, c, mentor, alumno, id) = preparar();
    let co = c.cohorte(&id);
    assert_eq!(co.mentor, mentor);
    assert_eq!(co.alumnos, 1);
    assert_eq!(c.progreso(&id, &alumno).asistencias, 0);
}

#[test]
fn no_se_une_dos_veces() {
    let (_env, c, _m, alumno, id) = preparar();
    assert_eq!(c.try_unirse_a_cohorte(&id, &alumno), Err(Ok(soroban_sdk::Error::from_contract_error(Error::YaInscrito as u32))));
}

#[test]
fn configuracion_invalida_se_rechaza() {
    let env = Env::default();
    env.mock_all_auths();
    let c = CohortesClient::new(&env, &env.register(Cohortes, ()));
    let m = Address::generate(&env);
    assert_eq!(
        c.try_abrir_cohorte(&m, &String::from_str(&env, "x"), &2, &3, &0),
        Err(Ok(soroban_sdk::Error::from_contract_error(Error::ConfigInvalida as u32)))
    );
}

#[test]
fn asistencia_una_vez_por_sesion_y_dentro_de_rango() {
    let (_env, c, _m, alumno, id) = preparar();
    c.marcar_asistencia(&id, &alumno, &1);
    assert_eq!(c.try_marcar_asistencia(&id, &alumno, &1), Err(Ok(soroban_sdk::Error::from_contract_error(Error::YaAsistio as u32))));
    assert_eq!(c.try_marcar_asistencia(&id, &alumno, &5), Err(Ok(soroban_sdk::Error::from_contract_error(Error::SesionInvalida as u32))));
    assert_eq!(c.progreso(&id, &alumno).asistencias, 1);
}

#[test]
fn tarea_entregada_y_calificada() {
    let (env, c, _m, alumno, id) = preparar();
    let enlace = String::from_str(&env, "https://github.com/x/y");
    assert_eq!(c.try_calificar_tarea(&id, &alumno, &1, &true), Err(Ok(soroban_sdk::Error::from_contract_error(Error::SinTarea as u32))));
    c.entregar_tarea(&id, &alumno, &1, &enlace);
    c.calificar_tarea(&id, &alumno, &1, &false);
    assert_eq!(c.progreso(&id, &alumno).tareas_aprobadas, 0);
    c.entregar_tarea(&id, &alumno, &1, &enlace);
    c.calificar_tarea(&id, &alumno, &1, &true);
    assert_eq!(c.progreso(&id, &alumno).tareas_aprobadas, 1);
    assert_eq!(c.try_entregar_tarea(&id, &alumno, &1, &enlace), Err(Ok(soroban_sdk::Error::from_contract_error(Error::YaAprobada as u32))));
}

#[test]
fn se_gradua_solo_si_cumple() {
    let (env, c, _m, alumno, id) = preparar();
    assert_eq!(c.try_graduar(&id, &alumno), Err(Ok(soroban_sdk::Error::from_contract_error(Error::NoCumple as u32))));
    for s in 1..=3 {
        c.marcar_asistencia(&id, &alumno, &s);
    }
    assert_eq!(c.try_graduar(&id, &alumno), Err(Ok(soroban_sdk::Error::from_contract_error(Error::NoCumple as u32))));
    c.entregar_tarea(&id, &alumno, &1, &String::from_str(&env, "https://a.b"));
    c.calificar_tarea(&id, &alumno, &1, &true);
    c.graduar(&id, &alumno);
    assert!(c.progreso(&id, &alumno).graduado);
    assert_eq!(c.try_salir_de_cohorte(&id, &alumno), Err(Ok(soroban_sdk::Error::from_contract_error(Error::YaGraduado as u32))));
}

#[test]
fn salir_y_cambiar_mentor() {
    let (env, c, _m, alumno, id) = preparar();
    c.salir_de_cohorte(&id, &alumno);
    assert_eq!(c.cohorte(&id).alumnos, 0);
    assert_eq!(c.try_progreso(&id, &alumno), Err(Ok(soroban_sdk::Error::from_contract_error(Error::NoInscrito as u32))));
    let nuevo = Address::generate(&env);
    c.asignar_mentor(&id, &nuevo);
    assert_eq!(c.cohorte(&id).mentor, nuevo);
}

#[test]
fn sin_firma_del_mentor_no_pasa_lista() {
    let env = Env::default();
    let c = CohortesClient::new(&env, &env.register(Cohortes, ()));
    let m = Address::generate(&env);
    env.mock_all_auths();
    let id = c.abrir_cohorte(&m, &String::from_str(&env, "g"), &2, &1, &0);
    let a = Address::generate(&env);
    c.unirse_a_cohorte(&id, &a);
    env.set_auths(&[]);
    assert!(c.try_marcar_asistencia(&id, &a, &1).is_err());
}

fn err(e: Error) -> soroban_sdk::Error {
    soroban_sdk::Error::from_contract_error(e as u32)
}

#[test]
fn cerrar_cohorte_bloquea_nuevos_alumnos() {
    let (env, c, _m, _a, id) = preparar();
    c.cerrar_cohorte(&id);
    let otro = Address::generate(&env);
    assert_eq!(c.try_unirse_a_cohorte(&id, &otro), Err(Ok(err(Error::Cerrada))));
    assert_eq!(c.try_cerrar_cohorte(&id), Err(Ok(err(Error::Cerrada))));
}

#[test]
fn agregar_sesion_y_justificar_falta() {
    let (_env, c, _m, alumno, id) = preparar();
    assert_eq!(c.agregar_sesion(&id), 5);
    c.justificar_falta(&id, &alumno, &5);
    assert_eq!(c.progreso(&id, &alumno).asistencias, 1);
    assert_eq!(c.try_marcar_asistencia(&id, &alumno, &5), Err(Ok(err(Error::YaAsistio))));
}

#[test]
fn reabrir_tarea_resta_la_aprobada() {
    let (env, c, _m, alumno, id) = preparar();
    c.entregar_tarea(&id, &alumno, &1, &String::from_str(&env, "https://x"));
    assert_eq!(c.try_reabrir_tarea(&id, &alumno, &1), Err(Ok(err(Error::NoAprobada))));
    c.calificar_tarea(&id, &alumno, &1, &true);
    c.reabrir_tarea(&id, &alumno, &1);
    assert_eq!(c.progreso(&id, &alumno).tareas_aprobadas, 0);
    c.entregar_tarea(&id, &alumno, &1, &String::from_str(&env, "https://x2"));
}

#[test]
fn expulsar_renombrar_y_cambiar_requisitos() {
    let (env, c, _m, alumno, id) = preparar();
    c.renombrar_cohorte(&id, &String::from_str(&env, "Generacion Uno"));
    assert_eq!(c.cohorte(&id).nombre, String::from_str(&env, "Generacion Uno"));
    c.cambiar_requisitos(&id, &1, &0);
    assert_eq!(c.cohorte(&id).asistencia_minima, 1);
    assert_eq!(c.try_cambiar_requisitos(&id, &9, &0), Err(Ok(err(Error::ConfigInvalida))));
    c.expulsar_de_cohorte(&id, &alumno);
    assert_eq!(c.cohorte(&id).alumnos, 0);
}

#[test]
#[should_panic]
fn solo_el_mentor_cierra() {
    let env = Env::default();
    let c = CohortesClient::new(&env, &env.register(Cohortes, ()));
    let m = Address::generate(&env);
    env.mock_all_auths();
    let id = c.abrir_cohorte(&m, &String::from_str(&env, "x"), &2, &1, &0);
    env.set_auths(&[]);
    c.cerrar_cohorte(&id);
}
