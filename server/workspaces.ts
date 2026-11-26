// One workspace per person, with a time limit. A workspace takes about a
// minute to open, so opening runs in the background and the aula polls.
// State is in memory: after a restart, open sessions still close on their own
// at Tenki's max duration, and the person opens a new one.
import { randomBytes } from "node:crypto";
import { closeSessionById, configureAgent, openWorkspace, startIde, type Workspace } from "../tenki/workspace.js";
import type { Cuenta } from "./llm.js";

export interface Config {
  tenkiApiKey: string;
  image: string;
  minutes: number;
  /** Without it, workspaces open with no agent. */
  agente?: { baseUrl: string; model: string; budgetUsd: number; instrucciones: string };
}

export type Estado = "abriendo" | "listo" | "error";

export interface Entrada {
  estado: Estado;
  /** Epoch ms when the workspace closes. */
  cierra: number;
  sessionId?: string;
  ide?: { url: string };
  error?: string;
  /** The agent's proxy token and what it has spent. */
  agente?: { token: string; cuenta: Cuenta };
}

/** What the aula gets back: no session handles or tokens, only what the person needs. */
export type Vista = Omit<Entrada, "sessionId" | "agente"> & { agente?: { gastado: number; tope: number } };

export class Workspaces {
  private porPersona = new Map<string, Entrada>();

  constructor(private cfg: Config) {}

  ver(persona: string): Vista | null {
    const e = this.vigente(persona);
    return e ? vista(e) : null;
  }

  /** Opens a workspace for this person, or returns the one they already have. */
  abrir(persona: string): Vista {
    const actual = this.vigente(persona);
    if (actual && actual.estado !== "error") return vista(actual);
    const e: Entrada = { estado: "abriendo", cierra: Date.now() + this.cfg.minutes * 60_000 };
    if (this.cfg.agente) {
      e.agente = { token: randomBytes(24).toString("base64url"), cuenta: { gastado: 0, tope: this.cfg.agente.budgetUsd } };
    }
    this.porPersona.set(persona, e);
    void this.arrancar(e);
    return vista(e);
  }

  async cerrar(persona: string): Promise<void> {
    const e = this.porPersona.get(persona);
    this.porPersona.delete(persona);
    if (e?.sessionId) await closeSessionById(this.cfg.tenkiApiKey, e.sessionId);
  }

  /** The budget behind a proxy token, while its workspace is open. */
  cuentaDe(token: string): Cuenta | undefined {
    for (const [persona, e] of this.porPersona) {
      if (e.agente?.token === token) return this.vigente(persona) ? e.agente.cuenta : undefined;
    }
    return undefined;
  }

  private vigente(persona: string): Entrada | undefined {
    const e = this.porPersona.get(persona);
    if (e && e.cierra <= Date.now()) {
      this.porPersona.delete(persona);
      return undefined;
    }
    return e;
  }

  private async arrancar(e: Entrada): Promise<void> {
    let ws: Workspace | undefined;
    try {
      ws = await openWorkspace({ apiKey: this.cfg.tenkiApiKey, image: this.cfg.image, maxMinutes: this.cfg.minutes });
      e.sessionId = ws.sessionId;
      if (this.cfg.agente && e.agente) {
        const { baseUrl, model, instrucciones } = this.cfg.agente;
        await configureAgent(ws, { baseUrl, model, instrucciones, token: e.agente.token });
      }
      e.ide = { url: (await startIde(ws, this.cfg.minutes, "none")).url };
      e.estado = "listo";
    } catch (err) {
      e.estado = "error";
      e.error = err instanceof Error ? err.message : String(err);
      if (ws) await ws.session.closeIfOpen().catch(() => {});
    } finally {
      // The session lives on in Tenki; this process only drops its connection.
      ws?.tenki.close();
    }
  }
}

function vista({ sessionId: _, agente, ...v }: Entrada): Vista {
  return agente ? { ...v, agente: { gastado: agente.cuenta.gastado, tope: agente.cuenta.tope } } : v;
}
