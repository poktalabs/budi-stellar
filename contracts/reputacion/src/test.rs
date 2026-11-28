extern crate std;

use super::*;
use insignias::{Insignias, InsigniasClient as Emisor};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{symbol_short, token::StellarAssetClient};

const XLM: i128 = 1_0000000;
const DIA: u64 = 24 * 60 * 60;

struct Caso {
    env: Env,
    humano: Address,
    agente: Address,
    cliente: Address,
    token: token::Client<'static>,
    insignias: Emisor<'static>,
    r: ReputacionClient<'static>,
}

fn soroban() -> Symbol {
    symbol_short!("soroban")
}

/// Piso de 10 XLM por punto, máximo 5 puntos por trabajo, puntos de 1 año,
/// las evaluaciones suman hasta 3 puntos en total y el cliente tiene 3 días para revisar.
fn caso() -> Caso {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);
    let admin = Address::generate(&env);
    let humano = Address::generate(&env);
    let agente = Address::generate(&env);
    let cliente = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &sac.address()).mint(&cliente, &(1_000 * XLM));
    let insignias = Emisor::new(&env, &env.register(Insignias, (admin.clone(),)));
    let id = env.register(
        Reputacion,
        (admin.clone(), sac.address(), insignias.address.clone()),
    );
    let r = ReputacionClient::new(&env, &id);
    r.definir_habilidad(
        &soroban(),
        &Habilidad {
            insignia: symbol_short!("sorob_101"),
            piso: 10 * XLM,
            max_por_trabajo: 5,
            vigencia: 365 * DIA,
            tope_evaluacion: 3,
            revision: 3 * DIA,
        },
    );
    let token = token::Client::new(&env, &sac.address());
    Caso {
        env,
        humano,
        agente,
        cliente,
        token,
        insignias,
        r,
    }
}

/// Caso con el humano ya licenciado (tiene la insignia) y su equipo formado.
fn con_equipo() -> (Caso, u32) {
    let c = caso();
    c.insignias.otorgar(&c.humano, &symbol_short!("sorob_101"));
    let equipo = c.r.formar_equipo(&c.humano, &c.agente);
    (c, equipo)
}

fn prueba(c: &Caso) -> String {
    String::from_str(&c.env, "https://github.com/ana/entrega/pull/1")
}

#[test]
fn trabajo_pagado_y_entregado_suma_puntos_y_paga_al_humano() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &(7 * DIA));
    assert_eq!(c.token.balance(&c.r.address), 30 * XLM);
    c.r.entregar(&t, &prueba(&c));
    assert_eq!(c.r.aceptar(&t), 3);
    assert_eq!(c.token.balance(&c.humano), 30 * XLM);
    assert_eq!(c.token.balance(&c.r.address), 0);
    let p = c.r.puntaje(&equipo, &soroban());
    assert_eq!((p.puntos, p.trabajos, p.por_evaluacion), (3, 1, 0));
    assert_eq!(c.r.trabajo(&t).estado, Estado::Pagado);
}

#[test]
fn sin_insignia_no_se_puede_contratar_en_esa_habilidad() {
    let c = caso();
    let equipo = c.r.formar_equipo(&c.humano, &c.agente);
    assert_eq!(
        c.r.try_contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA),
        Err(Ok(Error::SinInsignia))
    );
}

#[test]
fn bajo_el_piso_se_paga_pero_no_suma_y_cada_trabajo_tiene_tope() {
    let (c, equipo) = con_equipo();
    let barato =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(5 * XLM), &DIA);
    c.r.entregar(&barato, &prueba(&c));
    assert_eq!(c.r.aceptar(&barato), 0);
    assert_eq!(c.token.balance(&c.humano), 5 * XLM);
    let grande =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(500 * XLM), &DIA);
    c.r.entregar(&grande, &prueba(&c));
    assert_eq!(c.r.aceptar(&grande), 5);
    let p = c.r.puntaje(&equipo, &soroban());
    assert_eq!((p.puntos, p.trabajos), (5, 1));
}

#[test]
fn los_puntos_vencen() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    c.r.aceptar(&t);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 2);
    c.env.ledger().with_mut(|l| l.timestamp += 365 * DIA);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 0);
}

#[test]
fn las_evaluaciones_suman_con_tope_total() {
    let (c, equipo) = con_equipo();
    assert_eq!(
        c.r.try_evaluar(&equipo, &soroban(), &4),
        Err(Ok(Error::PuntosInvalidos))
    );
    c.r.evaluar(&equipo, &soroban(), &3);
    c.r.evaluar(&equipo, &soroban(), &3);
    let p = c.r.puntaje(&equipo, &soroban());
    assert_eq!((p.puntos, p.trabajos, p.por_evaluacion), (3, 0, 3));
}

#[test]
fn sin_entrega_el_cliente_recupera_su_pago_despues_del_plazo() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    assert_eq!(c.r.try_reembolsar(&t), Err(Ok(Error::PlazoNoVencido)));
    c.env.ledger().with_mut(|l| l.timestamp += DIA);
    assert_eq!(
        c.r.try_entregar(&t, &prueba(&c)),
        Err(Ok(Error::PlazoVencido))
    );
    c.r.reembolsar(&t);
    assert_eq!(c.token.balance(&c.cliente), 1_000 * XLM);
    assert_eq!(c.r.trabajo(&t).estado, Estado::Reembolsado);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 0);
}

#[test]
fn no_se_acepta_sin_entrega_ni_se_entrega_sin_prueba() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    assert_eq!(c.r.try_aceptar(&t), Err(Ok(Error::EstadoInvalido)));
    assert_eq!(
        c.r.try_entregar(&t, &String::from_str(&c.env, "")),
        Err(Ok(Error::PruebaVacia))
    );
}

#[test]
fn anular_quita_los_puntos_de_ese_trabajo_y_deja_los_demas() {
    let (c, equipo) = con_equipo();
    let a =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&a, &prueba(&c));
    c.r.aceptar(&a);
    let b =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.entregar(&b, &prueba(&c));
    c.r.aceptar(&b);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 5);
    c.r.anular(
        &a,
        &String::from_str(&c.env, "auditoría: la entrega no corre"),
    );
    let p = c.r.puntaje(&equipo, &soroban());
    assert_eq!((p.puntos, p.trabajos), (2, 1));
    assert_eq!(c.r.trabajo(&a).estado, Estado::Anulado);
    assert_eq!(
        c.r.try_anular(&a, &String::from_str(&c.env, "otra vez")),
        Err(Ok(Error::EstadoInvalido))
    );
}

#[test]
fn un_agente_esta_en_un_solo_equipo_y_nadie_se_contrata_a_si_mismo() {
    let (c, equipo) = con_equipo();
    let otro = Address::generate(&c.env);
    assert_eq!(
        c.r.try_formar_equipo(&otro, &c.agente),
        Err(Ok(Error::AgenteYaEnEquipo))
    );
    assert_eq!(
        c.r.try_formar_equipo(&otro, &otro),
        Err(Ok(Error::MismaCuenta))
    );
    assert_eq!(c.r.equipo_de(&c.agente), Some(equipo));
    assert_eq!(
        c.r.try_contratar(&c.humano, &equipo, &soroban(), &(30 * XLM), &DIA),
        Err(Ok(Error::MismaCuenta))
    );
}

#[test]
fn retirar_al_agente_cierra_trabajos_nuevos_y_guarda_el_historial() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    c.r.aceptar(&t);
    c.r.retirar(&equipo);
    assert!(!c.r.equipo(&equipo).activo);
    assert_eq!(
        c.r.try_contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA),
        Err(Ok(Error::EquipoRetirado))
    );
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 3);
    let nuevo_agente = Address::generate(&c.env);
    let nuevo = c.r.formar_equipo(&c.humano, &nuevo_agente);
    assert_eq!(c.r.puntaje(&nuevo, &soroban()).puntos, 0);
}

#[test]
fn reglas_invalidas_no_se_guardan() {
    let c = caso();
    let mala = Habilidad {
        insignia: symbol_short!("sorob_101"),
        piso: 0,
        max_por_trabajo: 5,
        vigencia: DIA,
        tope_evaluacion: 3,
        revision: DIA,
    };
    assert_eq!(
        c.r.try_definir_habilidad(&symbol_short!("rust"), &mala),
        Err(Ok(Error::ReglasInvalidas))
    );
    assert_eq!(
        c.r.try_habilidad(&symbol_short!("rust")),
        Err(Ok(Error::HabilidadNoExiste))
    );
}

#[test]
fn si_le_revocan_la_insignia_a_media_obra_se_paga_pero_no_suma() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    c.insignias
        .revocar(&1, &String::from_str(&c.env, "fraude en la ruta"));
    assert_eq!(c.r.aceptar(&t), 0);
    assert_eq!(c.token.balance(&c.humano), 30 * XLM);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 0);
}

#[test]
fn si_el_cliente_no_responde_el_humano_cobra_al_vencer_la_revision() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    assert_eq!(
        c.r.try_cobrar_sin_respuesta(&t),
        Err(Ok(Error::PlazoNoVencido))
    );
    c.env.ledger().with_mut(|l| l.timestamp += 3 * DIA);
    assert_eq!(c.r.cobrar_sin_respuesta(&t), 3);
    assert_eq!(c.token.balance(&c.humano), 30 * XLM);
    assert_eq!(c.token.balance(&c.r.address), 0);
    assert_eq!(c.r.trabajo(&t).estado, Estado::Pagado);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 3);
    assert_eq!(c.r.try_aceptar(&t), Err(Ok(Error::EstadoInvalido)));
}

#[test]
fn no_se_cobra_sin_entrega_y_el_cliente_puede_aceptar_despues_de_la_revision() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.env.ledger().with_mut(|l| l.timestamp += 10 * DIA);
    assert_eq!(
        c.r.try_cobrar_sin_respuesta(&t),
        Err(Ok(Error::EstadoInvalido))
    );
    let otro =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.entregar(&otro, &prueba(&c));
    assert_eq!(
        c.r.trabajo(&otro).revisar_hasta,
        c.env.ledger().timestamp() + 3 * DIA
    );
    c.env.ledger().with_mut(|l| l.timestamp += 5 * DIA);
    assert_eq!(c.r.aceptar(&otro), 2);
    assert_eq!(
        c.r.try_cobrar_sin_respuesta(&otro),
        Err(Ok(Error::EstadoInvalido))
    );
}

fn motivo(c: &Caso, texto: &str) -> String {
    String::from_str(&c.env, texto)
}

#[test]
fn una_disputa_congela_el_pago_y_budi_le_da_la_razon_al_cliente() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    assert_eq!(
        c.r.try_disputar(&t, &motivo(&c, "")),
        Err(Ok(Error::MotivoVacio))
    );
    c.r.disputar(&t, &motivo(&c, "El contrato no compila"));
    assert_eq!(c.r.trabajo(&t).estado, Estado::Disputado);
    c.env.ledger().with_mut(|l| l.timestamp += 10 * DIA);
    assert_eq!(
        c.r.try_cobrar_sin_respuesta(&t),
        Err(Ok(Error::EstadoInvalido))
    );
    assert_eq!(
        c.r.resolver(&t, &false, &motivo(&c, "Revisamos: no compila")),
        0
    );
    assert_eq!(c.token.balance(&c.cliente), 1_000 * XLM);
    assert_eq!(c.token.balance(&c.humano), 0);
    assert_eq!(c.r.trabajo(&t).estado, Estado::Reembolsado);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 0);
}

#[test]
fn budi_le_da_la_razon_al_equipo_y_el_equipo_suma() {
    let (c, equipo) = con_equipo();
    let t =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    assert_eq!(
        c.r.try_resolver(&t, &true, &motivo(&c, "x")),
        Err(Ok(Error::EstadoInvalido))
    );
    c.r.entregar(&t, &prueba(&c));
    c.r.disputar(&t, &motivo(&c, "No era lo pedido"));
    assert_eq!(c.r.resolver(&t, &true, &motivo(&c, "Cumple lo pedido")), 3);
    assert_eq!(c.token.balance(&c.humano), 30 * XLM);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 3);
    assert_eq!(
        c.r.try_resolver(&t, &false, &motivo(&c, "otra vez")),
        Err(Ok(Error::EstadoInvalido))
    );
}

#[test]
fn no_se_disputa_despues_de_la_revision_y_el_cliente_puede_retirar_la_disputa() {
    let (c, equipo) = con_equipo();
    let a =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(30 * XLM), &DIA);
    c.r.entregar(&a, &prueba(&c));
    c.r.disputar(&a, &motivo(&c, "Falta la prueba de carga"));
    assert_eq!(c.r.aceptar(&a), 3);
    let b =
        c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.entregar(&b, &prueba(&c));
    c.env.ledger().with_mut(|l| l.timestamp += 3 * DIA);
    assert_eq!(
        c.r.try_disputar(&b, &motivo(&c, "tarde")),
        Err(Ok(Error::PlazoVencido))
    );
}

#[test]
fn el_cliente_aumenta_la_custodia_antes_de_la_entrega() {
    let (c, equipo) = con_equipo();
    let t = c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    assert_eq!(c.r.aumentar(&t, &(10 * XLM)), 30 * XLM);
    assert_eq!(c.token.balance(&c.r.address), 30 * XLM);
    assert_eq!(c.r.try_aumentar(&t, &0), Err(Ok(Error::MontoInvalido)));
    c.r.entregar(&t, &prueba(&c));
    assert_eq!(c.r.try_aumentar(&t, &XLM), Err(Ok(Error::EstadoInvalido)));
    // Al aceptar, el humano cobra el monto aumentado y suma por él (30 XLM / piso 10 = 3).
    assert_eq!(c.r.aceptar(&t), 3);
    assert_eq!(c.token.balance(&c.humano), 30 * XLM);
}

#[test]
fn no_se_aumenta_despues_del_plazo() {
    let (c, equipo) = con_equipo();
    let t = c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.env.ledger().set_timestamp(1_000_000 + DIA);
    assert_eq!(c.r.try_aumentar(&t, &XLM), Err(Ok(Error::PlazoVencido)));
}

#[test]
fn el_equipo_declina_y_el_cliente_recupera_su_pago_al_momento() {
    let (c, equipo) = con_equipo();
    let antes = c.token.balance(&c.cliente);
    let t = c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.declinar(&t);
    assert_eq!(c.token.balance(&c.cliente), antes);
    assert_eq!(c.r.trabajo(&t).estado, Estado::Reembolsado);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, 0);
    assert_eq!(c.r.try_declinar(&t), Err(Ok(Error::EstadoInvalido)));
}

#[test]
fn no_se_declina_lo_ya_entregado() {
    let (c, equipo) = con_equipo();
    let t = c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    c.r.entregar(&t, &prueba(&c));
    assert_eq!(c.r.try_declinar(&t), Err(Ok(Error::EstadoInvalido)));
}

#[test]
fn la_propina_va_al_humano_y_no_suma_puntos() {
    let (c, equipo) = con_equipo();
    let t = c.r.contratar(&c.cliente, &equipo, &soroban(), &(20 * XLM), &DIA);
    assert_eq!(c.r.try_propina(&t, &XLM), Err(Ok(Error::EstadoInvalido)));
    c.r.entregar(&t, &prueba(&c));
    c.r.aceptar(&t);
    let puntos = c.r.puntaje(&equipo, &soroban()).puntos;
    c.r.propina(&t, &(5 * XLM));
    assert_eq!(c.token.balance(&c.humano), 25 * XLM);
    assert_eq!(c.r.puntaje(&equipo, &soroban()).puntos, puntos);
    assert_eq!(c.r.try_propina(&t, &0), Err(Ok(Error::MontoInvalido)));
}
