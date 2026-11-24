// Narration in Mel's own ElevenLabs voice clone (the one Buddi Sage uses), one clip per scene,
// silence-trimmed, durations written to src/voiceover.json so scene lengths fit the clips.
// Usage: ELEVENLABS_API_KEY=... node scripts/generate-voiceover.mjs   (ONLY=hook regenerates one)
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";

const VOICE_ID = "HTqwCT7d7eBsFDIsuiy0"; // Mel's clone (sage-owner-f356f484); never delete
const MODEL = "eleven_v4";
const OUT = "public/voiceover";
const JSON_PATH = "src/voiceover.json";

export const LINES = {
  hook: "¿Quieres construir en Stellar? Hoy empiezas con diez pestañas abiertas.",
  path: "Budi las convierte en un camino: cada lección termina en un ejercicio.",
  workspace: "Abres tu workspace en Tenki, con Rust y Stellar CLI listos. En sesenta y cuatro segundos, tus pruebas pasan y tu contrato está en testnet.",
  chain: "Y tu primer hito queda registrado onchain.",
  next: "Esta semana llega la ruta completa, en español: explicadores, ejercicios, y Budi como tu guía.",
  end: "Budi para Stellar. Aprende Stellar construyendo.",
};

const key = process.env.ELEVENLABS_API_KEY;
if (!key) throw new Error("ELEVENLABS_API_KEY is not set");
mkdirSync(OUT, { recursive: true });
const durations = existsSync(JSON_PATH) ? JSON.parse(readFileSync(JSON_PATH, "utf8")) : {};
const only = process.env.ONLY?.split(",");

for (const [id, text] of Object.entries(LINES)) {
  if (only && !only.includes(id)) continue;
  const res = await fetch(`https://api.elevenlabs.io/v1/text-to-speech/${VOICE_ID}?output_format=mp3_44100_128`, {
    method: "POST",
    headers: { "xi-api-key": key, "Content-Type": "application/json" },
    body: JSON.stringify({ model_id: MODEL, text, language_code: "es" }),
  });
  if (!res.ok) throw new Error(`${id}: ${res.status} ${await res.text()}`);
  const raw = `${OUT}/${id}.raw.mp3`;
  writeFileSync(raw, Buffer.from(await res.arrayBuffer()));
  const trim = "silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.05";
  execFileSync("ffmpeg", ["-loglevel", "error", "-y", "-i", raw, "-af", `${trim},areverse,${trim},areverse`, "-b:a", "192k", `${OUT}/${id}.mp3`]);
  execFileSync("rm", [raw]);
  const d = Number(execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", `${OUT}/${id}.mp3`]).toString().trim());
  durations[id] = Math.round(d * 1000) / 1000;
  console.log(`${id}: ${durations[id]}s`);
}
writeFileSync(JSON_PATH, JSON.stringify(durations, null, 2) + "\n");
