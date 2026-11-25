extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{symbol_short, token::StellarAssetClient};

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
fn liberar_paga_al_admin_y_sigue_inscrito() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    c.m.liberar(&id);
    assert_eq!(c.token.balance(&c.admin), PRECIO);
    assert_eq!(c.token.balance(&c.m.address), 0);
    assert_eq!(c.m.inscripcion(&id).estado, Estado::Liberada);
    assert!(c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.try_liberar(&id), Err(Ok(Error::NoEstaEnCustodia)));
    assert_eq!(c.m.try_reembolsar(&id), Err(Ok(Error::NoEstaEnCustodia)));
}

#[test]
fn reembolso_solo_despues_del_plazo() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.m.try_reembolsar(&id), Err(Ok(Error::PlazoNoVencido)));

    c.env.ledger().with_mut(|l| l.timestamp += PLAZO);
    c.m.reembolsar(&id);
    assert_eq!(c.token.balance(&c.alumno), PRECIO * 3);
    assert_eq!(c.m.inscripcion(&id).estado, Estado::Reembolsada);
    assert!(!c.m.inscrito(&c.alumno, &symbol_short!("soroban")));
    assert_eq!(c.m.try_liberar(&id), Err(Ok(Error::NoEstaEnCustodia)));

    // Tras un reembolso puede volver a inscribirse.
    assert_eq!(c.m.inscribir(&c.alumno, &symbol_short!("soroban")), 2);
}

#[test]
fn firmas_requeridas() {
    let c = caso();
    let id = c.m.inscribir(&c.alumno, &symbol_short!("soroban"));
    assert_eq!(c.env.auths()[0].0, c.alumno);
    c.m.liberar(&id);
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
