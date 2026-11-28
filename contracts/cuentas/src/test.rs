extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{symbol_short, vec, IntoVal, String, Symbol, Val, Vec};

mod agentes {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/agentes.wasm");
}
mod cuenta {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/cuenta.wasm");
}

struct Caso {
    env: Env,
    registro: CuentasClient<'static>,
    agentes: agentes::Client<'static>,
    dueno: Address,
    id: u32,
}

fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    let wasm = env.deployer().upload_contract_wasm(cuenta::WASM);
    let registro = CuentasClient::new(&env, &env.register(Cuentas, (wasm,)));
    let agentes = agentes::Client::new(&env, &env.register(agentes::WASM, ()));
    let dueno = Address::generate(&env);
    let id = agentes.registrar_agente(&dueno, &String::from_str(&env, "https://budi.lat/agentes/budi.json"));
    Caso { env, registro, agentes, dueno, id }
}

#[test]
fn la_cuenta_vive_en_la_direccion_que_se_sabe_de_antemano() {
    let c = caso();
    let prevista = c.registro.cuenta_de(&c.agentes.address, &c.id);
    let creada = c.registro.crear_cuenta(&c.agentes.address, &c.id);
    assert_eq!(prevista, creada);
    assert_eq!(c.registro.try_crear_cuenta(&c.agentes.address, &c.id), Err(Ok(Error::YaExiste)));
    let cta = cuenta::Client::new(&c.env, &creada);
    assert_eq!(cta.dueno(), c.dueno);
}

#[test]
#[should_panic]
fn no_hay_cuenta_para_un_agente_que_no_existe() {
    let c = caso();
    c.registro.crear_cuenta(&c.agentes.address, &99);
}

#[test]
fn la_cuenta_paga_y_se_va_con_el_agente_al_transferirlo() {
    let c = caso();
    let dir = c.registro.crear_cuenta(&c.agentes.address, &c.id);
    let cta = cuenta::Client::new(&c.env, &dir);
    let sac = c.env.register_stellar_asset_contract_v2(Address::generate(&c.env));
    StellarAssetClient::new(&c.env, &sac.address()).mint(&dir, &1_000);
    let xlm = TokenClient::new(&c.env, &sac.address());
    let ana = Address::generate(&c.env);

    cta.pagar(&sac.address(), &ana, &300);
    assert_eq!(c.env.auths()[0].0, c.dueno);
    assert_eq!(xlm.balance(&ana), 300);

    let nueva = Address::generate(&c.env);
    c.agentes.transferir_agente(&c.id, &nueva);
    assert_eq!(cta.dueno(), nueva);
    cta.pagar(&sac.address(), &ana, &200);
    assert_eq!(c.env.auths()[0].0, nueva);
    assert_eq!(xlm.balance(&dir), 500);
}

#[test]
fn la_cuenta_actua_en_otro_contrato_como_firmante() {
    let c = caso();
    let dir = c.registro.crear_cuenta(&c.agentes.address, &c.id);
    let cta = cuenta::Client::new(&c.env, &dir);
    // La cuenta del agente 1 califica a otro agente: ella es el cliente que firma.
    let otro = c.agentes.registrar_agente(&Address::generate(&c.env), &String::from_str(&c.env, "uri"));
    let args: Vec<Val> = vec![
        &c.env,
        dir.into_val(&c.env),
        otro.into_val(&c.env),
        80u32.into_val(&c.env),
        symbol_short!("codigo").into_val(&c.env),
        String::from_str(&c.env, "pr/9").into_val(&c.env),
    ];
    cta.ejecutar(&c.agentes.address, &Symbol::new(&c.env, "dar_feedback"), &args);
    assert_eq!(c.agentes.feedback(&otro, &0).cliente, dir);
}
