// Contrata a un equipo humano + agente por su reputación (contrato Reputación).
// Cualquiera ve los equipos y su puntaje; con Freighter, el cliente contrata, acepta o
// disputa, y el humano del equipo entrega o cobra si el cliente no responde.
import { Address, nativeToScVal } from "@stellar/stellar-sdk";
import { conectar, escribir, existe, EXPLORER, fondear, leer } from "./stellar";

const REPUTACION = "CCNHF6DUBUDY5MG2AQYXZU4OHHG2NXWVIFDZQLG4Z3XDJ4N5KFDASFCL";
const HABILIDAD = "soroban";
// Cuenta con fondos que sirve de origen para leer sin wallet (las lecturas no se firman).
const LECTOR = "GDCTBCAWSNHUAZST4HMX7FOEBZFKJXQXN5PYYHBHXT66GH6R6A3KUJ4M";
const XLM = 10_000_000n;
const MAX = 50;

type Equipo = { id: number; humano: string; agente: string; activo: boolean; puntos: number; trabajos: number };
type Trabajo = {
  id: number; cliente: string; equipo: number; monto: bigint; limite: bigint;
  revisar_hasta: bigint; prueba: string; puntos: number; estado: number;
};

const ESTADOS = ["En custodia", "Entregado", "Pagado", "Reembolsado", "Anulado", "En disputa"];
const ERRORES: Record<number, string> = {
  3: "El pago tiene que ser mayor a 0.",
  6: "Ese equipo ya no toma trabajos.",
  7: "El humano de ese equipo no tiene la insignia de esta habilidad.",
  9: "Ese trabajo ya no está en ese paso. Recarga la página.",
  10: "Ya venció el plazo para eso.",
  11: "Todavía no vence el plazo.",
  13: "No puedes contratar a tu propio equipo.",
  14: "Escribe la prueba de tu entrega (un enlace).",
  15: "Escribe el motivo.",
};

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const corta = (c: string) => `${c.slice(0, 4)}…${c.slice(-4)}`;
const enlace = (hash: string) => `<a href="${EXPLORER}/tx/${hash}" target="_blank" rel="noopener">ver la transacción</a>`;
const esc = (t: string) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
const u32 = (n: number) => nativeToScVal(n, { type: "u32" });
const texto = (t: string) => nativeToScVal(t, { type: "string" });
const fecha = (s: bigint) => new Date(Number(s) * 1000).toLocaleString("es-MX", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
const ahora = () => BigInt(Math.floor(Date.now() / 1000));

let cuenta = "";
let equipos: Equipo[] = [];

function mensaje(e: unknown): string {
  const m = (e as Error).message ?? String(e);
  const n = /Error\(Contract, #(\d+)\)/.exec(m);
  return n && ERRORES[Number(n[1])] ? ERRORES[Number(n[1])] : m;
}

function decir(id: string, html: string, error = false) {
  const el = $(id);
  el.innerHTML = html;
  el.classList.toggle("err", error);
}

async function ocupado(btn: HTMLButtonElement, txt: string, fn: () => Promise<void>) {
  const antes = btn.textContent;
  btn.disabled = true;
  btn.textContent = txt;
  try {
    await fn();
  } finally {
    btn.textContent = antes;
    btn.disabled = false;
  }
}

/** Lee del 1 en adelante hasta que el contrato diga que no existe. */
async function todos<T>(metodo: string, de: string): Promise<T[]> {
  const lista: T[] = [];
  for (let id = 1; id <= MAX; id++) {
    try {
      lista.push({ id, ...(await leer(de, REPUTACION, metodo, [u32(id)])) } as T);
    } catch {
      break;
    }
  }
  return lista;
}

async function cargarEquipos() {
  const de = cuenta || LECTOR;
  const base = await todos<Omit<Equipo, "puntos" | "trabajos">>("equipo", de);
  equipos = await Promise.all(
    base.map(async (e) => {
      const p = (await leer(de, REPUTACION, "puntaje", [u32(e.id), nativeToScVal(HABILIDAD, { type: "symbol" })])) as { puntos: number; trabajos: number };
      return { ...e, puntos: Number(p.puntos), trabajos: Number(p.trabajos) };
    }),
  );
  equipos.sort((a, b) => Number(b.activo) - Number(a.activo) || b.puntos - a.puntos);
  $("equipos").innerHTML = equipos.length
    ? equipos
        .map(
          (e) => `<div class="equipo${e.activo ? "" : " retirado"}">
        <div class="pts"><b>${e.puntos}</b><span>puntos</span></div>
        <div><b>Equipo ${e.id}</b>${e.activo ? "" : " · retirado"}<br>
        <span class="nota">Humano ${corta(e.humano)} · agente ${corta(e.agente)} · ${e.trabajos} trabajo${e.trabajos === 1 ? "" : "s"} vigente${e.trabajos === 1 ? "" : "s"}</span></div>
      </div>`,
        )
        .join("")
    : `<p class="nota">Todavía no hay equipos.</p>`;
  $("equipo").innerHTML = equipos
    .filter((e) => e.activo)
    .map((e) => `<option value="${e.id}">Equipo ${e.id} · ${e.puntos} puntos</option>`)
    .join("");
}

function botones(t: Trabajo, soyCliente: boolean, soyHumano: boolean): string {
  const ya = ahora();
  const b: string[] = [];
  if (soyCliente && (t.estado === 1 || t.estado === 5)) b.push(`<button class="btn primary" data-accion="aceptar" data-id="${t.id}">Aceptar y pagar</button>`);
  if (soyCliente && t.estado === 1 && ya < t.revisar_hasta)
    b.push(`<input data-motivo="${t.id}" placeholder="Qué falta o qué está mal"><button class="btn ghost" data-accion="disputar" data-id="${t.id}">Disputar</button>`);
  if (soyCliente && t.estado === 0 && ya >= t.limite) b.push(`<button class="btn ghost" data-accion="reembolsar" data-id="${t.id}">Recuperar mi pago</button>`);
  if (soyHumano && t.estado === 0 && ya < t.limite)
    b.push(`<input data-prueba="${t.id}" placeholder="Enlace a tu entrega"><button class="btn primary" data-accion="entregar" data-id="${t.id}">Entregar</button>`);
  if (soyHumano && t.estado === 1 && ya >= t.revisar_hasta) b.push(`<button class="btn primary" data-accion="cobrar_sin_respuesta" data-id="${t.id}">Cobrar</button>`);
  return b.length ? `<div class="row">${b.join("")}</div>` : "";
}

function detalle(t: Trabajo): string {
  if (t.estado === 0) return `Entrega a más tardar el ${fecha(t.limite)}.`;
  if (t.estado === 1) return `Entregado: <a href="${esc(t.prueba)}" target="_blank" rel="noopener">${esc(t.prueba)}</a>. Revisión hasta el ${fecha(t.revisar_hasta)}; después, el equipo puede cobrar.`;
  if (t.estado === 2) return `Pagado al equipo. Sumó ${t.puntos} punto${t.puntos === 1 ? "" : "s"}.`;
  if (t.estado === 5) return "En disputa: el pago está congelado hasta que Budi resuelva.";
  return "";
}

async function cargarTrabajos() {
  const mios = equipos.filter((e) => e.humano === cuenta).map((e) => e.id);
  const lista = (await todos<Trabajo>("trabajo", cuenta)).filter((t) => t.cliente === cuenta || mios.includes(t.equipo));
  $("trabajos").innerHTML = lista.length
    ? lista
        .reverse()
        .map((t) => {
          const soyCliente = t.cliente === cuenta;
          return `<div class="trabajo">
        <div class="cab"><b>Trabajo ${t.id}</b> · equipo ${t.equipo} · ${Number(t.monto / XLM)} XLM · <span class="estado e${t.estado}">${ESTADOS[t.estado]}</span>
        <span class="rol">${soyCliente ? "contrataste" : "tu equipo"}</span></div>
        <p class="nota">${detalle(t)}</p>${botones(t, soyCliente, mios.includes(t.equipo))}</div>`;
        })
        .join("")
    : `<p class="nota">Aún no tienes trabajos con esta wallet.</p>`;
}

async function revisar() {
  if (!(await existe(cuenta))) {
    $("fondear").hidden = false;
    decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>. Tu cuenta aún no existe en testnet: fondéala primero.`);
    return;
  }
  $("fondear").hidden = true;
  decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>.`);
  for (const id of ["paso-contratar", "paso-trabajos"]) $(id).classList.remove("off");
  for (const id of ["equipo", "monto", "plazo", "contratar"]) ($(id) as HTMLInputElement).disabled = false;
  await cargarEquipos();
  await cargarTrabajos();
}

$("conectar").addEventListener("click", () =>
  ocupado($("conectar"), "Conectando…", async () => {
    try {
      cuenta = await conectar();
      $("conectar").textContent = "Wallet conectada";
      await revisar();
    } catch (e) {
      decir("out-wallet", mensaje(e), true);
    }
  }),
);

$("fondear").addEventListener("click", () =>
  ocupado($("fondear"), "Fondeando…", async () => {
    try {
      await fondear(cuenta);
      await revisar();
    } catch (e) {
      decir("out-wallet", mensaje(e), true);
    }
  }),
);

$("contratar").addEventListener("click", () =>
  ocupado($("contratar"), "Firma en Freighter…", async () => {
    const equipo = Number(($("equipo") as HTMLSelectElement).value);
    const monto = Math.floor(Number(($("monto") as HTMLInputElement).value));
    const dias = Math.floor(Number(($("plazo") as HTMLInputElement).value));
    if (!equipo) return decir("out-contratar", "Elige un equipo.", true);
    if (!(monto > 0) || !(dias > 0)) return decir("out-contratar", "El pago y el plazo tienen que ser mayores a 0.", true);
    try {
      const hash = await escribir(cuenta, REPUTACION, "contratar", [
        new Address(cuenta).toScVal(),
        u32(equipo),
        nativeToScVal(HABILIDAD, { type: "symbol" }),
        nativeToScVal(BigInt(monto) * XLM, { type: "i128" }),
        nativeToScVal(BigInt(dias * 86_400), { type: "u64" }),
      ]);
      decir("out-contratar", `Listo: tus ${monto} XLM quedaron en custodia. El equipo tiene ${dias} día${dias === 1 ? "" : "s"} para entregar. ${enlace(hash)}`);
      await cargarTrabajos();
    } catch (e) {
      decir("out-contratar", mensaje(e), true);
    }
  }),
);

const LISTO: Record<string, string> = {
  aceptar: "Aceptaste: el equipo cobró y sumó sus puntos.",
  disputar: "Disputa abierta: el pago queda congelado hasta que Budi resuelva.",
  reembolsar: "Recuperaste tu pago.",
  entregar: "Entregaste. El cliente tiene su plazo de revisión; si no responde, cobras.",
  cobrar_sin_respuesta: "Cobraste: el cliente no respondió en el plazo de revisión.",
};

$("trabajos").addEventListener("click", (ev) => {
  const btn = (ev.target as HTMLElement).closest<HTMLButtonElement>("button[data-accion]");
  if (!btn) return;
  const accion = btn.dataset.accion!;
  const id = Number(btn.dataset.id);
  const args = [u32(id)];
  if (accion === "disputar") args.push(texto(document.querySelector<HTMLInputElement>(`[data-motivo="${id}"]`)!.value.trim()));
  if (accion === "entregar") args.push(texto(document.querySelector<HTMLInputElement>(`[data-prueba="${id}"]`)!.value.trim()));
  void ocupado(btn, "Firma en Freighter…", async () => {
    try {
      const hash = await escribir(cuenta, REPUTACION, accion, args);
      decir("out-trabajos", `${LISTO[accion]} ${enlace(hash)}`);
      await cargarEquipos();
      await cargarTrabajos();
    } catch (e) {
      decir("out-trabajos", mensaje(e), true);
    }
  });
});

cargarEquipos().catch(() => decir("equipos", "No pudimos leer la red. Recarga la página.", true));
