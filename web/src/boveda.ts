// La bóveda: un bono de testnet que se abre con un código. El id del contrato se lee de
// /boveda/contrato.json al cargar, para poder publicar la página antes de desplegar.
import { Buffer } from "buffer";
import { Address, xdr } from "@stellar/stellar-sdk";
import { conectar, escribir, existe, EXPLORER, fondear, leer } from "./stellar";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const corta = (c: string) => `${c.slice(0, 4)}…${c.slice(-4)}`;
const enlace = (hash: string) => `<a href="${EXPLORER}/tx/${hash}" target="_blank" rel="noopener">ver la transacción</a>`;
const xlm = (stroops: bigint | number) =>
  `${(Number(stroops) / 10_000_000).toLocaleString("es-MX", { maximumFractionDigits: 2 })} XLM`;

// Cuenta pública solo para leer el saldo sin wallet (simulación, nunca firma).
const LECTOR = "GAH2VRAMEGQS3LBKVXVFIHZT7X4UB34BXW3AJZEB6XJCKSDPYO4YMH6F"; // la cuenta de Budi que llena la bóveda

let cuenta = "";
let contrato = (import.meta.env.VITE_BOVEDA as string | undefined) ?? "";

function decir(id: string, html: string, error = false) {
  const el = $(id);
  el.innerHTML = html;
  el.classList.toggle("err", error);
}

async function ocupado(btn: HTMLButtonElement, texto: string, fn: () => Promise<void>) {
  const antes = btn.textContent;
  btn.disabled = true;
  btn.textContent = texto;
  try {
    await fn();
  } finally {
    btn.textContent = antes;
    btn.disabled = false;
  }
}

async function cargarContrato() {
  try {
    const r = await fetch("/boveda/contrato.json", { cache: "no-store" });
    if (r.ok) contrato = ((await r.json()) as { contrato?: string }).contrato ?? contrato;
  } catch {
    /* sin archivo: se queda el de build */
  }
}

async function pintarSaldo() {
  if (!contrato) {
    $("saldo").textContent = "5555 XLM";
    decir("out-boveda", "La bóveda abre hoy a las 12:00 (CDMX). Vuelve en un rato.");
    return;
  }
  $("contrato").innerHTML = `<a href="${EXPLORER}/contract/${contrato}" target="_blank" rel="noopener">Ver el contrato en stellar.expert</a>. `;
  try {
    const s = (await leer(cuenta || LECTOR, contrato, "saldo", [])) as bigint;
    $("saldo").textContent = xlm(s);
    decir("out-boveda", s > 0n ? "" : "La bóveda está vacía por ahora: alguien ya la abrió. Vuelve en un minuto, la rellenamos.");
  } catch {
    $("saldo").textContent = "5555 XLM";
  }
}

async function revisar() {
  if (!(await existe(cuenta))) {
    $("fondear").hidden = false;
    decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>. Tu cuenta aún no existe en testnet: actívala primero.`);
    return;
  }
  $("fondear").hidden = true;
  decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>.`);
  if (!contrato) return;
  $("paso-reclamar").classList.remove("off");
  const ya = (await leer(cuenta, contrato, "reclamado", [new Address(cuenta).toScVal()])) as bigint;
  if (ya > 0n) {
    decir("out-reclamar", `Esta wallet ya abrió la bóveda y se llevó ${xlm(ya)}.`);
    return;
  }
  ($("codigo") as HTMLInputElement).disabled = false;
  ($("reclamar") as HTMLButtonElement).disabled = false;
}

const ERRORES: Record<string, string> = {
  "2": "Ese código no abre la bóveda. Revísalo y vuelve a intentar.",
  "3": "Esta wallet ya abrió la bóveda una vez.",
  "4": "La bóveda está vacía por ahora. Vuelve en un minuto.",
};

$("conectar").addEventListener("click", () =>
  ocupado($("conectar"), "Conectando…", async () => {
    try {
      cuenta = await conectar();
      $("conectar").textContent = "Wallet conectada";
      await revisar();
    } catch (e) {
      decir("out-wallet", (e as Error).message, true);
    }
  }),
);

$("fondear").addEventListener("click", () =>
  ocupado($("fondear"), "Activando…", async () => {
    try {
      await fondear(cuenta);
      await revisar();
    } catch (e) {
      decir("out-wallet", (e as Error).message, true);
    }
  }),
);

$("reclamar").addEventListener("click", () =>
  ocupado($("reclamar"), "Firma en Freighter…", async () => {
    const texto = ($("codigo") as HTMLInputElement).value;
    if (!texto.trim()) return decir("out-reclamar", "Escribe el código primero.", true);
    try {
      const antes = (await leer(cuenta, contrato, "saldo", [])) as bigint;
      const codigo = xdr.ScVal.scvBytes(Buffer.from(texto, "utf8"));
      const hash = await escribir(cuenta, contrato, "reclamar", [new Address(cuenta).toScVal(), codigo]);
      decir("out-reclamar", `¡Abriste la bóveda! ${xlm(antes)} van a tu wallet. ${enlace(hash)}`);
      ($("codigo") as HTMLInputElement).disabled = true;
      await pintarSaldo();
    } catch (e) {
      const m = (e as Error).message;
      const n = /Error\(Contract, #(\d+)\)/.exec(m)?.[1];
      decir("out-reclamar", (n && ERRORES[n]) || m, true);
    }
  }).then(async () => {
    if ($("codigo").hasAttribute("disabled")) ($("reclamar") as HTMLButtonElement).disabled = true;
  }),
);

await cargarContrato();
await pintarSaldo();
