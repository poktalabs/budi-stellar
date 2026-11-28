extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
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

#[test]
fn editar_reto_solo_sin_postulantes() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    let nueva = String::from_str(&c.env, "https://budi.lat/retos/1-v2");
    c.retos.editar_reto(&id, &nueva);
    assert_eq!(c.retos.reto(&id).uri, nueva);
    c.retos.postular(&id, &c.agente);
    assert_eq!(c.retos.try_editar_reto(&id, &uri(&c.env)), Err(Ok(Error::TienePostulantes)));
}

#[test]
fn reabrir_reto_cancelado_con_premio_nuevo() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    assert_eq!(c.retos.try_reabrir_reto(&id, &100), Err(Ok(Error::NoEstaCancelado)));
    c.retos.postular(&id, &c.agente);
    c.retos.cancelar_reto(&id);
    c.retos.reabrir_reto(&id, &200);
    let r = c.retos.reto(&id);
    assert_eq!(r.estado, EstadoReto::Abierto);
    assert_eq!(r.premio, 200);
    assert!(r.postulantes.is_empty());
    assert_eq!(c.xlm.balance(&c.creador), 800);
    assert_eq!(c.xlm.balance(&c.retos.address), 200);
}

#[test]
fn traspasar_reto_al_nuevo_creador() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    let empresa = Address::generate(&c.env);
    assert_eq!(c.retos.try_traspasar_reto(&id, &c.creador), Err(Ok(Error::MismoCreador)));
    c.retos.traspasar_reto(&id, &empresa);
    assert_eq!(c.retos.reto(&id).creador, empresa);
    c.retos.cancelar_reto(&id);
    assert_eq!(c.xlm.balance(&empresa), 300);
}

#[test]
fn la_fecha_limite_solo_se_amplia_y_despues_nadie_se_postula() {
    use soroban_sdk::testutils::Ledger;
    let c = caso();
    c.env.ledger().set_timestamp(1_000);
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &100, &uri(&c.env));
    assert_eq!(c.retos.try_ampliar_fecha_reto(&id, &500), Err(Ok(Error::FechaInvalida)));
    c.retos.ampliar_fecha_reto(&id, &2_000);
    assert_eq!(c.retos.try_ampliar_fecha_reto(&id, &1_500), Err(Ok(Error::FechaInvalida)));
    c.retos.ampliar_fecha_reto(&id, &3_000);
    c.env.ledger().set_timestamp(3_001);
    assert_eq!(c.retos.try_postular(&id, &c.agente), Err(Ok(Error::FechaVencida)));
}

#[test]
fn un_reto_con_invitados_solo_acepta_a_los_invitados() {
    let c = caso();
    let otro = c.agentes.registrar_agente(&Address::generate(&c.env), &String::from_str(&c.env, "uri2"));
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &100, &uri(&c.env));
    c.retos.invitar_agente(&id, &c.agente);
    assert_eq!(c.retos.try_invitar_agente(&id, &c.agente), Err(Ok(Error::YaInvitado)));
    assert_eq!(c.retos.try_postular(&id, &otro), Err(Ok(Error::NoInvitado)));
    c.retos.postular(&id, &c.agente);
    assert_eq!(c.retos.invitados(&id).len(), 1);
}

#[test]
fn el_creador_califica_la_entrega_ganadora_una_vez() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &100, &uri(&c.env));
    c.retos.postular(&id, &c.agente);
    let nota = String::from_str(&c.env, "entrega limpia");
    assert_eq!(c.retos.try_calificar_entrega(&id, &5, &nota), Err(Ok(Error::NoPagado)));
    c.retos.elegir_ganador(&id, &c.agente);
    assert_eq!(c.retos.try_calificar_entrega(&id, &6, &nota), Err(Ok(Error::NotaInvalida)));
    c.retos.calificar_entrega(&id, &5, &nota);
    assert_eq!(c.retos.try_calificar_entrega(&id, &4, &nota), Err(Ok(Error::YaCalificado)));
    let cal = c.retos.calificacion(&id).unwrap();
    assert_eq!((cal.agente, cal.nota), (c.agente, 5));
}

#[test]
fn un_reto_vencido_sin_postulantes_lo_cierra_cualquiera_y_el_premio_regresa() {
    let c = caso();
    c.env.ledger().set_timestamp(1_000);
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    let otro = Address::generate(&c.env);
    assert_eq!(c.retos.try_cerrar_reto_sin_ganador(&id, &otro), Err(Ok(Error::SinFecha)));
    c.retos.ampliar_fecha_reto(&id, &2_000);
    assert_eq!(c.retos.try_cerrar_reto_sin_ganador(&id, &otro), Err(Ok(Error::FechaInvalida)));
    c.env.ledger().set_timestamp(2_001);
    c.retos.cerrar_reto_sin_ganador(&id, &otro);
    assert_eq!(c.xlm.balance(&c.creador), 1_000);
    assert_eq!(c.retos.reto(&id).estado, EstadoReto::Cancelado);
}

#[test]
fn el_creador_comenta_a_un_postulado() {
    let c = caso();
    let id = c.retos.publicar_reto(&c.creador, &c.xlm.address, &300, &uri(&c.env));
    let texto = String::from_str(&c.env, "Falta el test del caso de error");
    assert_eq!(c.retos.try_comentar_postulacion(&id, &c.agente, &texto), Err(Ok(Error::NoPostulado)));
    c.retos.postular(&id, &c.agente);
    c.retos.comentar_postulacion(&id, &c.agente, &texto);
    assert_eq!(c.retos.comentario(&id, &c.agente), Some(texto));
}
