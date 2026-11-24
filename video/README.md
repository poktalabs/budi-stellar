# Videos de Budi para Stellar

Videos para X, hechos con el motor del video de lanzamiento de buddi-sage (el kit de Remotion en `src/kit`) y la marca de Budi (`src/brand`, copiada de `packages/design-system` del monorepo de Budi; tipografía y colores de su DESIGN.md).

```bash
pnpm install --ignore-workspace
pnpm dev       # Remotion Studio
pnpm render    # out/ship-it-dia-1.mp4 (1920x1080, 39.5 s, con voz y música)
```

`ShipItDia1` cuenta el primer día de Ship It: el problema (diez pestañas para aprender Stellar), el camino de Budi, la corrida real del workspace (cada línea y tiempo sale del log del 23 de noviembre de 2026), el contrato Bitácora en testnet y lo que llega esta semana.

## Audio

- **Voz**: el clon de voz de Mel en ElevenLabs (el mismo de Buddi Sage), `eleven_v4`. `ELEVENLABS_API_KEY=... node scripts/generate-voiceover.mjs` genera un clip por escena y escribe sus duraciones en `src/voiceover.json`; cada escena se ajusta sola a su clip. La llave necesita el permiso Text to Speech.
- **Música**: `midnight-funk`, la cama del video de lanzamiento de Sage, con la misma mezcla (0.15, entrada de 0.4 s, salida de 2 s). No está en el repo: cópiala de `sage-launch-video/public/music/` a `public/music/`.
- **Efectos**: `paste` y `card`, los del corte de Sage.
