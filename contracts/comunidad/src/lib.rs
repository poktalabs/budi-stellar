//! Comunidad: grupos de estudio y foro de preguntas para quienes aprenden IA y
//! blockchain con Budi. Un grupo tiene dueño y miembros (abierto o solo con
//! invitación); los miembros preguntan, responden, votan y reportan, y el autor de
//! la pregunta acepta la respuesta que le sirvió. El dueño modera.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address,
    Env, String, Symbol, Vec,
};

/// Largo máximo de una pregunta o respuesta (en bytes).
pub const MAX_TEXTO: u32 = 280;

const TTL_UMBRAL: u32 = 17_280;
const TTL_EXTENDER: u32 = 518_400;

#[contracttype]
enum Clave {
    NGrupos,
    Grupo(u32),
    Invitado(u32, Address),
    NPreguntas,
    Pregunta(u32),
    NRespuestas,
    Respuesta(u32),
    Voto(u32, Address),
    Reporte(u32, Address),
    Moderador(u32, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grupo {
    pub dueno: Address,
    pub nombre: String,
    pub privado: bool,
    pub cerrado: bool,
    pub miembros: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pregunta {
    pub grupo: u32,
    pub autor: Address,
    pub texto: String,
    pub aceptada: Option<u32>,
    pub fijada: bool,
    pub oculta: bool,
    pub reportes: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Respuesta {
    pub pregunta: u32,
    pub autor: Address,
    pub texto: String,
    pub votos: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    GrupoInexistente = 1,
    NoEsDueno = 2,
    GrupoCerrado = 3,
    YaEsMiembro = 4,
    NoEsMiembro = 5,
    SinInvitacion = 6,
    NombreVacio = 7,
    TextoVacio = 8,
    TextoMuyLargo = 9,
    PreguntaInexistente = 10,
    RespuestaInexistente = 11,
    YaVoto = 12,
    VotoPropio = 13,
    NoEsAutor = 14,
    YaReportada = 15,
    PreguntaOculta = 16,
    DuenoNoSale = 17,
    RespuestaAjena = 18,
    SinVoto = 19,
    GrupoAbierto = 20,
    YaEsModerador = 21,
    NoEsModerador = 22,
    YaAceptada = 23,
}

/// Algo cambió en un grupo: `accion` dice qué (crear, invitar, unirse, salir,
/// expulsar, transferir, cerrar).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrupoCambio {
    #[topic]
    pub grupo: u32,
    #[topic]
    pub quien: Address,
    pub accion: Symbol,
}

/// Algo pasó en el foro: `accion` dice qué (preguntar, responder, aceptar, votar,
/// fijar, reportar, ocultar); `numero` es la pregunta o la respuesta.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForoCambio {
    #[topic]
    pub numero: u32,
    #[topic]
    pub quien: Address,
    pub accion: Symbol,
}

#[contract]
pub struct Comunidad;

fn guardar<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, clave: &Clave, v: &V) {
    env.storage().persistent().set(clave, v);
    env.storage().persistent().extend_ttl(clave, TTL_UMBRAL, TTL_EXTENDER);
}

fn siguiente(env: &Env, clave: &Clave) -> u32 {
    let n: u32 = env.storage().instance().get(clave).unwrap_or(0) + 1;
    env.storage().instance().set(clave, &n);
    env.storage().instance().extend_ttl(TTL_UMBRAL, TTL_EXTENDER);
    n
}

fn revisar_texto(texto: &String) -> Result<(), Error> {
    if texto.is_empty() {
        return Err(Error::TextoVacio);
    }
    if texto.len() > MAX_TEXTO {
        return Err(Error::TextoMuyLargo);
    }
    Ok(())
}

impl Comunidad {
    fn cargar_grupo(env: &Env, grupo: u32) -> Result<Grupo, Error> {
        env.storage().persistent().get(&Clave::Grupo(grupo)).ok_or(Error::GrupoInexistente)
    }

    fn grupo_del_dueno(env: &Env, dueno: &Address, grupo: u32) -> Result<Grupo, Error> {
        dueno.require_auth();
        let g = Self::cargar_grupo(env, grupo)?;
        if g.dueno != *dueno {
            return Err(Error::NoEsDueno);
        }
        Ok(g)
    }

    fn miembro_abierto(env: &Env, quien: &Address, grupo: u32) -> Result<Grupo, Error> {
        let g = Self::cargar_grupo(env, grupo)?;
        if g.cerrado {
            return Err(Error::GrupoCerrado);
        }
        if !g.miembros.contains(quien) {
            return Err(Error::NoEsMiembro);
        }
        Ok(g)
    }

    fn puede_moderar(env: &Env, quien: &Address, grupo: u32) -> Result<Grupo, Error> {
        quien.require_auth();
        let g = Self::cargar_grupo(env, grupo)?;
        if g.dueno != *quien && !env.storage().persistent().has(&Clave::Moderador(grupo, quien.clone())) {
            return Err(Error::NoEsModerador);
        }
        Ok(g)
    }

    fn cargar_pregunta(env: &Env, pregunta: u32) -> Result<Pregunta, Error> {
        env.storage().persistent().get(&Clave::Pregunta(pregunta)).ok_or(Error::PreguntaInexistente)
    }

    fn cargar_respuesta(env: &Env, respuesta: u32) -> Result<Respuesta, Error> {
        env.storage().persistent().get(&Clave::Respuesta(respuesta)).ok_or(Error::RespuestaInexistente)
    }
}

#[contractimpl]
impl Comunidad {
    /// Crea un grupo de estudio; quien lo crea es dueño y primer miembro. Si es
    /// privado, solo entra quien el dueño invite. Regresa el número del grupo.
    pub fn crear_grupo_estudio(env: Env, dueno: Address, nombre: String, privado: bool) -> Result<u32, Error> {
        dueno.require_auth();
        if nombre.is_empty() {
            return Err(Error::NombreVacio);
        }
        let n = siguiente(&env, &Clave::NGrupos);
        let mut miembros = Vec::new(&env);
        miembros.push_back(dueno.clone());
        guardar(&env, &Clave::Grupo(n), &Grupo { dueno: dueno.clone(), nombre, privado, cerrado: false, miembros });
        GrupoCambio { grupo: n, quien: dueno, accion: symbol_short!("crear") }.publish(&env);
        Ok(n)
    }

    /// El dueño invita a `invitado` a su grupo (necesario si es privado).
    pub fn invitar_a_grupo(env: Env, dueno: Address, grupo: u32, invitado: Address) -> Result<(), Error> {
        let g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if g.miembros.contains(&invitado) {
            return Err(Error::YaEsMiembro);
        }
        guardar(&env, &Clave::Invitado(grupo, invitado.clone()), &true);
        GrupoCambio { grupo, quien: invitado, accion: symbol_short!("invitar") }.publish(&env);
        Ok(())
    }

    /// `alumno` se une a un grupo abierto, o a uno privado si tiene invitación.
    pub fn unirse_grupo_estudio(env: Env, alumno: Address, grupo: u32) -> Result<(), Error> {
        alumno.require_auth();
        let mut g = Self::cargar_grupo(&env, grupo)?;
        if g.cerrado {
            return Err(Error::GrupoCerrado);
        }
        if g.miembros.contains(&alumno) {
            return Err(Error::YaEsMiembro);
        }
        let inv = Clave::Invitado(grupo, alumno.clone());
        if g.privado && !env.storage().persistent().has(&inv) {
            return Err(Error::SinInvitacion);
        }
        env.storage().persistent().remove(&inv);
        g.miembros.push_back(alumno.clone());
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: alumno, accion: symbol_short!("unirse") }.publish(&env);
        Ok(())
    }

    /// Un miembro sale del grupo. El dueño no sale: primero transfiere el grupo.
    pub fn salir_grupo_estudio(env: Env, alumno: Address, grupo: u32) -> Result<(), Error> {
        alumno.require_auth();
        let mut g = Self::cargar_grupo(&env, grupo)?;
        if g.dueno == alumno {
            return Err(Error::DuenoNoSale);
        }
        let i = g.miembros.first_index_of(&alumno).ok_or(Error::NoEsMiembro)?;
        g.miembros.remove(i);
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: alumno, accion: symbol_short!("salir") }.publish(&env);
        Ok(())
    }

    /// El dueño saca a un miembro del grupo.
    pub fn expulsar_de_grupo(env: Env, dueno: Address, grupo: u32, miembro: Address) -> Result<(), Error> {
        let mut g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if miembro == dueno {
            return Err(Error::DuenoNoSale);
        }
        let i = g.miembros.first_index_of(&miembro).ok_or(Error::NoEsMiembro)?;
        g.miembros.remove(i);
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: miembro, accion: symbol_short!("expulsar") }.publish(&env);
        Ok(())
    }

    /// El dueño pasa el grupo a otro miembro.
    pub fn transferir_grupo(env: Env, dueno: Address, grupo: u32, nuevo: Address) -> Result<(), Error> {
        let mut g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if !g.miembros.contains(&nuevo) {
            return Err(Error::NoEsMiembro);
        }
        g.dueno = nuevo.clone();
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: nuevo, accion: symbol_short!("transfer") }.publish(&env);
        Ok(())
    }

    /// El dueño cierra el grupo: ya nadie entra, pregunta ni responde; lo escrito queda.
    pub fn cerrar_grupo_estudio(env: Env, dueno: Address, grupo: u32) -> Result<(), Error> {
        let mut g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if g.cerrado {
            return Err(Error::GrupoCerrado);
        }
        g.cerrado = true;
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: dueno, accion: symbol_short!("cerrar") }.publish(&env);
        Ok(())
    }

    /// Un miembro pregunta en su grupo. Regresa el número de la pregunta.
    pub fn hacer_pregunta(env: Env, autor: Address, grupo: u32, texto: String) -> Result<u32, Error> {
        autor.require_auth();
        Self::miembro_abierto(&env, &autor, grupo)?;
        revisar_texto(&texto)?;
        let n = siguiente(&env, &Clave::NPreguntas);
        guardar(&env, &Clave::Pregunta(n), &Pregunta {
            grupo, autor: autor.clone(), texto, aceptada: None, fijada: false, oculta: false, reportes: 0,
        });
        ForoCambio { numero: n, quien: autor, accion: symbol_short!("preguntar") }.publish(&env);
        Ok(n)
    }

    /// Un miembro del grupo responde una pregunta visible. Regresa el número de la respuesta.
    pub fn responder_pregunta(env: Env, autor: Address, pregunta: u32, texto: String) -> Result<u32, Error> {
        autor.require_auth();
        let p = Self::cargar_pregunta(&env, pregunta)?;
        if p.oculta {
            return Err(Error::PreguntaOculta);
        }
        Self::miembro_abierto(&env, &autor, p.grupo)?;
        revisar_texto(&texto)?;
        let n = siguiente(&env, &Clave::NRespuestas);
        guardar(&env, &Clave::Respuesta(n), &Respuesta { pregunta, autor: autor.clone(), texto, votos: 0 });
        ForoCambio { numero: n, quien: autor, accion: symbol_short!("responder") }.publish(&env);
        Ok(n)
    }

    /// Quien preguntó marca la respuesta que le sirvió (puede cambiarla después).
    pub fn aceptar_respuesta(env: Env, autor: Address, pregunta: u32, respuesta: u32) -> Result<(), Error> {
        autor.require_auth();
        let mut p = Self::cargar_pregunta(&env, pregunta)?;
        if p.autor != autor {
            return Err(Error::NoEsAutor);
        }
        if Self::cargar_respuesta(&env, respuesta)?.pregunta != pregunta {
            return Err(Error::RespuestaAjena);
        }
        p.aceptada = Some(respuesta);
        guardar(&env, &Clave::Pregunta(pregunta), &p);
        ForoCambio { numero: respuesta, quien: autor, accion: symbol_short!("aceptar") }.publish(&env);
        Ok(())
    }

    /// Un miembro vota una respuesta útil, una vez y nunca la suya. Regresa los votos.
    pub fn votar_respuesta(env: Env, votante: Address, respuesta: u32) -> Result<u32, Error> {
        votante.require_auth();
        let mut r = Self::cargar_respuesta(&env, respuesta)?;
        if r.autor == votante {
            return Err(Error::VotoPropio);
        }
        let p = Self::cargar_pregunta(&env, r.pregunta)?;
        Self::miembro_abierto(&env, &votante, p.grupo)?;
        let voto = Clave::Voto(respuesta, votante.clone());
        if env.storage().persistent().has(&voto) {
            return Err(Error::YaVoto);
        }
        guardar(&env, &voto, &true);
        r.votos += 1;
        guardar(&env, &Clave::Respuesta(respuesta), &r);
        ForoCambio { numero: respuesta, quien: votante, accion: symbol_short!("votar") }.publish(&env);
        Ok(r.votos)
    }

    /// El dueño del grupo fija (o desfija) una pregunta arriba del foro.
    pub fn fijar_pregunta(env: Env, dueno: Address, pregunta: u32, fijada: bool) -> Result<(), Error> {
        let mut p = Self::cargar_pregunta(&env, pregunta)?;
        Self::puede_moderar(&env, &dueno, p.grupo)?;
        p.fijada = fijada;
        guardar(&env, &Clave::Pregunta(pregunta), &p);
        ForoCambio { numero: pregunta, quien: dueno, accion: symbol_short!("fijar") }.publish(&env);
        Ok(())
    }

    /// Un miembro reporta una pregunta para que el dueño la revise, una vez por miembro.
    /// Regresa cuántos reportes lleva.
    pub fn reportar_pregunta(env: Env, miembro: Address, pregunta: u32) -> Result<u32, Error> {
        miembro.require_auth();
        let mut p = Self::cargar_pregunta(&env, pregunta)?;
        Self::miembro_abierto(&env, &miembro, p.grupo)?;
        let rep = Clave::Reporte(pregunta, miembro.clone());
        if env.storage().persistent().has(&rep) {
            return Err(Error::YaReportada);
        }
        guardar(&env, &rep, &true);
        p.reportes += 1;
        guardar(&env, &Clave::Pregunta(pregunta), &p);
        ForoCambio { numero: pregunta, quien: miembro, accion: symbol_short!("reportar") }.publish(&env);
        Ok(p.reportes)
    }

    /// El dueño modera: oculta (o vuelve a mostrar) una pregunta. Oculta, nadie la responde.
    pub fn ocultar_pregunta(env: Env, dueno: Address, pregunta: u32, oculta: bool) -> Result<(), Error> {
        let mut p = Self::cargar_pregunta(&env, pregunta)?;
        Self::puede_moderar(&env, &dueno, p.grupo)?;
        p.oculta = oculta;
        guardar(&env, &Clave::Pregunta(pregunta), &p);
        ForoCambio { numero: pregunta, quien: dueno, accion: symbol_short!("ocultar") }.publish(&env);
        Ok(())
    }

    /// El dueño nombra moderador a un miembro: puede fijar, ocultar y quitar respuestas.
    pub fn nombrar_moderador_grupo(env: Env, dueno: Address, grupo: u32, miembro: Address) -> Result<(), Error> {
        let g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if !g.miembros.contains(&miembro) {
            return Err(Error::NoEsMiembro);
        }
        let k = Clave::Moderador(grupo, miembro.clone());
        if env.storage().persistent().has(&k) {
            return Err(Error::YaEsModerador);
        }
        guardar(&env, &k, &true);
        GrupoCambio { grupo, quien: miembro, accion: symbol_short!("moderador") }.publish(&env);
        Ok(())
    }

    /// El dueño le quita el rol de moderador a alguien.
    pub fn quitar_moderador_grupo(env: Env, dueno: Address, grupo: u32, miembro: Address) -> Result<(), Error> {
        Self::grupo_del_dueno(&env, &dueno, grupo)?;
        let k = Clave::Moderador(grupo, miembro.clone());
        if !env.storage().persistent().has(&k) {
            return Err(Error::NoEsModerador);
        }
        env.storage().persistent().remove(&k);
        GrupoCambio { grupo, quien: miembro, accion: symbol_short!("sinmod") }.publish(&env);
        Ok(())
    }

    /// El dueño cambia el nombre del grupo.
    pub fn renombrar_grupo_estudio(env: Env, dueno: Address, grupo: u32, nombre: String) -> Result<(), Error> {
        let mut g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if nombre.is_empty() {
            return Err(Error::NombreVacio);
        }
        g.nombre = nombre;
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: dueno, accion: symbol_short!("renombrar") }.publish(&env);
        Ok(())
    }

    /// El dueño reabre un grupo cerrado (por ejemplo, para una nueva generación).
    pub fn reabrir_grupo_estudio(env: Env, dueno: Address, grupo: u32) -> Result<(), Error> {
        let mut g = Self::grupo_del_dueno(&env, &dueno, grupo)?;
        if !g.cerrado {
            return Err(Error::GrupoAbierto);
        }
        g.cerrado = false;
        guardar(&env, &Clave::Grupo(grupo), &g);
        GrupoCambio { grupo, quien: dueno, accion: symbol_short!("reabrir") }.publish(&env);
        Ok(())
    }

    /// Quien preguntó corrige el texto de su pregunta mientras no tenga respuesta aceptada.
    pub fn editar_pregunta(env: Env, autor: Address, pregunta: u32, texto: String) -> Result<(), Error> {
        autor.require_auth();
        let mut p = Self::cargar_pregunta(&env, pregunta)?;
        if p.autor != autor {
            return Err(Error::NoEsAutor);
        }
        if p.aceptada.is_some() {
            return Err(Error::YaAceptada);
        }
        Self::miembro_abierto(&env, &autor, p.grupo)?;
        revisar_texto(&texto)?;
        p.texto = texto;
        guardar(&env, &Clave::Pregunta(pregunta), &p);
        ForoCambio { numero: pregunta, quien: autor, accion: symbol_short!("editar") }.publish(&env);
        Ok(())
    }

    /// Quien respondió corrige su respuesta.
    pub fn editar_respuesta(env: Env, autor: Address, respuesta: u32, texto: String) -> Result<(), Error> {
        autor.require_auth();
        let mut r = Self::cargar_respuesta(&env, respuesta)?;
        if r.autor != autor {
            return Err(Error::NoEsAutor);
        }
        let p = Self::cargar_pregunta(&env, r.pregunta)?;
        Self::miembro_abierto(&env, &autor, p.grupo)?;
        revisar_texto(&texto)?;
        r.texto = texto;
        guardar(&env, &Clave::Respuesta(respuesta), &r);
        ForoCambio { numero: respuesta, quien: autor, accion: symbol_short!("editresp") }.publish(&env);
        Ok(())
    }

    /// Un miembro retira su voto. Regresa los votos que quedan.
    pub fn quitar_voto_respuesta(env: Env, votante: Address, respuesta: u32) -> Result<u32, Error> {
        votante.require_auth();
        let mut r = Self::cargar_respuesta(&env, respuesta)?;
        let voto = Clave::Voto(respuesta, votante.clone());
        if !env.storage().persistent().has(&voto) {
            return Err(Error::SinVoto);
        }
        env.storage().persistent().remove(&voto);
        r.votos -= 1;
        guardar(&env, &Clave::Respuesta(respuesta), &r);
        ForoCambio { numero: respuesta, quien: votante, accion: symbol_short!("sinvoto") }.publish(&env);
        Ok(r.votos)
    }

    /// El autor o un moderador del grupo borra una respuesta; si era la aceptada, la
    /// pregunta vuelve a quedar sin aceptar.
    pub fn borrar_respuesta_foro(env: Env, quien: Address, respuesta: u32) -> Result<(), Error> {
        let r = Self::cargar_respuesta(&env, respuesta)?;
        let mut p = Self::cargar_pregunta(&env, r.pregunta)?;
        if r.autor == quien {
            quien.require_auth();
        } else {
            Self::puede_moderar(&env, &quien, p.grupo)?;
        }
        env.storage().persistent().remove(&Clave::Respuesta(respuesta));
        if p.aceptada == Some(respuesta) {
            p.aceptada = None;
            guardar(&env, &Clave::Pregunta(r.pregunta), &p);
        }
        ForoCambio { numero: respuesta, quien, accion: symbol_short!("borrar") }.publish(&env);
        Ok(())
    }

    pub fn es_moderador_grupo(env: Env, grupo: u32, quien: Address) -> bool {
        env.storage().persistent().has(&Clave::Moderador(grupo, quien))
    }

    pub fn grupo(env: Env, grupo: u32) -> Option<Grupo> {
        env.storage().persistent().get(&Clave::Grupo(grupo))
    }

    pub fn pregunta(env: Env, pregunta: u32) -> Option<Pregunta> {
        env.storage().persistent().get(&Clave::Pregunta(pregunta))
    }

    pub fn respuesta(env: Env, respuesta: u32) -> Option<Respuesta> {
        env.storage().persistent().get(&Clave::Respuesta(respuesta))
    }
}

#[cfg(test)]
mod test;
