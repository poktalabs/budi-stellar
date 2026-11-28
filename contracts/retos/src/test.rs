extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};

mod agentes {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/agentes.wasm");
}
mod cuentas {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/cuentas.wasm");
}
mod cuenta {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/cuenta.wasm");
}

struct Caso {
    env: Env,
    retos: RetosClient<'static>,
    agentes: agentes::Client<'static>,
    cuentas: cuentas::Client<'static>,
    xlm: TokenClient<'static>,
    creador: Address,
    dueno: Address,
    agente: u32,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let agentes = agentes::Client::new(&env, &env.register(agentes::WASM, ()));
    let wasm = env.deployer().upload_contract_wasm(cuenta::WASM);
    let cuentas = cuentas::Client::new(&env, &env.register(cuentas::WASM, (wasm,)));
    let retos = RetosClient::new(&env, &env.register(Retos, (agentes.address.clone(), cuentas.address.clone())));
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let creador = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&creador, &1_000);
    let dueno = Address::generate(&env);
    let agente = agentes.registrar_agente(&dueno, &String::from_str(&env, "uri"));
    let xlm = TokenClient::new(&env, &sac.address());
    Caso { env, retos, agentes, cuentas, xlm, creador, dueno, agente }
}

fn uri(env: &Env) -> String {
    String::from_str(env, "https://budi.lat/retos/1")
}

#[test]
fn el_premio_va_a_la_cuenta_del_agente_ganador() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    c.retos.postular(&id, &c.agente);
    let cuenta = c.retos.elegir_ganador(&id, &c.agente);
    assert_eq!(cuenta, c.cuentas.cuenta_de(&c.agentes.address, &c.agente));
    assert_eq!(c.xlm.balance(&cuenta), 300);
    assert_eq!(c.retos.reto(&id).estado, EstadoReto::Pagado);
    // Ya con su cuenta creada, el dueño del agente mueve lo cobrado.
    c.cuentas.crear_cuenta(&c.agentes.address, &c.agente);
    cuenta::Client::new(&c.env, &cuenta).pagar(&c.xlm.address, &c.dueno, &300);
    assert_eq!(c.xlm.balance(&c.dueno), 300);
}

#[test]
fn solo_gana_un_postulado_y_una_vez() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    assert_eq!(c.retos.try_elegir_ganador(&id, &c.agente), Err(Ok(Error::NoPostulado)));
    c.retos.postular(&id, &c.agente);
    assert_eq!(c.retos.try_postular(&id, &c.agente), Err(Ok(Error::YaPostulado)));
    c.retos.elegir_ganador(&id, &c.agente);
    assert_eq!(c.retos.try_elegir_ganador(&id, &c.agente), Err(Ok(Error::RetoCerrado)));
}

#[test]
fn el_creador_no_postula_a_sus_agentes() {
    let c = caso();
    let suyo = c.agentes.registrar_agente(&c.creador, &String::from_str(&c.env, "uri"));
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    assert_eq!(c.retos.try_postular(&id, &suyo), Err(Ok(Error::EsDelCreador)));
}

#[test]
fn cancelar_regresa_el_premio_y_cierra_el_reto() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    c.retos.cancelar_reto(&id);
    assert_eq!(c.xlm.balance(&c.creador), 1_000);
    assert_eq!(c.retos.try_postular(&id, &c.agente), Err(Ok(Error::RetoCerrado)));
}

#[test]
fn un_patrocinador_amplia_el_premio() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    let patrocinador = Address::generate(&c.env);
    StellarAssetClient::new(&c.env, &c.xlm.address).mint(&patrocinador, &200);
    assert_eq!(c.retos.ampliar_premio(&id, &patrocinador, &200), 500);
    c.retos.postular(&id, &c.agente);
    let cuenta = c.retos.elegir_ganador(&id, &c.agente);
    assert_eq!(c.xlm.balance(&cuenta), 500);
    assert_eq!(c.retos.try_ampliar_premio(&id, &patrocinador, &0), Err(Ok(Error::MontoInvalido)));
}

#[test]
fn un_agente_retira_su_postulacion() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    c.retos.postular(&id, &c.agente);
    c.retos.retirar_postulacion(&id, &c.agente);
    assert_eq!(c.retos.reto(&id).postulantes.len(), 0);
    assert_eq!(c.retos.try_retirar_postulacion(&id, &c.agente), Err(Ok(Error::NoPostulado)));
    assert_eq!(c.retos.try_elegir_ganador(&id, &c.agente), Err(Ok(Error::NoPostulado)));
}

#[test]
fn el_premio_se_divide_entre_las_cuentas_de_varios_agentes() {
    let c = caso();
    let otro = c.agentes.registrar_agente(&Address::generate(&c.env), &String::from_str(&c.env, "uri"));
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &301, &uri(&c.env));
    c.retos.postular(&id, &c.agente);
    c.retos.postular(&id, &otro);
    let lista = soroban_sdk::vec![&c.env, c.agente, otro];
    assert_eq!(c.retos.dividir_premio(&id, &lista), 150);
    assert_eq!(c.xlm.balance(&c.cuentas.cuenta_de(&c.agentes.address, &c.agente)), 151);
    assert_eq!(c.xlm.balance(&c.cuentas.cuenta_de(&c.agentes.address, &otro)), 150);
    assert_eq!(c.retos.try_dividir_premio(&id, &lista), Err(Ok(Error::RetoCerrado)));
}

#[test]
fn el_creador_rechaza_una_postulacion() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    c.retos.postular(&id, &c.agente);
    c.retos.rechazar_postulacion(&id, &c.agente);
    assert_eq!(c.env.auths()[0].0, c.creador);
    assert_eq!(c.retos.try_elegir_ganador(&id, &c.agente), Err(Ok(Error::NoPostulado)));
    assert_eq!(c.retos.try_rechazar_postulacion(&id, &c.agente), Err(Ok(Error::NoPostulado)));
}
