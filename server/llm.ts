// The model proxy for the learner's Budi agent. Pi in the workspace calls this
// with its workspace's token; we forward to Nebius with Budi's key, always on
// the same model, and stop once the workspace spends its budget. The Nebius
// key never enters the learner's machine; a leaked token is worth one budget.
export const NEBIUS_URL = "https://api.tokenfactory.nebius.com/v1/chat/completions";

/** USD per 1M tokens (Nebius public catalog), same table as Budi's Pi runs. */
const PRECIOS: Record<string, { input: number; output: number; cacheRead: number }> = {
  "MiniMaxAI/MiniMax-M3": { input: 0.3, output: 1.2, cacheRead: 0.06 },
  "deepseek-ai/DeepSeek-V4-Flash-0731": { input: 0.14, output: 0.28, cacheRead: 0.01 },
  "moonshotai/Kimi-K2.7-Code": { input: 0.95, output: 4, cacheRead: 0.15 },
};

export interface Usage {
  prompt_tokens?: number;
  completion_tokens?: number;
  prompt_tokens_details?: { cached_tokens?: number };
}

export function costo(model: string, u: Usage): number {
  const p = PRECIOS[model];
  if (!p) return 0;
  const cached = u.prompt_tokens_details?.cached_tokens ?? 0;
  const input = (u.prompt_tokens ?? 0) - cached;
  return (input * p.input + cached * p.cacheRead + (u.completion_tokens ?? 0) * p.output) / 1e6;
}

export function precioConocido(model: string): boolean {
  return model in PRECIOS;
}

export interface Cuenta {
  gastado: number;
  tope: number;
}

const errorOpenAI = (status: number, message: string) =>
  new Response(JSON.stringify({ error: { message, type: "budi_limit" } }), {
    status,
    headers: { "Content-Type": "application/json" },
  });

/**
 * Forwards one chat completion. Streaming responses pass through untouched
 * while we read the final usage chunk; plain responses are read for usage.
 */
export async function reenviar(body: Record<string, unknown>, cuenta: Cuenta, model: string, nebiusKey: string): Promise<Response> {
  if (cuenta.gastado >= cuenta.tope) {
    return errorOpenAI(402, "Tu agente usó todo el presupuesto de este workspace. Apágalo y abre otro para seguir.");
  }
  const stream = body.stream === true;
  const payload = { ...body, model, ...(stream ? { stream_options: { include_usage: true } } : {}) };
  const res = await fetch(NEBIUS_URL, {
    method: "POST",
    headers: { Authorization: `Bearer ${nebiusKey}`, "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  const headers = { "Content-Type": res.headers.get("Content-Type") ?? "application/json" };
  if (!res.ok || !res.body) return new Response(res.body, { status: res.status, headers });

  if (!stream) {
    const json = (await res.json()) as { usage?: Usage };
    if (json.usage) cuenta.gastado += costo(model, json.usage);
    return new Response(JSON.stringify(json), { status: res.status, headers });
  }

  // Nebius repeats a running usage total in many chunks of one stream, so we
  // keep the last one and count it once, when the stream ends.
  const decoder = new TextDecoder();
  let resto = "";
  let ultimo: Usage | undefined;
  const revisar = (l: string) => {
    if (!l.startsWith("data: ") || l.includes("[DONE]") || !l.includes('"usage"')) return;
    try {
      const u = (JSON.parse(l.slice(6)) as { usage?: Usage | null }).usage;
      if (u) ultimo = u;
    } catch {
      // a partial or non-JSON line: nothing to count
    }
  };
  const leer = new TransformStream<Uint8Array, Uint8Array>({
    transform(chunk, ctl) {
      ctl.enqueue(chunk);
      resto += decoder.decode(chunk, { stream: true });
      const lineas = resto.split("\n");
      resto = lineas.pop() ?? "";
      for (const l of lineas) revisar(l);
    },
    flush() {
      revisar(resto + decoder.decode());
      if (ultimo) cuenta.gastado += costo(model, ultimo);
    },
  });
  return new Response(res.body.pipeThrough(leer), { status: res.status, headers });
}
