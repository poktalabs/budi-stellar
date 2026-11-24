// A Budi para Stellar workspace: one Tenki session from the published image,
// the builder's project in it, checks run, a browser IDE behind a password.
// Budi's API calls these functions; `launch.ts` is the same flow from a terminal.
import { randomBytes } from "node:crypto";
import { TenkiSandbox, stderrText, stdoutText, type Session } from "@tenkicloud/sandbox";
import { IDE_PORT, LAYOUT } from "./layout.js";

export interface OpenOptions {
  apiKey: string;
  image: string;
  /** Public https Git URL of the builder's project. Omit to start from the Bitácora starter. */
  repo?: string;
  /** Hard ceiling for the session; Tenki also pauses at ~2 h. */
  maxMinutes?: number;
  log?: (msg: string) => void;
}

export interface CheckResult {
  name: string;
  ok: boolean;
  output: string;
}

export interface Workspace {
  sessionId: string;
  session: Session;
  tenki: TenkiSandbox;
  fromStarter: boolean;
}

const sh = (cmd: string) => `. "$HOME/.cargo/env"; export PATH="$HOME/.local/bin:$PATH"; ${cmd}`;
const tail = (s: string, n = 1500) => (s.length > n ? `...${s.slice(-n)}` : s);

/** Only plain public https Git URLs: the value reaches git as an argument, never a shell. */
export function checkRepoUrl(repo: string): string {
  const url = new URL(repo);
  if (url.protocol !== "https:" || url.username || url.password) {
    throw new Error("repo must be a public https Git URL without credentials");
  }
  return url.toString();
}

async function run(s: Session, name: string, cmd: string, timeoutMs: number): Promise<CheckResult> {
  const r = await s.exec("bash", { args: ["-lc", sh(cmd)], cwd: LAYOUT.project, timeoutMs });
  return { name, ok: r.exitCode === 0, output: tail(`${stdoutText(r)}\n${stderrText(r)}`.trim()) };
}

export async function openWorkspace(opts: OpenOptions): Promise<Workspace> {
  const log = opts.log ?? (() => {});
  const repo = opts.repo ? checkRepoUrl(opts.repo) : undefined;
  const tenki = new TenkiSandbox({ authToken: opts.apiKey });
  log("starting session");
  let session: Session;
  try {
    session = await tenki.create({
      name: `budi-stellar-${Date.now()}`,
      image: opts.image,
      allowOutbound: true,
      cpuCores: 2,
      memoryMb: 4096,
      diskSizeGb: 20,
      maxDurationMs: (opts.maxMinutes ?? 60) * 60_000,
      tags: ["budi-stellar"],
      waitReady: true,
      waitTimeoutMs: 8 * 60_000,
    });
  } catch (err) {
    tenki.close();
    throw err;
  }
  log(`session ${session.id} ready`);
  try {
    if (repo) {
      log(`cloning ${repo}`);
      const r = await session.exec("git", {
        args: ["clone", "--depth", "1", repo, LAYOUT.project],
        cwd: LAYOUT.home,
        timeoutMs: 180_000,
      });
      if (r.exitCode !== 0) throw new Error(`git clone failed: ${stderrText(r)}`);
    } else {
      // The starter keeps its warm target dir, so the first build is quick.
      const r = await session.exec("cp", { args: ["-a", LAYOUT.starter, LAYOUT.project], timeoutMs: 120_000 });
      if (r.exitCode !== 0) throw new Error(`copy starter failed: ${stderrText(r)}`);
    }
    return { sessionId: session.id, session, tenki, fromStarter: !repo };
  } catch (err) {
    await session.closeIfOpen().catch(() => {});
    tenki.close();
    throw err;
  }
}

/** Toolchain versions, then `cargo test` and `stellar contract build` when the project is a Cargo workspace. */
export async function runChecks(ws: Workspace): Promise<CheckResult[]> {
  const results = [await run(ws.session, "toolchain", "rustc --version && stellar --version | head -1", 60_000)];
  const isCargo = await run(ws.session, "cargo project", "test -f Cargo.toml", 30_000);
  if (!isCargo.ok) return [...results, { ...isCargo, output: "no Cargo.toml at the repo root; skipped build and tests" }];
  results.push(await run(ws.session, "cargo test", "cargo test --quiet 2>&1", 15 * 60_000));
  results.push(await run(ws.session, "stellar contract build", "stellar contract build 2>&1", 15 * 60_000));
  return results;
}

/**
 * Starter only: a throwaway testnet identity funded by Friendbot, Bitácora
 * deployed, and one milestone registered and read back.
 */
export async function deployStarterToTestnet(ws: Workspace, hito: string): Promise<CheckResult & { contractId?: string }> {
  if (!ws.fromStarter) throw new Error("testnet deploy is wired for the Bitácora starter only");
  await ws.session.writeFile(`${LAYOUT.runDirRel}/hito.txt`, hito);
  const script = [
    "set -euo pipefail",
    "stellar keys generate budi --network testnet --fund --overwrite >/dev/null 2>&1",
    'ADDR="$(stellar keys address budi)"',
    "ID=\"$(stellar contract deploy --wasm target/wasm32v1-none/release/bitacora.wasm --source-account budi --network testnet --alias bitacora 2>/dev/null | tail -1)\"",
    'echo "contract $ID"',
    `stellar contract invoke --id "$ID" --source-account budi --network testnet -- registrar --builder "$ADDR" --hito "$(cat ${LAYOUT.runDir}/hito.txt)" 2>/dev/null`,
    'stellar contract invoke --id "$ID" --source-account budi --network testnet --send=no -- hitos --builder "$ADDR" 2>/dev/null',
  ].join("\n");
  const r = await run(ws.session, "testnet deploy", script, 5 * 60_000);
  const contractId = /contract (C[A-Z0-9]{55})/.exec(r.output)?.[1];
  return { ...r, ok: r.ok && Boolean(contractId), contractId };
}

/** code-server on the project, behind a fresh password, exposed as a preview URL. */
export async function startIde(ws: Workspace, ttlMinutes = 60): Promise<{ url: string; password: string }> {
  const password = randomBytes(18).toString("base64url");
  // setsid nohup so the IDE outlives this exec; the password reaches it only through the process env.
  const r = await ws.session.exec("bash", {
    args: [
      "-lc",
      sh(
        `mkdir -p ${LAYOUT.runDir} && setsid nohup code-server --bind-addr 0.0.0.0:${IDE_PORT} --auth password --disable-telemetry ${LAYOUT.project} < /dev/null > ${LAYOUT.runDir}/ide.log 2>&1 &`,
      ),
    ],
    env: { PASSWORD: password },
    timeoutMs: 30_000,
  });
  if (r.exitCode !== 0) throw new Error(`code-server did not start: ${stderrText(r)}`);
  const ready = await ws.session.exec("bash", {
    args: ["-lc", `for i in $(seq 1 60); do curl -fs -o /dev/null http://127.0.0.1:${IDE_PORT}/healthz && exit 0; sleep 1; done; tail -20 ${LAYOUT.runDir}/ide.log; exit 1`],
    timeoutMs: 90_000,
  });
  if (ready.exitCode !== 0) throw new Error(`code-server not healthy: ${stdoutText(ready)}`);
  const exposed = await ws.session.exposePort(IDE_PORT, { ttlMs: ttlMinutes * 60_000 });
  return { url: exposed.previewUrl, password };
}

export async function closeWorkspace(ws: Workspace): Promise<void> {
  try {
    await ws.session.closeIfOpen();
  } finally {
    ws.tenki.close();
  }
}

/** Close a session by id (from a later process, e.g. Budi's "Cerrar" button). */
export async function closeSessionById(apiKey: string, sessionId: string): Promise<void> {
  const tenki = new TenkiSandbox({ authToken: apiKey });
  try {
    await (await tenki.get(sessionId)).closeIfOpen();
  } catch (err) {
    if (!(err instanceof Error && ["SessionNotFoundError", "SessionTerminatedError"].includes(err.name))) throw err;
  } finally {
    tenki.close();
  }
}
