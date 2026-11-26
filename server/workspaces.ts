// One workspace per person, with a time limit. A workspace takes about a
// minute to open, so opening runs in the background and the aula polls.
// State is in memory: after a restart, open sessions still close on their own
// at Tenki's max duration, and the person opens a new one.
import { closeSessionById, openWorkspace, startIde, type Workspace } from "../tenki/workspace.js";

export interface Config {
  tenkiApiKey: string;
  image: string;
  minutes: number;
}

export type Estado = "abriendo" | "listo" | "error";

export interface Entrada {
  estado: Estado;
  /** Epoch ms when the workspace closes. */
  cierra: number;
  sessionId?: string;
  ide?: { url: string };
  error?: string;
}

/** What the aula gets back: no session handles, only what the person needs. */
export type Vista = Omit<Entrada, "sessionId">;

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
    this.porPersona.set(persona, e);
    void this.arrancar(e);
    return vista(e);
  }

  async cerrar(persona: string): Promise<void> {
    const e = this.porPersona.get(persona);
    this.porPersona.delete(persona);
    if (e?.sessionId) await closeSessionById(this.cfg.tenkiApiKey, e.sessionId);
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

function vista({ sessionId: _, ...v }: Entrada): Vista {
  return v;
}
