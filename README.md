# Budi para Stellar

Budi te enseña a construir en Stellar, te da dónde practicar sin instalar nada y convierte lo que construyes en reputación con la que te contratan, a ti y a tu agente Budi.

1. **Aprende.** Explicadores animados en español, de qué es Stellar a tu primer contrato Soroban, con sus fuentes oficiales.
2. **Practica.** Cada lección termina en tu Budi workspace: una sesión en [Tenki](https://tenki.cloud) con Rust, Stellar CLI y VS Code en el navegador, que tu agente Budi prepara desde la app. De cero a un contrato en testnet en 64 segundos.
3. **Demuestra.** Al terminar una ruta recibes una insignia en Stellar. Tu reputación en cada habilidad crece solo con trabajo real, y quien busca ayuda contrata a tu equipo: tú y tu agente Budi.

Lo construimos por etapas: primero aprender y practicar (este repo, hoy), después las insignias, después contratar equipos persona más agente, y al final empresas que capacitan y contratan con Budi.

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
| `configurar(admin, matricula, curso)` | Opcional, una sola vez: desde ahí solo registra quien pagó `curso` en Matrícula. |

Para compilarlo en tu compu (Rust se fija solo con `rust-toolchain.toml`; necesitas [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 30 o más):

```bash
cargo test
stellar contract build
```

## Pagar un curso: Matrícula

Matrícula guarda el pago de un curso en custodia (escrow) hasta que se entrega. El alumno paga al inscribirse y el dinero se queda en el contrato. Cuando Budi entrega el curso, el pago pasa a Budi. Si no lo entrega antes del plazo, el alumno recupera su dinero. Bitácora, una vez configurada, solo deja registrar hitos a quien pagó el curso.

| Función | Qué hace |
| --- | --- |
| `crear_curso(curso, precio, plazo)` | El admin (Budi) fija el precio y el plazo de entrega en segundos. |
| `inscribir(alumno, curso)` | El alumno paga y el pago queda en custodia. Firma el alumno. Regresa el número de inscripción. |
| `liberar(id)` | Curso entregado: el pago pasa al admin. Firma el admin. |
| `reembolsar(id)` | Venció el plazo sin entrega: el pago regresa al alumno. Firma el alumno. |
| `inscrito(alumno, curso)` | Si el alumno pagó el curso (en custodia o entregado). |

En testnet (24 de noviembre de 2026), con XLM: Matrícula [`CAUATYLP...YDAA`](https://stellar.expert/explorer/testnet/contract/CAUATYLPFCJTSFWKQQUC7LO5R5RIIPY7WXE6EEYZLMA7Z7HCW7LHYDAA) y una Bitácora que pide el curso `soroban_101` [`CDOI7II5...4AUQ`](https://stellar.expert/explorer/testnet/contract/CDOI7II5MEP5M765BOWH4LQQ45YBE4HHLH7CN2534UUZLMBDNFUB4AUQ). Sin pagar, `registrar` falla; después de `inscribir`, funciona.

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
- [x] Pagos en testnet desde la app: `pnpm launch pagar --a <G...> --monto 1 --memo TEXTO` (firma con `STELLAR_SECRET` en `.env`)
- [x] Pago del curso en custodia (`contracts/matricula`) y Bitácora solo para quien pagó
- [ ] Explicadores de Stellar hechos con el motor de Budi, servidos por su API (Budi los genera; este repo los enlaza)
- [ ] El agente Budi dentro de la sesión por MCP y skills, para que te ayude a escribir y probar contratos
- [ ] Integración con la app de Budi: un botón «Abrir mi entorno Stellar» que llama a `openWorkspace`

## Licencia y marca

El código es abierto bajo la [licencia MIT](LICENSE). La marca Budi (nombre, logo, personaje, voz y videos) no es open source: es de Poktalabs y no está en este repo. Detalles en [TRADEMARKS.md](TRADEMARKS.md).
