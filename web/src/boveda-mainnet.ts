// La bóveda en mainnet: el mismo contrato `boveda`, desplegado en la red principal. Cada botón
// firma una transacción en Freighter: depositar, reclamar y las tres consultas (saldo, reclamado,
// admin) enviadas como transacción para que queden en el historial. El id del contrato se lee de
// /boveda/mainnet/contrato.json.
import { Buffer } from "buffer";
import { Address, nativeToScVal, xdr } from "@stellar/stellar-sdk";
import { conectar, escribir, EXPLORER, existe, leer, resultado, usarMainnet } from "./stellar";

usarMainnet();

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const corta = (c: string) => `${c.slice(0, 4)}…${c.slice(-4)}`;
const enlace = (hash: string) => `<a href="${EXPLORER}/tx/${hash}" target="_blank" rel="noopener">ver la transacción</a>`;
const xlm = (stroops: bigint | number) =>
  `${(Number(stroops) / 10_000_000).toLocaleString("es-MX", { maximumFractionDigits: 7 })} XLM`;

// Cuenta de Budi que desplegó el contrato: sirve para leer el saldo sin wallet (simulación, nunca firma).
const LECTOR = "GCL7OIN4KP646MU23OSR4PX6YJ5WBHCYPSRXQV2CGJZZVVYNA3D76BZO";

let cuenta = "";
let contrato = "";
let examen = "";
let insignias = "";
let bitacora = "";
let ultimoHito = 0;

function decir(id: string, html: string, error = false) {
  const el = $(id);
  el.innerHTML = html;
  el.classList.toggle("err", error);
}

async function ocupado(btn: HTMLButtonElement, texto: string, fn: () => Promise<void>) {
  const antes = btn.innerHTML;
  btn.disabled = true;
  btn.textContent = texto;
  try {
    await fn();
  } finally {
    btn.innerHTML = antes;
    btn.disabled = false;
  }
}

const ERRORES: Record<string, string> = {
  "1": "El monto tiene que ser mayor que cero.",
  "2": "Ese código no abre la bóveda.",
  "3": "Esta wallet ya abrió la bóveda una vez.",
  "4": "La bóveda está vacía. Deposita primero.",
};
// Errores del contrato Examen (mismos números, otro contrato).
const ERRORES_EXAMEN: Record<string, string> = {
  "1": "Alguna respuesta no es correcta. Revisa y vuelve a presentar.",
  "2": "Esta wallet ya aprobó el examen y tiene su insignia.",
};
const error = (e: unknown) => {
  const m = (e as Error).message;
  const n = /Error\(Contract, #(\d+)\)/.exec(m)?.[1];
  return (n && ERRORES[n]) || m;
};

async function pintarSaldo() {
  if (!contrato) {
    decir("out-boveda", "La bóveda en mainnet todavía no está desplegada.");
    $("saldo").textContent = "0 XLM";
    return;
  }
  $("contrato").innerHTML = `<a href="${EXPLORER}/contract/${contrato}" target="_blank" rel="noopener">Ver el contrato en stellar.expert</a>. `;
  try {
    $("saldo").textContent = xlm((await leer(cuenta || LECTOR, contrato, "saldo", [])) as bigint);
  } catch {
    $("saldo").textContent = "…";
  }
}

const yo = () => new Address(cuenta).toScVal();

$("conectar").addEventListener("click", () =>
  ocupado($("conectar"), "Conectando…", async () => {
    try {
      cuenta = await conectar();
      if (!(await existe(cuenta))) {
        return decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>. Esta cuenta no tiene XLM en mainnet todavía.`, true);
      }
      decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>.`);
      if (!contrato) return;
      for (const id of ["paso-depositar", "paso-reclamar", "paso-registro", "paso-examen", "paso-bitacora"]) $(id).classList.remove("off");
      for (const el of document.querySelectorAll<HTMLInputElement | HTMLButtonElement>("main input, main button")) el.disabled = false;
      await pintarSaldo();
    } catch (e) {
      decir("out-wallet", error(e), true);
    }
  }),
);

$("depositar").addEventListener("click", () =>
  ocupado($("depositar"), "Firma en Freighter…", async () => {
    const monto = Number(($("monto") as HTMLInputElement).value.replace(",", "."));
    if (!(monto > 0)) return decir("out-depositar", "Escribe cuántos XLM quieres depositar.", true);
    try {
      const stroops = BigInt(Math.round(monto * 10_000_000));
      const hash = await escribir(cuenta, contrato, "depositar", [yo(), nativeToScVal(stroops, { type: "i128" })]);
      decir("out-depositar", `Depositaste ${xlm(stroops)}. ${enlace(hash)}`);
      await pintarSaldo();
    } catch (e) {
      decir("out-depositar", error(e), true);
    }
  }),
);

$("reclamar").addEventListener("click", () =>
  ocupado($("reclamar"), "Firma en Freighter…", async () => {
    const texto = ($("codigo") as HTMLInputElement).value;
    if (!texto.trim()) return decir("out-reclamar", "Escribe el código primero.", true);
    try {
      const antes = (await leer(cuenta, contrato, "saldo", [])) as bigint;
      const hash = await escribir(cuenta, contrato, "reclamar", [yo(), xdr.ScVal.scvBytes(Buffer.from(texto, "utf8"))]);
      decir("out-reclamar", `¡Abriste la bóveda! ${xlm(antes)} van a tu wallet. ${enlace(hash)}`);
      await pintarSaldo();
    } catch (e) {
      decir("out-reclamar", error(e), true);
    }
  }),
);

// La imagen de la insignia: la medalla de la ruta en el brand de Budi (oro, tinta, sombra sin blur).
// Se dibuja aquí con las fuentes de la página; el contrato guarda la insignia, no la imagen.
function medalla(id: number | string, ruta: string, fecha?: number) {
  const dia = fecha ? new Date(fecha * 1000).toLocaleDateString("es-MX", { day: "numeric", month: "short", year: "numeric" }) : "";
  const nombre = ruta === "soroban1" ? "Soroban 101" : ruta;
  return `<figure class="medalla" aria-label="Insignia ${nombre} #${id}">
  <svg viewBox="0 0 240 260" width="200" role="img">
    <path d="M78 150 L52 248 L92 228 L112 256 L128 160 Z" fill="#7c3aed" stroke="#0a0a0a" stroke-width="5" stroke-linejoin="round"/>
    <path d="M162 150 L188 248 L148 228 L128 256 L112 160 Z" fill="#c4177a" stroke="#0a0a0a" stroke-width="5" stroke-linejoin="round"/>
    <circle cx="126" cy="106" r="92" fill="#0a0a0a"/>
    <circle cx="120" cy="100" r="92" fill="#ffb800" stroke="#0a0a0a" stroke-width="6"/>
    <circle cx="120" cy="100" r="74" fill="none" stroke="#0a0a0a" stroke-width="3" stroke-dasharray="3 7"/>
    <text x="120" y="72" text-anchor="middle" font-family="Rubik Variable, sans-serif" font-weight="600" font-size="10.5" fill="#0a0a0a" letter-spacing="1">BUDI PARA STELLAR</text>
    <text x="120" y="108" text-anchor="middle" font-family="Bricolage Grotesque Variable, sans-serif" font-weight="800" font-size="25" fill="#0a0a0a">${nombre}</text>
    <text x="120" y="132" text-anchor="middle" font-family="Rubik Variable, sans-serif" font-weight="500" font-size="13" fill="#0a0a0a">Insignia #${id}</text>
  </svg>
  <figcaption>${nombre} · Insignia #${id}${dia ? ` · ${dia}` : ""}<br><span>Un NFT en Stellar que no se transfiere: es tuya.</span></figcaption>
</figure>`;
}

const CONSULTAS: Record<string, { args: () => xdr.ScVal[]; texto: (v: unknown) => string }> = {
  saldo: { args: () => [], texto: (v) => `La bóveda guarda ${xlm(v as bigint)}.` },
  reclamado: { args: () => [yo()], texto: (v) => `Esta wallet se ha llevado ${xlm(v as bigint)}.` },
  admin: { args: () => [], texto: (v) => `La administra ${corta(String(v))}.` },
};

for (const btn of document.querySelectorAll<HTMLButtonElement>("button[data-fn]")) {
  btn.addEventListener("click", () =>
    ocupado(btn, "Firma en Freighter…", async () => {
      const fn = btn.dataset.fn!;
      try {
        const hash = await escribir(cuenta, contrato, fn, CONSULTAS[fn].args());
        const valor = await resultado(hash);
        decir("out-registro", `${CONSULTAS[fn].texto(valor)} ${enlace(hash)}`);
      } catch (e) {
        decir("out-registro", error(e), true);
      }
    }),
  );
}

$("presentar").addEventListener("click", () =>
  ocupado($("presentar"), "Firma en Freighter…", async () => {
    const r = ["p1", "p2", "p3"].map((p) => document.querySelector<HTMLInputElement>(`input[name=${p}]:checked`)?.value);
    if (r.some((x) => !x)) return decir("out-examen", "Responde las 3 preguntas.", true);
    try {
      const hash = await escribir(cuenta, examen, "presentar", [yo(), xdr.ScVal.scvBytes(Buffer.from(r.join(""), "utf8"))]);
      const id = await resultado(hash);
      decir("out-examen", `¡Aprobaste! Tu insignia #${id} ya es tuya en Stellar. ${enlace(hash)}${medalla(String(id), "soroban1")}`);
    } catch (e) {
      const m = (e as Error).message;
      const n = /Error\(Contract, #(\d+)\)/.exec(m)?.[1];
      decir("out-examen", (n && ERRORES_EXAMEN[n]) || m, true);
    }
  }),
);

$("ver-insignia").addEventListener("click", () =>
  ocupado($("ver-insignia"), "Firma en Freighter…", async () => {
    try {
      const hash = await escribir(cuenta, insignias, "insignias", [yo()]);
      const lista = ((await resultado(hash)) as { id: number; ruta: string; otorgada_en: bigint }[] | undefined) ?? [];
      const texto = lista.length
        ? `Tienes ${lista.length === 1 ? "1 insignia" : `${lista.length} insignias`}: ${lista.map((i) => `#${i.id} (${i.ruta})`).join(", ")}.`
        : "Todavía no tienes insignias. Aprueba el examen.";
      decir("out-examen", `${texto} ${enlace(hash)}${lista.map((i) => medalla(i.id, i.ruta, Number(i.otorgada_en))).join("")}`);
    } catch (e) {
      decir("out-examen", error(e), true);
    }
  }),
);

const ERRORES_BITACORA: Record<string, string> = {
  "1": "Escribe tu hito primero.",
  "2": "El hito es muy largo (máximo 140 caracteres).",
  "3": "Tu Bitácora está llena.",
  "6": "Ese hito no existe. Registra uno primero.",
};
const errorBitacora = (e: unknown) => {
  const m = (e as Error).message;
  const n = /Error\(Contract, #(\d+)\)/.exec(m)?.[1];
  return (n && ERRORES_BITACORA[n]) || m;
};

$("registrar").addEventListener("click", () =>
  ocupado($("registrar"), "Firma en Freighter…", async () => {
    const hito = ($("hito") as HTMLInputElement).value.trim();
    if (!hito) return decir("out-bitacora", "Escribe tu hito primero.", true);
    try {
      const hash = await escribir(cuenta, bitacora, "registrar", [yo(), nativeToScVal(hito, { type: "string" })]);
      ultimoHito = Number(await resultado(hash));
      decir("out-bitacora", `Registraste tu hito #${ultimoHito} en Stellar. ${enlace(hash)}`);
    } catch (e) {
      decir("out-bitacora", errorBitacora(e), true);
    }
  }),
);

$("corregir").addEventListener("click", () =>
  ocupado($("corregir"), "Firma en Freighter…", async () => {
    const hito = ($("hito-nuevo") as HTMLInputElement).value.trim();
    if (!hito) return decir("out-bitacora", "Escribe el texto corregido.", true);
    try {
      if (!ultimoHito) ultimoHito = ((await leer(cuenta, bitacora, "contar", [yo()])) as number) ?? 0;
      const hash = await escribir(cuenta, bitacora, "corregir", [yo(), nativeToScVal(ultimoHito, { type: "u32" }), nativeToScVal(hito, { type: "string" })]);
      decir("out-bitacora", `Corregiste tu hito #${ultimoHito}: «${hito}». El texto de antes queda en el evento. ${enlace(hash)}`);
    } catch (e) {
      decir("out-bitacora", errorBitacora(e), true);
    }
  }),
);

try {
  const r = await fetch("/boveda/mainnet/contrato.json", { cache: "no-store" });
  if (r.ok) {
    const j = (await r.json()) as { contrato?: string; examen?: string; insignias?: string; bitacora?: string };
    contrato = j.contrato ?? "";
    examen = j.examen ?? "";
    insignias = j.insignias ?? "";
    bitacora = j.bitacora ?? "";
  }
} catch {
  /* sin archivo: aún no desplegada */
}
await pintarSaldo();
