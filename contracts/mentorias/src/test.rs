extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};

struct Caso {
    env: Env,
    c: MentoriasClient<'static>,
    xlm: TokenClient<'static>,
    mentor: Address,
    alumno: Address,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let mentor = Address::generate(&env);
    let alumno = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&alumno, &1_000);
    let id = env.register(Mentorias, ());
    let c = MentoriasClient::new(&env, &id);
    let xlm = TokenClient::new(&env, &sac.address());
    c.registrar_mentor(&mentor, &sac.address(), &100);
    Caso { env, c, xlm, mentor, alumno }
}

#[test]
fn registra_publica_temas_y_tarifa() {
    let k = caso();
    k.c.agregar_tema_mentor(&k.mentor, &String::from_str(&k.env, "Soroban"));
    k.c.cambiar_tarifa_mentor(&k.mentor, &150);
    k.c.publicar_horario(&k.mentor, &5_000);
    let m = k.c.mentor(&k.mentor);
    assert_eq!(m.tarifa, 150);
    assert_eq!(m.temas.len(), 1);
    assert!(k.c.horario_libre(&k.mentor, &5_000));
}

#[test]
fn reserva_confirma_cobra_califica_y_propina() {
    let k = caso();
    k.c.publicar_horario(&k.mentor, &5_000);
    let id = k.c.reservar_mentoria(&k.alumno, &k.mentor, &5_000);
    assert_eq!(k.xlm.balance(&k.alumno), 900);
    assert!(!k.c.horario_libre(&k.mentor, &5_000));
    k.c.confirmar_mentoria(&k.alumno, &id);
    k.c.liberar_pago_mentor(&k.mentor, &id);
    assert_eq!(k.xlm.balance(&k.mentor), 100);
    k.c.calificar_mentor(&k.alumno, &id, &5);
    assert_eq!(k.c.mentor(&k.mentor).suma_estrellas, 5);
    assert!(k.c.try_calificar_mentor(&k.alumno, &id, &4).is_err());
    k.c.dar_propina_mentor(&k.alumno, &id, &20);
    assert_eq!(k.xlm.balance(&k.mentor), 120);
}

#[test]
fn no_cobra_sin_confirmacion() {
    let k = caso();
    k.c.publicar_horario(&k.mentor, &5_000);
    let id = k.c.reservar_mentoria(&k.alumno, &k.mentor, &5_000);
    assert!(k.c.try_liberar_pago_mentor(&k.mentor, &id).is_err());
}

#[test]
fn reprograma_y_cancela_con_reembolso() {
    let k = caso();
    k.c.publicar_horario(&k.mentor, &5_000);
    k.c.publicar_horario(&k.mentor, &6_000);
    let id = k.c.reservar_mentoria(&k.alumno, &k.mentor, &5_000);
    k.c.reprogramar_mentoria(&k.alumno, &id, &6_000);
    assert!(k.c.horario_libre(&k.mentor, &5_000));
    assert_eq!(k.c.reserva(&id).inicio, 6_000);
    k.c.cancelar_reserva(&k.alumno, &id);
    assert_eq!(k.xlm.balance(&k.alumno), 1_000);
    assert!(k.c.horario_libre(&k.mentor, &6_000));
}

#[test]
fn si_el_mentor_no_llega_el_alumno_recupera_su_pago() {
    let k = caso();
    k.c.publicar_horario(&k.mentor, &2_000);
    let id = k.c.reservar_mentoria(&k.alumno, &k.mentor, &2_000);
    assert!(k.c.try_marcar_no_asistio(&k.alumno, &id).is_err());
    k.env.ledger().set_timestamp(2_500);
    k.c.marcar_no_asistio(&k.alumno, &id);
    assert_eq!(k.xlm.balance(&k.alumno), 1_000);
    assert_eq!(k.c.reserva(&id).estado, NO_ASISTIO);
}

#[test]
fn el_mentor_reembolsa_pausa_y_se_retira() {
    let k = caso();
    k.c.publicar_horario(&k.mentor, &5_000);
    k.c.publicar_horario(&k.mentor, &6_000);
    let id = k.c.reservar_mentoria(&k.alumno, &k.mentor, &5_000);
    k.c.reembolsar_mentoria(&k.mentor, &id);
    assert_eq!(k.xlm.balance(&k.alumno), 1_000);
    k.c.pausar_mentor(&k.mentor, &true);
    assert!(k.c.try_reservar_mentoria(&k.alumno, &k.mentor, &6_000).is_err());
    k.c.pausar_mentor(&k.mentor, &false);
    k.c.retirar_mentor(&k.mentor);
    assert!(k.c.try_reservar_mentoria(&k.alumno, &k.mentor, &6_000).is_err());
    assert!(k.c.try_publicar_horario(&k.mentor, &7_000).is_err());
}

#[test]
fn sin_firma_no_hay_reserva() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let mentor = Address::generate(&env);
    let alumno = Address::generate(&env);
    let id = env.register(Mentorias, ());
    let c = MentoriasClient::new(&env, &id);
    assert!(c.try_registrar_mentor(&mentor, &sac.address(), &100).is_err());
    let _ = alumno;
}
