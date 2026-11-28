extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};

struct Caso {
    env: Env,
    becas: BecasClient<'static>,
    xlm: TokenClient<'static>,
    patrocinador: Address,
    comite: Address,
    alumno: Address,
    fondo: u32,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let xlm = TokenClient::new(&env, &sac.address());
    let patrocinador = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&patrocinador, &10_000);
    let becas = BecasClient::new(&env, &env.register(Becas, ()));
    let fondo = becas.crear_fondo_beca(&patrocinador, &sac.address(), &1_000);
    let comite = Address::generate(&env);
    becas.agregar_comite(&fondo, &comite);
    let alumno = Address::generate(&env);
    Caso { env, becas, xlm, patrocinador, comite, alumno, fondo }
}

fn s(env: &Env, t: &str) -> String {
    String::from_str(env, t)
}

#[test]
fn crea_aporta_y_fija_criterios() {
    let c = caso();
    c.becas.aportar_fondo_beca(&c.fondo, &c.patrocinador, &500);
    c.becas.fijar_criterios_beca(&c.fondo, &s(&c.env, "terminar la leccion 1"));
    let f = c.becas.fondo(&c.fondo);
    assert_eq!(f.disponible, 1_500);
    assert_eq!(f.criterios, s(&c.env, "terminar la leccion 1"));
    assert_eq!(c.xlm.balance(&c.becas.address), 1_500);
}

#[test]
fn el_comite_aprueba_y_libera_por_tramos() {
    let c = caso();
    let b = c.becas.solicitar_beca(&c.fondo, &c.alumno, &300, &3, &s(&c.env, "quiero aprender"));
    c.becas.aprobar_beca(&b, &c.comite);
    assert_eq!(c.becas.fondo(&c.fondo).disponible, 700);
    c.becas.liberar_tramo_beca(&b, &c.comite);
    c.becas.suspender_beca(&b, &c.comite);
    assert!(c.becas.try_liberar_tramo_beca(&b, &c.comite).is_err());
    c.becas.reanudar_beca(&b, &c.comite);
    c.becas.liberar_tramo_beca(&b, &c.comite);
    c.becas.liberar_tramo_beca(&b, &c.comite);
    assert_eq!(c.xlm.balance(&c.alumno), 300);
    assert_eq!(c.becas.beca(&b).estado, EstadoBeca::Pagada);
}

#[test]
fn solo_el_comite_decide() {
    let c = caso();
    let b = c.becas.solicitar_beca(&c.fondo, &c.alumno, &100, &1, &s(&c.env, "m"));
    let extrano = Address::generate(&c.env);
    assert_eq!(c.becas.try_aprobar_beca(&b, &extrano), Err(Ok(Error::NoEsComite)));
    c.becas.quitar_comite(&c.fondo, &c.comite);
    assert_eq!(c.becas.try_aprobar_beca(&b, &c.comite), Err(Ok(Error::NoEsComite)));
}

#[test]
fn rechazar_y_retirar() {
    let c = caso();
    let b1 = c.becas.solicitar_beca(&c.fondo, &c.alumno, &100, &1, &s(&c.env, "a"));
    c.becas.rechazar_beca(&b1, &c.comite);
    assert_eq!(c.becas.beca(&b1).estado, EstadoBeca::Rechazada);
    let b2 = c.becas.solicitar_beca(&c.fondo, &c.alumno, &100, &1, &s(&c.env, "b"));
    c.becas.retirar_solicitud_beca(&b2);
    assert_eq!(c.becas.beca(&b2).estado, EstadoBeca::Retirada);
    assert!(c.becas.try_aprobar_beca(&b2, &c.comite).is_err());
}

#[test]
fn no_aprueba_sin_saldo() {
    let c = caso();
    let b = c.becas.solicitar_beca(&c.fondo, &c.alumno, &5_000, &1, &s(&c.env, "m"));
    assert_eq!(c.becas.try_aprobar_beca(&b, &c.comite), Err(Ok(Error::SaldoInsuficiente)));
}

#[test]
fn cerrar_devolver_sobrante_y_ceder() {
    let c = caso();
    let b = c.becas.solicitar_beca(&c.fondo, &c.alumno, &400, &1, &s(&c.env, "m"));
    c.becas.aprobar_beca(&b, &c.comite);
    assert_eq!(c.becas.try_devolver_sobrante_beca(&c.fondo), Err(Ok(Error::FondoAbierto)));
    c.becas.cerrar_fondo_beca(&c.fondo);
    assert!(c.becas.try_solicitar_beca(&c.fondo, &c.alumno, &1, &1, &s(&c.env, "m")).is_err());
    let antes = c.xlm.balance(&c.patrocinador);
    assert_eq!(c.becas.devolver_sobrante_beca(&c.fondo), 600);
    assert_eq!(c.xlm.balance(&c.patrocinador), antes + 600);
    // Lo reservado sigue para el alumno.
    c.becas.liberar_tramo_beca(&b, &c.comite);
    assert_eq!(c.xlm.balance(&c.alumno), 400);
    let nuevo = Address::generate(&c.env);
    c.becas.transferir_patrocinio(&c.fondo, &nuevo);
    assert_eq!(c.becas.fondo(&c.fondo).patrocinador, nuevo);
}
