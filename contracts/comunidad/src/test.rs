extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;

fn s(env: &Env, t: &str) -> String {
    String::from_str(env, t)
}

fn setup() -> (Env, ComunidadClient<'static>) {
    let env = Env::default();
    let id = env.register(Comunidad, ());
    let client = ComunidadClient::new(&env, &id);
    (env, client)
}

#[test]
fn grupo_privado_invitar_unirse_salir_expulsar_transferir_y_cerrar() {
    let (env, c) = setup();
    env.mock_all_auths();
    let (d, ana, beto) = (Address::generate(&env), Address::generate(&env), Address::generate(&env));
    assert_eq!(c.try_crear_grupo_estudio(&d, &s(&env, ""), &true), Err(Ok(Error::NombreVacio)));
    let g = c.crear_grupo_estudio(&d, &s(&env, "Soroban desde cero"), &true);
    assert_eq!(g, 1);
    assert_eq!(c.try_unirse_grupo_estudio(&ana, &g), Err(Ok(Error::SinInvitacion)));
    assert_eq!(c.try_invitar_a_grupo(&ana, &g, &beto), Err(Ok(Error::NoEsDueno)));
    c.invitar_a_grupo(&d, &g, &ana);
    c.invitar_a_grupo(&d, &g, &beto);
    c.unirse_grupo_estudio(&ana, &g);
    c.unirse_grupo_estudio(&beto, &g);
    assert_eq!(c.try_unirse_grupo_estudio(&ana, &g), Err(Ok(Error::YaEsMiembro)));
    assert_eq!(c.grupo(&g).unwrap().miembros.len(), 3);
    assert_eq!(c.try_salir_grupo_estudio(&d, &g), Err(Ok(Error::DuenoNoSale)));
    c.salir_grupo_estudio(&beto, &g);
    assert_eq!(c.try_salir_grupo_estudio(&beto, &g), Err(Ok(Error::NoEsMiembro)));
    c.expulsar_de_grupo(&d, &g, &ana);
    assert_eq!(c.grupo(&g).unwrap().miembros.len(), 1);
    assert_eq!(c.try_transferir_grupo(&d, &g, &ana), Err(Ok(Error::NoEsMiembro)));
    c.invitar_a_grupo(&d, &g, &ana);
    c.unirse_grupo_estudio(&ana, &g);
    c.transferir_grupo(&d, &g, &ana);
    assert_eq!(c.grupo(&g).unwrap().dueno, ana);
    assert_eq!(c.try_cerrar_grupo_estudio(&d, &g), Err(Ok(Error::NoEsDueno)));
    c.cerrar_grupo_estudio(&ana, &g);
    assert_eq!(c.try_unirse_grupo_estudio(&beto, &g), Err(Ok(Error::GrupoCerrado)));
    assert_eq!(c.try_hacer_pregunta(&ana, &g, &s(&env, "¿Sigue abierto?")), Err(Ok(Error::GrupoCerrado)));
}

#[test]
fn foro_preguntar_responder_aceptar_votar_fijar_reportar_y_ocultar() {
    let (env, c) = setup();
    env.mock_all_auths();
    let (d, ana, beto, fuera) = (Address::generate(&env), Address::generate(&env), Address::generate(&env), Address::generate(&env));
    let g = c.crear_grupo_estudio(&d, &s(&env, "IA y Stellar"), &false);
    c.unirse_grupo_estudio(&ana, &g);
    c.unirse_grupo_estudio(&beto, &g);
    assert_eq!(c.try_hacer_pregunta(&fuera, &g, &s(&env, "Hola")), Err(Ok(Error::NoEsMiembro)));
    assert_eq!(c.try_hacer_pregunta(&ana, &g, &s(&env, "")), Err(Ok(Error::TextoVacio)));
    let p = c.hacer_pregunta(&ana, &g, &s(&env, "¿Qué es require_auth?"));
    let r = c.responder_pregunta(&beto, &p, &s(&env, "Pide la firma de esa cuenta."));
    assert_eq!(c.try_aceptar_respuesta(&beto, &p, &r), Err(Ok(Error::NoEsAutor)));
    c.aceptar_respuesta(&ana, &p, &r);
    assert_eq!(c.pregunta(&p).unwrap().aceptada, Some(r));
    assert_eq!(c.try_votar_respuesta(&beto, &r), Err(Ok(Error::VotoPropio)));
    assert_eq!(c.votar_respuesta(&ana, &r), 1);
    assert_eq!(c.try_votar_respuesta(&ana, &r), Err(Ok(Error::YaVoto)));
    assert_eq!(c.votar_respuesta(&d, &r), 2);
    assert_eq!(c.try_fijar_pregunta(&ana, &p, &true), Err(Ok(Error::NoEsModerador)));
    c.fijar_pregunta(&d, &p, &true);
    assert!(c.pregunta(&p).unwrap().fijada);
    assert_eq!(c.reportar_pregunta(&beto, &p), 1);
    assert_eq!(c.try_reportar_pregunta(&beto, &p), Err(Ok(Error::YaReportada)));
    c.ocultar_pregunta(&d, &p, &true);
    assert_eq!(c.try_responder_pregunta(&d, &p, &s(&env, "Otra")), Err(Ok(Error::PreguntaOculta)));
    c.ocultar_pregunta(&d, &p, &false);
    let otra = c.hacer_pregunta(&beto, &g, &s(&env, "¿Y los eventos?"));
    assert_eq!(c.try_aceptar_respuesta(&beto, &otra, &r), Err(Ok(Error::RespuestaAjena)));
}

#[test]
#[should_panic]
fn sin_firma_no_crea_grupo() {
    let (env, c) = setup();
    let d = Address::generate(&env);
    c.crear_grupo_estudio(&d, &s(&env, "Sin firma"), &false);
}

#[test]
#[should_panic]
fn sin_firma_no_pregunta() {
    let (env, c) = setup();
    let d = Address::generate(&env);
    env.mock_all_auths();
    let g = c.crear_grupo_estudio(&d, &s(&env, "G"), &false);
    env.set_auths(&[]);
    c.hacer_pregunta(&d, &g, &s(&env, "Sin firma"));
}

#[test]
fn moderadores_renombrar_reabrir_editar_quitar_voto_y_borrar() {
    let (env, c) = setup();
    env.mock_all_auths();
    let (d, ana, beto) = (Address::generate(&env), Address::generate(&env), Address::generate(&env));
    let g = c.crear_grupo_estudio(&d, &s(&env, "G"), &false);
    c.unirse_grupo_estudio(&ana, &g);
    c.unirse_grupo_estudio(&beto, &g);
    let p = c.hacer_pregunta(&ana, &g, &s(&env, "¿Qué es un SAC?"));
    let r = c.responder_pregunta(&beto, &p, &s(&env, "Un token"));
    assert_eq!(c.try_ocultar_pregunta(&beto, &p, &true), Err(Ok(Error::NoEsModerador)));
    c.nombrar_moderador_grupo(&d, &g, &beto);
    assert!(c.es_moderador_grupo(&g, &beto));
    assert_eq!(c.try_nombrar_moderador_grupo(&d, &g, &beto), Err(Ok(Error::YaEsModerador)));
    c.ocultar_pregunta(&beto, &p, &true);
    c.ocultar_pregunta(&beto, &p, &false);
    c.quitar_moderador_grupo(&d, &g, &beto);
    assert_eq!(c.try_quitar_moderador_grupo(&d, &g, &beto), Err(Ok(Error::NoEsModerador)));
    c.editar_pregunta(&ana, &p, &s(&env, "¿Qué es un Stellar Asset Contract?"));
    assert_eq!(c.try_editar_pregunta(&beto, &p, &s(&env, "x")), Err(Ok(Error::NoEsAutor)));
    c.editar_respuesta(&beto, &r, &s(&env, "El contrato de un activo de Stellar"));
    assert_eq!(c.respuesta(&r).unwrap().texto, s(&env, "El contrato de un activo de Stellar"));
    assert_eq!(c.votar_respuesta(&ana, &r), 1);
    assert_eq!(c.quitar_voto_respuesta(&ana, &r), 0);
    assert_eq!(c.try_quitar_voto_respuesta(&ana, &r), Err(Ok(Error::SinVoto)));
    c.aceptar_respuesta(&ana, &p, &r);
    assert_eq!(c.try_editar_pregunta(&ana, &p, &s(&env, "y")), Err(Ok(Error::YaAceptada)));
    assert_eq!(c.try_borrar_respuesta_foro(&ana, &r), Err(Ok(Error::NoEsModerador)));
    c.borrar_respuesta_foro(&d, &r);
    assert_eq!(c.respuesta(&r), None);
    assert_eq!(c.pregunta(&p).unwrap().aceptada, None);
    c.renombrar_grupo_estudio(&d, &g, &s(&env, "G2"));
    assert_eq!(c.grupo(&g).unwrap().nombre, s(&env, "G2"));
    assert_eq!(c.try_reabrir_grupo_estudio(&d, &g), Err(Ok(Error::GrupoAbierto)));
    c.cerrar_grupo_estudio(&d, &g);
    c.reabrir_grupo_estudio(&d, &g);
    assert!(!c.grupo(&g).unwrap().cerrado);
}

#[test]
fn describir_privacidad_invitaciones_desaceptar_reportes_y_borrar_pregunta() {
    let (env, c) = setup();
    env.mock_all_auths();
    let (d, ana, beto) = (Address::generate(&env), Address::generate(&env), Address::generate(&env));
    let g = c.crear_grupo_estudio(&d, &s(&env, "G"), &false);
    c.describir_grupo_estudio(&d, &g, &s(&env, "Martes 7 pm, nivel inicial"));
    assert_eq!(c.descripcion_grupo(&g), Some(s(&env, "Martes 7 pm, nivel inicial")));
    assert_eq!(c.try_describir_grupo_estudio(&ana, &g, &s(&env, "x")), Err(Ok(Error::NoEsDueno)));
    c.cambiar_privacidad_grupo(&d, &g, &true);
    assert_eq!(c.try_unirse_grupo_estudio(&ana, &g), Err(Ok(Error::SinInvitacion)));
    c.invitar_a_grupo(&d, &g, &ana);
    c.cancelar_invitacion_grupo(&d, &g, &ana);
    assert_eq!(c.try_unirse_grupo_estudio(&ana, &g), Err(Ok(Error::SinInvitacion)));
    assert_eq!(c.try_cancelar_invitacion_grupo(&d, &g, &ana), Err(Ok(Error::SinInvitacion)));
    c.invitar_a_grupo(&d, &g, &beto);
    c.rechazar_invitacion_grupo(&beto, &g);
    assert_eq!(c.try_rechazar_invitacion_grupo(&beto, &g), Err(Ok(Error::SinInvitacion)));
    c.invitar_a_grupo(&d, &g, &ana);
    c.unirse_grupo_estudio(&ana, &g);
    let p = c.hacer_pregunta(&ana, &g, &s(&env, "¿Testnet o mainnet?"));
    let r = c.responder_pregunta(&d, &p, &s(&env, "Primero testnet"));
    assert_eq!(c.try_desaceptar_respuesta(&ana, &p), Err(Ok(Error::SinAceptada)));
    c.aceptar_respuesta(&ana, &p, &r);
    c.desaceptar_respuesta(&ana, &p);
    assert_eq!(c.pregunta(&p).unwrap().aceptada, None);
    assert_eq!(c.reportar_pregunta(&ana, &p), 1);
    assert_eq!(c.retirar_reporte_pregunta(&ana, &p), 0);
    assert_eq!(c.try_retirar_reporte_pregunta(&ana, &p), Err(Ok(Error::SinReporte)));
    c.borrar_pregunta_foro(&ana, &p);
    assert_eq!(c.pregunta(&p), None);
    let p2 = c.hacer_pregunta(&ana, &g, &s(&env, "Otra"));
    assert_eq!(c.try_borrar_pregunta_foro(&beto, &p2), Err(Ok(Error::NoEsModerador)));
    c.borrar_pregunta_foro(&d, &p2);
    assert_eq!(c.pregunta(&p2), None);
}

#[test]
fn quitar_descripcion_mostrar_y_desfijar() {
    let (env, c) = setup();
    env.mock_all_auths();
    let (d, ana) = (Address::generate(&env), Address::generate(&env));
    let g = c.crear_grupo_estudio(&d, &s(&env, "G"), &false);
    c.describir_grupo_estudio(&d, &g, &s(&env, "Martes"));
    c.quitar_descripcion_grupo(&d, &g);
    assert_eq!(c.descripcion_grupo(&g), None);
    c.unirse_grupo_estudio(&ana, &g);
    let p = c.hacer_pregunta(&ana, &g, &s(&env, "Hola"));
    assert_eq!(c.try_mostrar_pregunta(&ana, &p), Err(Ok(Error::NoEsModerador)));
    assert_eq!(c.try_desfijar_pregunta(&ana, &p), Err(Ok(Error::NoEsModerador)));
    c.mostrar_pregunta(&d, &p);
    c.desfijar_pregunta(&d, &p);
    let q = c.pregunta(&p).unwrap();
    assert!(!q.oculta && !q.fijada);
}
