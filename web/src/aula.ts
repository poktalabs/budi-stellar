// El aula: un escritorio con ventanas (Lección, Editor, Ejercicio, Bitácora).
// La lección se completa cuando la Bitácora del alumno tiene al menos un hito en
// testnet; lo revisamos leyendo los eventos del contrato en la red.
import { rpc, scValToNative, StrKey } from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org";
const EXPLORER = "https://stellar.expert/explorer/testnet";
const server = new rpc.Server(RPC_URL);
const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

// --- ventanas ---------------------------------------------------------------
let z = 10;
const escritorio = $("escritorio");

function alFrente(w: HTMLElement) {
  w.style.zIndex = String(++z);
  document.querySelectorAll(".win").forEach((x) => x.classList.toggle("activa", x === w));
  document.querySelectorAll<HTMLButtonElement>(".dock button").forEach((b) =>
    b.classList.toggle("on", b.dataset.abrir === w.id && !w.hidden),
  );
}

function abrir(id: string) {
  const w = $(id);
  w.hidden = false;
  alFrente(w);
  if (window.innerWidth <= 820) w.scrollIntoView({ behavior: "smooth", block: "start" });
}

// Acomoda las ventanas al tamaño de la pantalla hasta que el alumno mueva alguna.
let movida = false;
function acomodar() {
  if (movida || window.innerWidth <= 820) return;
  const { width: W, height: H } = escritorio.getBoundingClientRect();
  const m = 20, alto = H - 2 * m;
  const leccion = Math.round((W - 3 * m) * 0.6);
  const poner = (id: string, x: number, y: number, w: number, h: number) =>
    Object.assign($(id).style, { left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px` });
  poner("w-leccion", m, m, leccion, alto);
  poner("w-ejercicio", 2 * m + leccion, m, W - 3 * m - leccion, alto);
  poner("w-editor", Math.round(W * 0.1), m * 2, Math.round(W * 0.8), Math.round(H * 0.8));
  // La Bitácora abre sobre la lección para no tapar el resultado del ejercicio.
  poner("w-bitacora", m * 3, m * 4, Math.min(560, leccion - 4 * m), Math.min(380, alto - 6 * m));
}
acomodar();
window.addEventListener("resize", acomodar);

for (const w of document.querySelectorAll<HTMLElement>(".win")) {
  w.addEventListener("pointerdown", () => alFrente(w));
  w.querySelector("[data-cerrar]")!.addEventListener("click", (e) => {
    e.stopPropagation();
    w.hidden = true;
    document.querySelector(`.dock [data-abrir="${w.id}"]`)?.classList.remove("on");
  });
  const barra = w.querySelector<HTMLElement>(".tbar")!;
  barra.addEventListener("pointerdown", (e) => {
    if ((e.target as HTMLElement).closest("button") || window.innerWidth <= 820) return;
    const r = w.getBoundingClientRect(), d = escritorio.getBoundingClientRect();
    const dx = e.clientX - r.left, dy = e.clientY - r.top;
    w.classList.add("arrastrando");
    movida = true;
    barra.setPointerCapture(e.pointerId);
    const mover = (ev: PointerEvent) => {
      const left = Math.min(Math.max(ev.clientX - d.left - dx, -r.width + 120), d.width - 120);
      const top = Math.min(Math.max(ev.clientY - d.top - dy, 0), d.height - 40);
      w.style.left = `${left}px`;
      w.style.top = `${top}px`;
    };
    const soltar = () => {
      w.classList.remove("arrastrando");
      barra.removeEventListener("pointermove", mover);
      barra.removeEventListener("pointerup", soltar);
    };
    barra.addEventListener("pointermove", mover);
    barra.addEventListener("pointerup", soltar);
  });
}
document.querySelectorAll<HTMLButtonElement>(".dock button").forEach((b) =>
  b.addEventListener("click", () => abrir(b.dataset.abrir!)),
);
$("estado-ws").addEventListener("click", () => abrir("w-editor"));
alFrente($("w-ejercicio"));

// --- copiar comandos ----------------------------------------------------------
for (const cmd of document.querySelectorAll<HTMLElement>(".cmd")) {
  const caja = document.createElement("div");
  caja.className = "cmdbox";
  cmd.replaceWith(caja);
  const copiar = document.createElement("button");
  copiar.className = "copiar";
  copiar.textContent = "Copiar";
  copiar.addEventListener("click", async () => {
    await navigator.clipboard.writeText(cmd.textContent ?? "");
    copiar.textContent = "Copiado";
    setTimeout(() => (copiar.textContent = "Copiar"), 1500);
  });
  caja.append(cmd, copiar);
}

// --- reloj ------------------------------------------------------------------
const reloj = () =>
  ($("reloj").textContent = new Date().toLocaleString("es-MX", { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" }));
reloj();
setInterval(reloj, 30_000);

// --- revisar la Bitácora en testnet -------------------------------------------
type Hito = { numero: number; hito: string; builder: string; ledger: number };

async function hitosDe(contrato: string): Promise<Hito[]> {
  const filtros: rpc.Api.EventFilter[] = [{ type: "contract", contractIds: [contrato] }];
  const { sequence } = await server.getLatestLedger();
  // El RPC guarda eventos de unos días y revisa un tramo de ledgers por llamada:
  // empezamos lo más atrás que se pueda y seguimos el cursor hasta el presente.
  let r: rpc.Api.GetEventsResponse | undefined;
  for (const atras of [120_000, 60_000, 17_000]) {
    try {
      r = await server.getEvents({ startLedger: Math.max(sequence - atras, 1), filters: filtros, limit: 100 });
      break;
    } catch (err) {
      if (!/startLedger|range/i.test(String(err))) throw err;
    }
  }
  if (!r) return [];
  const eventos = [...r.events];
  // ~10,000 ledgers por página: 15 páginas cubren los ~120,000 que guarda el RPC.
  for (let i = 0; i < 15 && r.cursor; i++) {
    r = await server.getEvents({ cursor: r.cursor, filters: filtros, limit: 100 });
    eventos.push(...r.events);
  }
  return eventos
    .filter((e) => scValToNative(e.topic[0]) === "hito_registrado")
    .map((e) => {
      const v = scValToNative(e.value) as { numero: number; hito: string };
      return { numero: Number(v.numero), hito: String(v.hito), builder: String(scValToNative(e.topic[1])), ledger: e.ledger };
    });
}

function pintarBitacora(contrato: string, lista: Hito[]) {
  $("bit-sub").textContent = `${contrato.slice(0, 6)}…${contrato.slice(-4)} · ${lista.length} hito${lista.length === 1 ? "" : "s"}`;
  $("bit-vacio").hidden = lista.length > 0;
  $("bit-lista").innerHTML = lista
    .map((h) => `<li><span class="num">#${h.numero}</span><span>${h.hito.replace(/</g, "&lt;")}</span><span class="meta">${h.builder.slice(0, 4)}…${h.builder.slice(-4)} · ledger ${h.ledger}</span></li>`)
    .join("");
}

$("revisar").addEventListener("click", async () => {
  const btn = $("revisar") as HTMLButtonElement;
  const contrato = ($("contrato") as HTMLInputElement).value.trim();
  const out = $("out-revisar");
  out.className = "out";
  if (!StrKey.isValidContract(contrato)) {
    out.className = "out err";
    out.textContent = "Esa no es una dirección de contrato. Empieza con C y tiene 56 caracteres.";
    return;
  }
  btn.disabled = true;
  btn.textContent = "Leyendo la red…";
  try {
    const lista = await hitosDe(contrato);
    pintarBitacora(contrato, lista);
    if (lista.length > 0) {
      ["p1", "p2", "p3"].forEach((p) => $(p).classList.add("hecho"));
      out.className = "out ok";
      out.innerHTML = `<b>¡Lección completa!</b> Tu Bitácora tiene ${lista.length} hito${lista.length === 1 ? "" : "s"} en testnet. <a href="${EXPLORER}/contract/${contrato}" target="_blank" rel="noopener">Ver en stellar.expert</a>`;
      localStorage.setItem("budi-stellar:leccion-1", contrato);
      abrir("w-bitacora");
      out.scrollIntoView({ block: "nearest" });
    } else {
      out.className = "out err";
      out.textContent = "Encontramos el contrato pero todavía no tiene hitos recientes. Haz el paso 3 y vuelve a revisar.";
    }
  } catch {
    out.className = "out err";
    out.textContent = "No pudimos leer la red. Revisa la dirección e intenta de nuevo.";
  } finally {
    btn.disabled = false;
    btn.textContent = "Revisar";
  }
});

const previo = localStorage.getItem("budi-stellar:leccion-1");
if (previo) ($("contrato") as HTMLInputElement).value = previo;
