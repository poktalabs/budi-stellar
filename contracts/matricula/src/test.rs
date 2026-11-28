extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{symbol_short, token::StellarAssetClient, vec, Bytes, BytesN};

const PRECIO: i128 = 100_0000000; // 100 XLM en stroops
const PLAZO: u64 = 30 * 24 * 60 * 60;

struct Caso {
    env: Env,
    admin: Address,
    alumno: Address,
    token: token::Client<'static>,
    m: MatriculaClient<'static>,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let alumno = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &sac.address()).mint(&alumno, &(PRECIO * 3));
    let id = env.register(Matricula, (admin.clone(), sac.address()));
    let m = MatriculaClient::new(&env, &id);
    m.crear_curso(&symbol_short!("soroban"), &PRECIO, &PLAZO);
    let token = token::Client::new(&env, &sac.address());
    Caso { env, admin, alumno, token, m }
}

#[test]
fn inscribir_deja_el_pago_en_custodia() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(id, 1);
    assert_eq!(c.token.balance(&c.m.address), PRECIO);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 2);
    assert!(c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.inscripcion(&id).estado, Estado::EnCustodia);
}

#[test]
fn no_se_inscribe_dos_veces_ni_a_un_curso_que_no_existe() {
    let c = caso();
    c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(
        c.m.try_inscribir(&c.alumno, &symbol_short!("soroban")),
        Err(Ok(Error::YaInscrito))
    );
    assert_eq!(
        c.m.try_inscribir(&c.alumno, &symbol_short!("rust")),
        Err(Ok(Error::CursoNoExiste))
    );
}

#[test]
fn completar_devuelve_el_deposito_al_alumno() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    c.m.completar(&id);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 3);
    assert_eq!(c.token.balance(&c.m.address), 0);
    assert_eq!(c.token.balance(&c.admin), 0);
    assert_eq!(c.m.inscripcion(&id).estado, Estado::Devuelta);
    assert!(c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.try_completar(&id), Err(Ok(Error::NoEstaEnCustodia)));
    assert_eq!(c.m.try_cobrar(&id), Err(Ok(Error::NoEstaEnCustodia)));
}

#[test]
fn no_se_completa_despues_del_plazo() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    c.env.ledger().with_mut(|l| l.timestamp += PLAZO);
    assert_eq!(c.m.try_completar(&id), Err(Ok(Error::PlazoVencido)));
}

#[test]
fn cobrar_solo_despues_del_plazo() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.m.try_cobrar(&id), Err(Ok(Error::PlazoNoVencido)));

    c.env.ledger().with_mut(|l| l.timestamp += PLAZO);
    c.m.cobrar(&id);
    assert_eq!(c.token.balance(&c.admin), PRECIO);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 2);
    assert_eq!(c.m.inscripcion(&id).estado, Estado::Cobrada);
    assert!(!c.m.inscrito(&c.alumno, &symbol_short!("soroban")));

    // Puede volver a inscribirse con un depósito nuevo.
    assert_eq!(c.m.inscribir(&c.alumno, &symbol_short!("soroban")), 2);
}

#[test]
fn firmas_requeridas() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.env.auths()[0].0, c.alumno);
    c.m.completar(&id);
    assert_eq!(c.env.auths()[0].0, c.admin);
}

#[test]
fn precio_invalido() {
    let c = caso();
    assert_eq!(
        c.m.try_crear_curso(&symbol_short!("gratis"), &0, &PLAZO),
        Err(Ok(Error::PrecioInvalido))
    );
}

fn codigo(c: &Caso, texto: &str) -> (Bytes, BytesN<32>) {
    let b = Bytes::from_slice(&c.env, texto.as_bytes());
    let h: BytesN<32> = c.env.crypto().sha256(&b).into();
    (b, h)
}

#[test]
fn un_codigo_da_acceso_gratis_una_sola_vez() {
    let c = caso();
    let (b, h) = codigo(&c, "ALFA-1234");
    c.m.crear_codigos(&symbol_short!("soroban"), &vec![&c.env, h]);

    let id = c.m.canjear(&c.alumno, &b);
    assert!(c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.inscripcion(&id).monto, 0);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 3);

    // Otro alumno no puede reusar el mismo código.
    let otro = Address::generate(&c.env);
    assert_eq!(c.m.try_canjear(&otro, &b), Err(Ok(Error::CodigoInvalido)));

    // Terminar o vencer sin depósito no mueve dinero.
    c.m.completar(&id);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 3);
}

#[test]
fn codigo_desconocido_no_sirve() {
    let c = caso();
    let (b, _) = codigo(&c, "NO-EXISTE");
    assert_eq!(c.m.try_canjear(&c.alumno, &b), Err(Ok(Error::CodigoInvalido)));
}

#[test]
fn con_codigo_no_se_inscribe_dos_veces() {
    let c = caso();
    let (b, h) = codigo(&c, "ALFA-5678");
    c.m.crear_codigos(&symbol_short!("soroban"), &vec![&c.env, h]);
    c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.m.try_canjear(&c.alumno, &b), Err(Ok(Error::YaInscrito)));
    // El código sigue disponible para alguien más.
    let otro = Address::generate(&c.env);
    c.m.canjear(&otro, &b);
}

#[test]
fn codigos_solo_para_cursos_que_existen() {
    let c = caso();
    let (_, h) = codigo(&c, "ALFA-0000");
    assert_eq!(
        c.m.try_crear_codigos(&symbol_short!("rust"), &vec![&c.env, h]),
        Err(Ok(Error::CursoNoExiste))
    );
}

#[test]
fn abandonar_regresa_la_mitad_y_deja_volver_a_inscribirse() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.m.abandonar(&id), PRECIO / 2);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 2 + PRECIO / 2);
    assert_eq!(c.token.balance(&c.admin), PRECIO / 2);
    assert_eq!(c.m.try_abandonar(&id), Err(Ok(Error::NoEstaEnCustodia)));
    assert!(!c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
}

#[test]
fn no_se_abandona_despues_del_plazo() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    c.env.ledger().with_mut(|l| l.timestamp += PLAZO);
    assert_eq!(c.m.try_abandonar(&id), Err(Ok(Error::PlazoVencido)));
}

#[test]
fn el_lugar_se_transfiere_a_otra_persona() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    let beto = Address::generate(&c.env);
    c.m.transferir_inscripcion(&id, &beto);
    assert!(c.m.inscrito(&beto, &symbol_short!("soroban")));
    assert!(!c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.try_transferir_inscripcion(&id, &beto), Err(Ok(Error::MismoAlumno)));
    c.m.completar(&id);
    assert_eq!(c.token.balance(&beto), PRECIO);
}

#[test]
fn extender_plazo_deja_completar_despues_del_limite_original() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    let antes = c.m.inscripcion(&id).limite;
    assert_eq!(c.m.extender_plazo(&id, &1000), antes + 1000);
    c.env.ledger().with_mut(|l| l.timestamp = antes + 10);
    c.m.completar(&id);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 3);
    assert_eq!(c.m.try_extender_plazo(&id, &1000), Err(Ok(Error::NoEstaEnCustodia)));
}

#[test]
fn cerrar_inscripciones_bloquea_nuevos_alumnos() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    c.m.cerrar_inscripciones(&symbol_short!("soroban"));
    let otro = Address::generate(&c.env);
    assert_eq!(
        c.m.try_inscribir(&otro, &symbol_short!("soroban")),
        Err(Ok(Error::CursoCerrado))
    );
    c.m.completar(&id);
    assert!(c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
}
