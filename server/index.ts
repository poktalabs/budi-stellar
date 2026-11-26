// Budi para Stellar service: opens a person's workspace behind Budi sign-in.
//   GET    /workspace   the person's workspace, or null
//   POST   /workspace   open one (or get the open one)
//   DELETE /workspace   close it
//   POST   /llm/v1/chat/completions   the agent's model proxy (workspace token)
// Run: pnpm server (reads .env). Env: TENKI_API_KEY, BUDI_STELLAR_IMAGE,
// PRIVY_APP_ID, ORIGINS (comma-separated), WORKSPACE_MINUTES, PORT; for the
// agent: NEBIUS_API_KEY, PUBLIC_URL (this service, reachable from Tenki),
// AGENT_MODEL, AGENT_BUDGET_USD.
import { readFileSync } from "node:fs";
import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { precioConocido, reenviar } from "./llm.js";
import { privyVerifier } from "./privy.js";
import { Workspaces } from "./workspaces.js";

const req = (k: string) => {
  const v = process.env[k];
  if (!v) throw new Error(`${k} is not set`);
  return v;
};

const verify = privyVerifier(req("PRIVY_APP_ID"));
const nebiusKey = process.env.NEBIUS_API_KEY;
const publicUrl = process.env.PUBLIC_URL?.replace(/\/$/, "");
const model = process.env.AGENT_MODEL ?? "MiniMaxAI/MiniMax-M3";
if (!precioConocido(model)) throw new Error(`AGENT_MODEL ${model} has no price, so its budget can't be enforced`);
const workspaces = new Workspaces({
  tenkiApiKey: req("TENKI_API_KEY"),
  image: req("BUDI_STELLAR_IMAGE"),
  minutes: Number(process.env.WORKSPACE_MINUTES ?? 45),
  agente:
    nebiusKey && publicUrl
      ? {
          baseUrl: `${publicUrl}/llm/v1`,
          model,
          budgetUsd: Number(process.env.AGENT_BUDGET_USD ?? 1),
          instrucciones: readFileSync(new URL("../tenki/agente/APPEND_SYSTEM.md", import.meta.url), "utf8"),
        }
      : undefined,
});
if (!nebiusKey || !publicUrl) console.warn("NEBIUS_API_KEY or PUBLIC_URL missing: workspaces open without the agent");
const origins = (process.env.ORIGINS ?? "https://stellar.budi.lat,http://localhost:5173").split(",");

const app = new Hono<{ Variables: { persona: string } }>();
app.use("*", cors({ origin: origins, allowHeaders: ["Authorization", "Content-Type"], allowMethods: ["GET", "POST", "DELETE"] }));
app.get("/", (c) => c.text("budi-stellar"));

app.use("/workspace", async (c, next) => {
  const token = c.req.header("Authorization")?.replace(/^Bearer /, "");
  if (!token) return c.json({ error: "inicia sesión con tu cuenta de Budi" }, 401);
  try {
    c.set("persona", await verify(token));
  } catch {
    return c.json({ error: "tu sesión venció, vuelve a iniciar sesión" }, 401);
  }
  await next();
});

app.get("/workspace", (c) => c.json(workspaces.ver(c.get("persona"))));
app.post("/workspace", (c) => c.json(workspaces.abrir(c.get("persona"))));
app.delete("/workspace", async (c) => {
  await workspaces.cerrar(c.get("persona"));
  return c.json(null);
});

app.post("/llm/v1/chat/completions", async (c) => {
  const token = c.req.header("Authorization")?.replace(/^Bearer /, "") ?? "";
  const cuenta = nebiusKey ? workspaces.cuentaDe(token) : undefined;
  if (!cuenta) return c.json({ error: { message: "este workspace ya no está abierto" } }, 401);
  return reenviar(await c.req.json(), cuenta, model, nebiusKey!);
});

const port = Number(process.env.PORT ?? 8787);
serve({ fetch: app.fetch, port }, () => console.log(`budi-stellar service on :${port}`));
