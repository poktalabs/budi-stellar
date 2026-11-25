import { conectar, existe, EXPLORER, fondear, hitos, inscribir, inscrito, precioXlm, registrar } from "./stellar";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const corta = (c: string) => `${c.slice(0, 4)}…${c.slice(-4)}`;
const enlace = (hash: string) => `<a href="${EXPLORER}/tx/${hash}" target="_blank" rel="noopener">ver la transacción</a>`;

let cuenta = "";

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

async function pintarHitos() {
  const lista = await hitos(cuenta);
  $("lista").innerHTML = lista.map((h) => `<li>${h.replace(/</g, "&lt;")}</li>`).join("");
}

async function abrirHito() {
  $("paso-hito").classList.remove("off");
  ($("hito") as HTMLInputElement).disabled = false;
  ($("registrar") as HTMLButtonElement).disabled = false;
  await pintarHitos();
}

async function revisar() {
  if (!(await existe(cuenta))) {
    $("fondear").hidden = false;
    decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>. Tu cuenta aún no existe en testnet: fondéala primero.`);
    return;
  }
  $("fondear").hidden = true;
  decir("out-wallet", `Conectada: <b>${corta(cuenta)}</b>.`);
  $("paso-inscribir").classList.remove("off");
  $("precio").textContent = `${await precioXlm(cuenta)} XLM`;
  if (await inscrito(cuenta)) {
    decir("out-inscribir", "Ya pagaste este curso. Tu pago está en custodia o el curso ya se entregó.");
    await abrirHito();
  } else {
    ($("inscribir") as HTMLButtonElement).disabled = false;
  }
}

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
  ocupado($("fondear"), "Fondeando…", async () => {
    try {
      await fondear(cuenta);
      await revisar();
    } catch (e) {
      decir("out-wallet", (e as Error).message, true);
    }
  }),
);

$("inscribir").addEventListener("click", () =>
  ocupado($("inscribir"), "Firma en Freighter…", async () => {
    try {
      const hash = await inscribir(cuenta);
      decir("out-inscribir", `Listo: tu pago quedó en custodia. ${enlace(hash)}`);
      await abrirHito();
    } catch (e) {
      decir("out-inscribir", (e as Error).message, true);
    }
  }).then(async () => {
    if (await inscrito(cuenta)) ($("inscribir") as HTMLButtonElement).disabled = true;
  }),
);

$("registrar").addEventListener("click", () =>
  ocupado($("registrar"), "Firma en Freighter…", async () => {
    const input = $("hito") as HTMLInputElement;
    const texto = input.value.trim();
    if (!texto) return decir("out-hito", "Escribe tu hito primero.", true);
    try {
      const hash = await registrar(cuenta, texto);
      input.value = "";
      decir("out-hito", `Hito registrado en Stellar. ${enlace(hash)}`);
      await pintarHitos();
    } catch (e) {
      decir("out-hito", (e as Error).message, true);
    }
  }),
);
