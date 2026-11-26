// The Editor window: sign in with Budi, open your workspace (about a minute),
// and VS Code from the session shows up here. One workspace per person, with a
// time limit; the service in `server/` opens and closes it.
import { montarSesion, sesion } from "./sesion.js";

const API = import.meta.env.VITE_BUDI_STELLAR_API ?? "http://localhost:8788";
const APP_ID = import.meta.env.VITE_PRIVY_APP_ID ?? "cmv950xc802g30flc8r3ce6rm";

interface Vista {
  estado: "abriendo" | "listo" | "error";
  cierra: number;
  ide?: { url: string };
  error?: string;
}

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const caja = $("boot");
const marco = $<HTMLIFrameElement>("ide");
const chip = $("estado-ws");
let sondeo: number | undefined;
let desde = 0;

async function api(method: "GET" | "POST" | "DELETE"): Promise<Vista | null> {
  const token = await sesion.token();
  if (!token) throw new Error("inicia sesión con tu cuenta de Budi");
  const r = await fetch(`${API}/workspace`, { method, headers: { Authorization: `Bearer ${token}` } });
  const body = await r.json();
  if (!r.ok) throw new Error(body?.error ?? `error ${r.status}`);
  return body;
}

const hora = (ms: number) => new Date(ms).toLocaleTimeString("es-MX", { hour: "2-digit", minute: "2-digit" });

function mostrar(html: string) {
  marco.hidden = true;
  marco.removeAttribute("src");
  caja.hidden = false;
  caja.innerHTML = html;
}

function pintar(v: Vista | null) {
  clearTimeout(sondeo);
  if (!sesion.dentro) {
    chip.textContent = "workspace: apagado";
    mostrar(`<div class="hand">tu workspace: Rust, Stellar CLI y tu agente Budi, listo en un minuto</div>
      <p>Para abrirlo, entra con tu cuenta de Budi.</p>
      <button class="btn primary" data-ws="entrar">Entrar con mi cuenta de Budi</button>`);
    return;
  }
  if (!v) {
    chip.textContent = "workspace: apagado";
    mostrar(`<div class="hand">tu workspace: Rust, Stellar CLI y tu agente Budi, listo en un minuto</div>
      <p>Tienes uno a la vez, por 45 minutos. Entraste como <b>${sesion.correo ?? "tú"}</b>.</p>
      <div class="row"><button class="btn primary" data-ws="abrir">Abrir mi workspace</button>
      <button class="btn ghost" data-ws="salir">Salir</button></div>`);
    return;
  }
  if (v.estado === "abriendo") {
    chip.textContent = "workspace: encendiendo";
    const s = Math.round((Date.now() - desde) / 1000);
    mostrar(`<div class="hand">encendiendo tu workspace…</div>
      <p class="arranque"><span class="punto"></span> ${desde ? `${s} s` : ""} · suele tardar menos de un minuto</p>`);
    sondeo = window.setTimeout(refrescar, 2000);
    return;
  }
  if (v.estado === "error") {
    chip.textContent = "workspace: error";
    mostrar(`<p>No pudimos abrir tu workspace: ${v.error ?? "error desconocido"}</p>
      <button class="btn primary" data-ws="abrir">Intentar de nuevo</button>`);
    return;
  }
  chip.textContent = `workspace: listo · cierra ${hora(v.cierra)}`;
  caja.hidden = true;
  if (marco.src !== v.ide!.url) marco.src = v.ide!.url;
  marco.hidden = false;
  $("ws-cerrar").hidden = false;
}

async function refrescar() {
  try {
    pintar(sesion.dentro ? await api("GET") : null);
  } catch (err) {
    mostrar(`<p>${(err as Error).message}</p><button class="btn primary" data-ws="reintentar">Reintentar</button>`);
  }
}

caja.addEventListener("click", async (e) => {
  const accion = (e.target as HTMLElement).closest<HTMLElement>("[data-ws]")?.dataset.ws;
  if (accion === "entrar") sesion.entrar();
  if (accion === "salir") await sesion.salir();
  if (accion === "reintentar") await refrescar();
  if (accion === "abrir") {
    desde = Date.now();
    try {
      pintar(await api("POST"));
    } catch (err) {
      mostrar(`<p>${(err as Error).message}</p>`);
    }
  }
});

$("ws-cerrar").addEventListener("click", async () => {
  $("ws-cerrar").hidden = true;
  await api("DELETE").catch(() => {});
  pintar(null);
});

window.addEventListener("sesion", () => {
  if (sesion.lista) void refrescar();
});
montarSesion(APP_ID);
