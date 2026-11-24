// Open a Budi para Stellar workspace from the terminal.
//   pnpm launch                              starter project, IDE stays up
//   pnpm launch --repo https://github.com/you/your-soroban-repo
//   pnpm launch --testnet                    also deploy Bitácora to testnet
//   pnpm launch --check [--testnet]          smoke test: run everything, then close
//   pnpm launch close <session-id>
import { parseArgs } from "node:util";
import { closeSessionById, closeWorkspace, deployStarterToTestnet, openWorkspace, runChecks, startIde } from "./workspace.js";

const apiKey = process.env.TENKI_API_KEY;
const image = process.env.BUDI_STELLAR_IMAGE;
if (!apiKey) throw new Error("TENKI_API_KEY is not set");

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    repo: { type: "string" },
    testnet: { type: "boolean", default: false },
    check: { type: "boolean", default: false },
    minutes: { type: "string", default: "60" },
    hito: { type: "string", default: "Mi entorno Stellar en Budi" },
  },
});

if (positionals[0] === "close") {
  if (!positionals[1]) throw new Error("usage: pnpm launch close <session-id>");
  await closeSessionById(apiKey, positionals[1]);
  console.log(`closed ${positionals[1]}`);
  process.exit(0);
}
if (!image) throw new Error("BUDI_STELLAR_IMAGE is not set (run pnpm image:build)");

const minutes = Number.parseInt(values.minutes, 10);
const started = Date.now();
const log = (msg: string) => console.log(`[${((Date.now() - started) / 1000).toFixed(0)}s] ${msg}`);

const ws = await openWorkspace({ apiKey, image, repo: values.repo, maxMinutes: values.check ? 20 : minutes, log });
let failed = false;
try {
  for (const c of await runChecks(ws)) {
    log(`${c.ok ? "ok  " : "FAIL"} ${c.name}`);
    if (!c.ok || c.name === "toolchain") console.log(c.output.replace(/^/gm, "     "));
    failed ||= !c.ok;
  }
  if (values.testnet) {
    const d = await deployStarterToTestnet(ws, values.hito);
    log(`${d.ok ? "ok  " : "FAIL"} testnet deploy`);
    console.log(d.output.replace(/^/gm, "     "));
    if (d.contractId) console.log(`     https://stellar.expert/explorer/testnet/contract/${d.contractId}`);
    failed ||= !d.ok;
  }
  const ide = await startIde(ws, values.check ? 5 : minutes);
  log(`ok   IDE at ${ide.url}`);
  if (values.check) {
    const r = await fetch(ide.url, { redirect: "manual" });
    const reachable = r.status < 500;
    log(`${reachable ? "ok  " : "FAIL"} IDE reachable from outside (HTTP ${r.status})`);
    failed ||= !reachable;
  } else {
    console.log(`\n  IDE:        ${ide.url}\n  Contraseña: ${ide.password}\n  Sesión:     ${ws.sessionId} (se cierra sola en ${minutes} min)\n  Cerrar:     pnpm launch close ${ws.sessionId}\n`);
  }
} catch (err) {
  failed = true;
  console.error(err);
} finally {
  if (values.check || failed) {
    await closeWorkspace(ws);
    log(`session ${ws.sessionId} closed`);
  } else {
    ws.tenki.close();
  }
}
process.exitCode = failed ? 1 : 0;
