# Budi para Stellar

Budi te da un entorno de desarrollo Stellar listo en minutos: una sesión en [Tenki](https://tenki.cloud) con Rust, Stellar CLI y tu repo, que tu agente Budi prepara y maneja desde la app. Sin instalar nada en tu compu: abres Budi, eliges tu proyecto y empiezas a escribir contratos Soroban.

Proyecto para Ship It: Builder Challenge (BAF × Stellar México, 23 al 27 de noviembre de 2026).

## Qué vamos a lanzar el 27 de noviembre

Budi como la plataforma para aprender a construir en Stellar. Hoy los recursos existen pero están dispersos (Stellar Learn, la documentación, Stellar Quest, Stellar Lab, el Community Fund); Budi los convierte en un camino guiado. El 27 lanzamos:

1. **Una ruta de aprendizaje en español**: explicadores animados que van de qué es Stellar a tu primer contrato Soroban, cada uno con sus fuentes oficiales.
2. **Tu Budi workspace**: cada lección termina en un ejercicio que abres en un entorno listo en Tenki (Rust, Stellar CLI y VS Code en el navegador), sin instalar nada. Ya funciona: ver «Cómo funciona».
3. **Tu agente Budi como guía**: te acompaña de la primera lección a desplegar tu primer contrato en testnet, y te conecta con lo que sigue (Stellar Quest, Stellar Community Fund, Stellar APEX).
4. **Un demo de dos minutos** con el recorrido completo: explicador, workspace y contrato en testnet.

## Cómo se conecta Budi

Budi para Stellar es el primer cliente de la plataforma de Budi. Este repo es abierto (licencia MIT) y tiene lo que cualquiera puede correr: el contrato de ejemplo, la imagen de Tenki y el lanzador. Lo que hace especial a Budi, el motor de explicadores y la extensión, se queda en Budi y llega aquí por sus skills, su servidor MCP y su API. Este repo solo guarda la forma de conectarse, nunca su código.

## Cómo funciona

1. **La imagen** (`tenki/recipe.ts`): Ubuntu con Rust 1.92 y el target `wasm32v1-none`, Stellar CLI 30 y VS Code en el navegador (code-server). Trae este repo adentro, ya compilado una vez, así que tu primera compilación es rápida. Está publicada como imagen pública de Tenki: `dnai/budi-stellar-dev:main`.
2. **La sesión** (`tenki/workspace.ts`): Budi arranca una micro VM con esa imagen, clona tu repo (o copia el contrato de ejemplo), corre `cargo test` y `stellar contract build`, y abre VS Code detrás de una contraseña nueva en una URL de Tenki.
3. **Testnet**: con el contrato de ejemplo, crea una llave desechable, la fondea con Friendbot, despliega el contrato y registra un primer hito, todo dentro de la sesión.
4. **Al terminar**: la sesión se cierra sola al llegar a su tope de minutos, o antes con `pnpm launch close <id>`.

Primera corrida real (23 de noviembre de 2026): todo en 64 segundos, contrato en testnet [`CCMAFZJH...DXSOP`](https://stellar.expert/explorer/testnet/contract/CCMAFZJHAC2VY2UCYG23QB5K6MZPBDOB4JZG5QIKMM7W7T22ZUKDXSOP).

## Úsalo desde la terminal

Necesitas Node 22, pnpm y una llave de API de Tenki (el plan gratis alcanza: la imagen pide 2 vCPU y 4 GiB).

```bash
pnpm install
cp .env.example .env        # pon tu TENKI_API_KEY
pnpm launch                 # contrato de ejemplo, VS Code por 60 min
pnpm launch --repo https://github.com/tu-usuario/tu-repo-soroban
pnpm launch --testnet       # además despliega Bitácora en testnet
pnpm launch --check --testnet   # prueba todo y cierra la sesión
pnpm launch close <id-de-sesión>
```

`pnpm launch` imprime la URL de VS Code, la contraseña y el id de la sesión.

## El contrato de ejemplo: Bitácora

Cada builder registra sus hitos onchain. Es el contrato que Budi abre en cada sesión nueva: corto, pero con autorización, almacenamiento persistente, errores propios y eventos, para que el entorno pruebe el ciclo completo desde el primer minuto.

| Función | Qué hace |
| --- | --- |
| `registrar(builder, hito)` | Guarda un hito (máximo 140 bytes, hasta 100 por builder). Solo firma el propio builder. Regresa el número del hito. |
| `hitos(builder)` | Todos los hitos del builder, en orden. |
| `contar(builder)` | Cuántos hitos lleva. |

Para compilarlo en tu compu (Rust se fija solo con `rust-toolchain.toml`; necesitas [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 30 o más):

```bash
cargo test
stellar contract build
```

## Reconstruir la imagen

```bash
pnpm image:build    # construye con este repo en origin/main y publica dnai/budi-stellar-dev:main
```

Imprime el `BUDI_STELLAR_IMAGE` nuevo para tu `.env`.

## Lo que aprendimos al construirlo

- Rust 1.91.0 no compila contratos: Stellar CLI lo bloquea. Por eso el repo fija 1.92.0.
- El linker que trae Rust (rust-lld) truena en la VM de Tenki; `.cargo/config.toml` usa el linker del sistema en Linux x86_64.
- Los recursos de una plantilla de Tenki son también el mínimo de cada sesión: con 4 vCPU las cuentas gratis no pueden arrancarla.

## Avance

- [x] Repo público
- [x] Contrato Soroban de ejemplo con pruebas (`contracts/bitacora`)
- [x] Imagen de Tenki con Rust, Stellar CLI y VS Code en el navegador
- [x] Lanzador: sesión, tu repo, pruebas, VS Code y despliegue a testnet
- [ ] Explicadores de Stellar hechos con el motor de Budi, servidos por su API (Budi los genera; este repo los enlaza)
- [ ] El agente Budi dentro de la sesión por MCP y skills, para que te ayude a escribir y probar contratos
- [ ] Integración con la app de Budi: un botón «Abrir mi entorno Stellar» que llama a `openWorkspace`

## Licencia y marca

El código es abierto bajo la [licencia MIT](LICENSE). La marca Budi (nombre, logo, personaje, voz y videos) no es open source: es de Poktalabs y no está en este repo. Detalles en [TRADEMARKS.md](TRADEMARKS.md).
