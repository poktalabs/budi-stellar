extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _};
use soroban_sdk::{symbol_short, vec, Event, IntoVal};

fn setup() -> (Env, BitacoraClient<'static>) {
    let env = Env::default();
    let id = env.register(Bitacora, ());
    let client = BitacoraClient::new(&env, &id);
    (env, client)
}

#[test]
fn registra_y_lee_hitos_en_orden() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);

    assert_eq!(client.contar(&builder), 0);
    assert_eq!(
        client.registrar(&builder, &String::from_str(&env, "Repo público")),
        1
    );
    assert_eq!(
        client.registrar(&builder, &String::from_str(&env, "Contrato con pruebas")),
        2
    );

    assert_eq!(
        client.hitos(&builder),
        vec![
            &env,
            String::from_str(&env, "Repo público"),
            String::from_str(&env, "Contrato con pruebas")
        ]
    );
    assert_eq!(client.contar(&builder), 2);
}

#[test]
fn pide_la_firma_del_builder() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Despegue");

    client.registrar(&builder, &hito);

    assert_eq!(
        env.auths(),
        std::vec![(
            builder.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    symbol_short!("registrar"),
                    (builder.clone(), hito.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

#[test]
#[should_panic]
fn sin_firma_no_registra() {
    let (env, client) = setup();
    let builder = Address::generate(&env);
    client.registrar(&builder, &String::from_str(&env, "Sin firma"));
}

#[test]
fn rechaza_hito_vacio_o_largo() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);

    assert_eq!(
        client.try_registrar(&builder, &String::from_str(&env, "")),
        Err(Ok(Error::HitoVacio))
    );
    let largo = "x".repeat(MAX_HITO as usize + 1);
    assert_eq!(
        client.try_registrar(&builder, &String::from_str(&env, &largo)),
        Err(Ok(Error::HitoMuyLargo))
    );
    assert_eq!(client.contar(&builder), 0);
}

#[test]
fn la_bitacora_tiene_tope() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "hito");
    for _ in 0..MAX_HITOS {
        client.registrar(&builder, &hito);
    }
    assert_eq!(
        client.try_registrar(&builder, &hito),
        Err(Ok(Error::BitacoraLlena))
    );
}

#[test]
fn cada_builder_tiene_su_bitacora() {
    let (env, client) = setup();
    env.mock_all_auths();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);

    client.registrar(&ana, &String::from_str(&env, "Hito de Ana"));
    assert_eq!(client.contar(&ana), 1);
    assert_eq!(client.contar(&beto), 0);
}

#[test]
fn publica_un_evento_por_hito() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Demo");

    client.registrar(&builder, &hito);

    let esperado = HitoRegistrado {
        builder: builder.clone(),
        numero: 1,
        hito,
    };
    assert_eq!(
        env.events().all(),
        std::vec![esperado.to_xdr(&env, &client.address)]
    );
}

#[test]
fn corrige_un_hito_con_la_firma_del_builder() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    client.registrar(&builder, &String::from_str(&env, "Primer contrato"));
    client.registrar(&builder, &String::from_str(&env, "Pruebas en verde"));
    let nuevo = String::from_str(&env, "Mi primer contrato en Stellar");

    client.corregir(&builder, &1, &nuevo);

    assert_eq!(
        env.auths(),
        std::vec![(
            builder.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "corregir"),
                    (builder.clone(), 1u32, nuevo.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
    assert_eq!(
        client.hitos(&builder),
        vec![&env, nuevo, String::from_str(&env, "Pruebas en verde")]
    );
    assert_eq!(client.contar(&builder), 2);
}

#[test]
#[should_panic]
fn sin_firma_no_corrige() {
    let (env, client) = setup();
    let builder = Address::generate(&env);
    env.mock_all_auths();
    client.registrar(&builder, &String::from_str(&env, "Hito"));
    env.set_auths(&[]);
    client.corregir(&builder, &1, &String::from_str(&env, "Otro"));
}

#[test]
fn no_corrige_un_hito_que_no_existe_ni_con_texto_invalido() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let hito = String::from_str(&env, "Hito");
    assert_eq!(client.try_corregir(&builder, &1, &hito), Err(Ok(Error::HitoInexistente)));
    client.registrar(&builder, &hito);
    assert_eq!(client.try_corregir(&builder, &0, &hito), Err(Ok(Error::HitoInexistente)));
    assert_eq!(client.try_corregir(&builder, &2, &hito), Err(Ok(Error::HitoInexistente)));
    assert_eq!(
        client.try_corregir(&builder, &1, &String::from_str(&env, "")),
        Err(Ok(Error::HitoVacio))
    );
    let largo = "x".repeat(MAX_HITO as usize + 1);
    assert_eq!(
        client.try_corregir(&builder, &1, &String::from_str(&env, &largo)),
        Err(Ok(Error::HitoMuyLargo))
    );
    assert_eq!(client.hitos(&builder), vec![&env, hito]);
}

#[test]
fn publica_la_correccion_con_el_texto_de_antes() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    let antes = String::from_str(&env, "Demo");
    let ahora = String::from_str(&env, "Demo en testnet");
    client.registrar(&builder, &antes);

    client.corregir(&builder, &1, &ahora);

    let esperado = HitoCorregido { builder: builder.clone(), numero: 1, antes, ahora };
    assert_eq!(env.events().all(), std::vec![esperado.to_xdr(&env, &client.address)]);
}

mod con_matricula {
    use super::*;
    use matricula::{Matricula, MatriculaClient as Escrow};
    use soroban_sdk::token::StellarAssetClient;

    fn caso() -> (Env, BitacoraClient<'static>, Escrow<'static>, Address) {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let alumno = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
        StellarAssetClient::new(&env, &sac.address()).mint(&alumno, &1_000);
        let escrow = Escrow::new(&env, &env.register(Matricula, (admin.clone(), sac.address())));
        escrow.crear_curso(&symbol_short!("soroban"), &100, &3600);
        client.configurar(&admin, &escrow.address, &symbol_short!("soroban"));
        (env, client, escrow, alumno)
    }

    #[test]
    fn sin_pagar_no_registra() {
        let (env, client, _, alumno) = caso();
        assert_eq!(
            client.try_registrar(&alumno, &String::from_str(&env, "Hola")),
            Err(Ok(Error::NoInscrito))
        );
    }

    #[test]
    fn con_el_curso_pagado_registra() {
        let (env, client, escrow, alumno) = caso();
        escrow.inscribir(&alumno, &symbol_short!("soroban"));
        assert_eq!(client.registrar(&alumno, &String::from_str(&env, "Pagué")), 1);
    }

    #[test]
    fn se_configura_una_sola_vez() {
        let (env, client, escrow, _) = caso();
        let otro = Address::generate(&env);
        assert_eq!(
            client.try_configurar(&otro, &escrow.address, &symbol_short!("rust")),
            Err(Ok(Error::YaConfigurada))
        );
    }
}

#[test]
fn borra_un_hito_y_los_siguientes_recorren_su_numero() {
    let (env, client) = setup();
    env.mock_all_auths();
    let builder = Address::generate(&env);
    for h in ["uno", "dos", "tres"] {
        client.registrar(&builder, &String::from_str(&env, h));
    }
    client.borrar(&builder, &2);
    assert_eq!(
        client.hitos(&builder),
        vec![&env, String::from_str(&env, "uno"), String::from_str(&env, "tres")]
    );
    assert_eq!(client.try_borrar(&builder, &3), Err(Ok(Error::HitoInexistente)));
    assert_eq!(client.try_borrar(&builder, &0), Err(Ok(Error::HitoInexistente)));
}

#[test]
#[should_panic]
fn sin_firma_no_borra() {
    let (env, client) = setup();
    let builder = Address::generate(&env);
    env.mock_all_auths();
    client.registrar(&builder, &String::from_str(&env, "Hito"));
    env.set_auths(&[]);
    client.borrar(&builder, &1);
}

#[test]
fn fijar_hito_y_borrar_lo_ajusta() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    client.registrar(&b, &String::from_str(&env, "Uno"));
    client.registrar(&b, &String::from_str(&env, "Dos"));
    client.registrar(&b, &String::from_str(&env, "Tres"));
    assert_eq!(client.fijado(&b), None);
    client.fijar_hito(&b, &3);
    assert_eq!(client.fijado(&b), Some(3));
    client.borrar(&b, &1);
    assert_eq!(client.fijado(&b), Some(2));
    client.borrar(&b, &2);
    assert_eq!(client.fijado(&b), None);
    assert_eq!(client.try_fijar_hito(&b, &5), Err(Ok(Error::HitoInexistente)));
}

#[test]
fn desfijar_y_vaciar_la_bitacora() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    assert_eq!(client.try_desfijar_hito(&b), Err(Ok(Error::SinHitoFijado)));
    assert_eq!(client.try_vaciar_bitacora(&b), Err(Ok(Error::BitacoraVacia)));
    client.registrar(&b, &String::from_str(&env, "Uno"));
    client.registrar(&b, &String::from_str(&env, "Dos"));
    client.fijar_hito(&b, &2);
    assert_eq!(client.desfijar_hito(&b), 2);
    assert_eq!(client.fijado(&b), None);
    client.fijar_hito(&b, &1);
    assert_eq!(client.vaciar_bitacora(&b), 2);
    assert_eq!(client.contar(&b), 0);
    assert_eq!(client.fijado(&b), None);
    assert_eq!(client.registrar(&b, &String::from_str(&env, "De nuevo")), 1);
}

#[test]
#[should_panic]
fn sin_firma_no_vacia() {
    let (env, client) = setup();
    let b = Address::generate(&env);
    env.mock_all_auths();
    client.registrar(&b, &String::from_str(&env, "Hito"));
    env.set_auths(&[]);
    client.vaciar_bitacora(&b);
}

#[test]
fn registra_un_lote_de_hitos_en_orden() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    let lote = vec![&env, String::from_str(&env, "Uno"), String::from_str(&env, "Dos")];
    assert_eq!(client.registrar_lote(&b, &lote), 2);
    assert_eq!(client.hitos(&b), lote);
    assert_eq!(client.try_registrar_lote(&b, &Vec::new(&env)), Err(Ok(Error::LoteVacio)));
    let malo = vec![&env, String::from_str(&env, "Tres"), String::from_str(&env, "")];
    assert_eq!(client.try_registrar_lote(&b, &malo), Err(Ok(Error::HitoVacio)));
    assert_eq!(client.contar(&b), 2);
}

#[test]
fn mover_hito_reordena_y_conserva_el_fijado() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    let lote = vec![&env, String::from_str(&env, "A"), String::from_str(&env, "B"), String::from_str(&env, "C")];
    client.registrar_lote(&b, &lote);
    client.fijar_hito(&b, &2);
    client.mover_hito(&b, &3, &1);
    assert_eq!(
        client.hitos(&b),
        vec![&env, String::from_str(&env, "C"), String::from_str(&env, "A"), String::from_str(&env, "B")]
    );
    assert_eq!(client.fijado(&b), Some(3));
    client.mover_hito(&b, &3, &1);
    assert_eq!(client.fijado(&b), Some(1));
    assert_eq!(client.try_mover_hito(&b, &0, &1), Err(Ok(Error::HitoInexistente)));
    assert_eq!(client.try_mover_hito(&b, &1, &4), Err(Ok(Error::HitoInexistente)));
}

#[test]
fn el_agente_vinculado_anota_y_otro_no() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    let agente = Address::generate(&env);
    let intruso = Address::generate(&env);
    let h = String::from_str(&env, "Contrato desplegado por mi agente");
    assert_eq!(client.try_registrar_por_agente(&agente, &b, &h), Err(Ok(Error::AgenteNoVinculado)));
    assert_eq!(client.try_vincular_agente(&b, &b), Err(Ok(Error::MismaCuenta)));
    client.vincular_agente(&b, &agente);
    assert_eq!(client.agente_de(&b), Some(agente.clone()));
    assert_eq!(client.registrar_por_agente(&agente, &b, &h), 1);
    assert_eq!(client.try_registrar_por_agente(&intruso, &b, &h), Err(Ok(Error::AgenteNoVinculado)));
    assert_eq!(client.desvincular_agente(&b), agente);
    assert_eq!(client.try_registrar_por_agente(&agente, &b, &h), Err(Ok(Error::AgenteNoVinculado)));
    assert_eq!(client.try_desvincular_agente(&b), Err(Ok(Error::SinAgente)));
}

#[test]
fn registrar_por_agente_pide_la_firma_del_agente() {
    let (env, client) = setup();
    let b = Address::generate(&env);
    let agente = Address::generate(&env);
    env.mock_all_auths();
    client.vincular_agente(&b, &agente);
    env.set_auths(&[]);
    assert!(client.try_registrar_por_agente(&agente, &b, &String::from_str(&env, "x")).is_err());
}

#[test]
fn la_bitacora_pasa_a_otra_cuenta_en_dos_pasos() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    let nuevo = Address::generate(&env);
    let otro = Address::generate(&env);
    assert_eq!(client.try_ofrecer_bitacora(&b, &nuevo), Err(Ok(Error::BitacoraVacia)));
    client.registrar_lote(&b, &vec![&env, String::from_str(&env, "A"), String::from_str(&env, "B")]);
    client.fijar_hito(&b, &2);
    client.vincular_agente(&b, &otro);
    assert_eq!(client.try_ofrecer_bitacora(&b, &b), Err(Ok(Error::MismaCuenta)));
    assert_eq!(client.try_aceptar_bitacora(&nuevo, &b), Err(Ok(Error::SinOferta)));
    client.ofrecer_bitacora(&b, &nuevo);
    assert_eq!(client.try_aceptar_bitacora(&otro, &b), Err(Ok(Error::SinOferta)));
    assert_eq!(client.aceptar_bitacora(&nuevo, &b), 2);
    assert_eq!(client.contar(&nuevo), 2);
    assert_eq!(client.fijado(&nuevo), Some(2));
    assert_eq!(client.contar(&b), 0);
    assert_eq!(client.agente_de(&b), None);
    assert_eq!(client.try_aceptar_bitacora(&nuevo, &b), Err(Ok(Error::SinOferta)));
}

#[test]
fn no_acepta_si_el_destino_ya_tiene_hitos() {
    let (env, client) = setup();
    env.mock_all_auths();
    let b = Address::generate(&env);
    let nuevo = Address::generate(&env);
    client.registrar(&b, &String::from_str(&env, "A"));
    client.registrar(&nuevo, &String::from_str(&env, "Propio"));
    client.ofrecer_bitacora(&b, &nuevo);
    assert_eq!(client.try_aceptar_bitacora(&nuevo, &b), Err(Ok(Error::DestinoConHitos)));
}

#[test]
fn el_admin_cambia_y_quita_el_acceso() {
    let (env, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    assert_eq!(client.try_cambiar_acceso(&m1, &symbol_short!("c1")), Err(Ok(Error::SinConfigurar)));
    assert_eq!(client.try_quitar_acceso(), Err(Ok(Error::SinConfigurar)));
    client.configurar(&admin, &m1, &symbol_short!("c1"));
    client.cambiar_acceso(&m2, &symbol_short!("c2"));
    assert_eq!(client.acceso(), Some(Acceso { matricula: m2.clone(), curso: symbol_short!("c2") }));
    client.quitar_acceso();
    assert_eq!(client.acceso(), None);
    assert_eq!(client.try_configurar(&admin, &m1, &symbol_short!("c1")), Err(Ok(Error::YaConfigurada)));
    let b = Address::generate(&env);
    assert_eq!(client.registrar(&b, &String::from_str(&env, "Abierto")), 1);
    client.cambiar_acceso(&m1, &symbol_short!("c1"));
    assert_eq!(client.acceso().unwrap().matricula, m1);
}

#[test]
fn cambiar_acceso_pide_la_firma_del_admin() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let m = Address::generate(&env);
    env.mock_all_auths();
    client.configurar(&admin, &m, &symbol_short!("c1"));
    env.set_auths(&[]);
    assert!(client.try_cambiar_acceso(&m, &symbol_short!("c2")).is_err());
    assert!(client.try_quitar_acceso().is_err());
}
