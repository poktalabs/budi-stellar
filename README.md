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

## El aula: la lección 1

En [stellar.budi.lat/aula](https://stellar.budi.lat/aula/) hay un escritorio con ventanas: la lección (un explicador animado), el ejercicio, tu Bitácora y, pronto, el editor de tu workspace. La lección 1, «Tu primer contrato Soroban», termina cuando despliegas tu propia Bitácora en testnet y registras un hito. El aula lo revisa leyendo los eventos de tu contrato en la red: no tienes que mandarnos nada. El código está en `web/` (`pnpm web:dev`, `pnpm web:build`).

## Paga con Stellar y te lo devolvemos al terminar: Matrícula

Matrícula es un depósito de compromiso (skin in the game). El alumno deposita al inscribirse y el dinero se queda en el contrato. Si termina el curso a tiempo, Budi lo confirma y el depósito regresa completo al alumno. Si no termina antes del plazo, Budi se lo queda. Es un primer pago en Stellar que te regresa. Los probadores alfa entran gratis con un código de un solo uso. Bitácora, una vez configurada, solo deja registrar hitos a quien está inscrito.

Esta versión del depósito todavía no está en testnet: primero va el curso. La página [stellar.budi.lat/inscribete](https://stellar.budi.lat/inscribete/) usa la versión anterior (pago en custodia que pasaba a Budi al entregar).

| Función | Qué hace |
| --- | --- |
| `crear_curso(curso, precio, plazo)` | El admin (Budi) fija el depósito y el plazo en segundos. |
| `inscribir(alumno, curso)` | El alumno deposita y el depósito queda en custodia. Firma el alumno. Regresa el número de inscripción. |
| `crear_codigos(curso, hashes)` | El admin guarda códigos de acceso gratis de un solo uso (solo su sha256). |
| `canjear(alumno, codigo)` | El alumno entra con un código, sin depósito. El código se borra al usarse. |
| `completar(id)` | El alumno terminó a tiempo: el depósito regresa al alumno. Firma el admin. |
| `cobrar(id)` | Venció el plazo sin terminar: el depósito pasa al admin. Firma el admin. |
| `inscrito(alumno, curso)` | Si el alumno está en el curso (depósito en custodia o curso terminado). |

## Reputación por habilidad: Reputación

Reputación guarda la reputación de un equipo humano + agente Budi, una por habilidad (por ejemplo, contratos Soroban). Las reglas siguen el modelo de los consejos médicos: la insignia es la licencia y el puntaje es el certificado que vence.

- **La insignia abre la habilidad.** Para ganar puntos en una habilidad, el humano del equipo necesita su insignia (contrato `insignias`). La insignia nunca suma puntos.
- **Solo cuenta el trabajo pagado y entregado.** Un cliente contrata al equipo y su pago queda en custodia. El equipo entrega con una prueba (un enlace o un hash). Cuando el cliente acepta, el pago pasa al humano y el equipo suma puntos.
- **Piso y tope.** Un trabajo por debajo del piso se paga pero no suma. Cada trabajo suma como máximo un tope. Las evaluaciones de Budi también suman, con un tope total para que no carguen el puntaje.
- **Los puntos vencen.** Sin trabajo reciente, el puntaje baja solo.
- **Anular es aparte de vencer.** Budi puede anular los puntos de un trabajo, con un motivo público.
- **El equipo es la pareja.** Budi forma el equipo y firman los tres. La reputación no se transfiere y un agente solo está en un equipo. Si el humano retira al agente, el historial se queda.

| Función | Qué hace |
| --- | --- |
| `definir_habilidad(habilidad, reglas)` | El admin (Budi) fija la insignia que la abre, el piso, el tope por trabajo, la vigencia y el tope de evaluaciones. |
| `formar_equipo(humano, agente)` | Budi, el humano y el agente firman. Regresa el número de equipo. |
| `retirar(equipo)` | El humano retira a su agente: no más trabajos nuevos, el historial se queda. |
| `contratar(cliente, equipo, habilidad, monto, plazo)` | El cliente paga y el pago queda en custodia. Pide la insignia de la habilidad. |
| `entregar(trabajo, prueba)` | El humano entrega antes del plazo, con una prueba. |
| `aceptar(trabajo)` | El cliente acepta: el pago pasa al humano y el equipo suma puntos. |
| `reembolsar(trabajo)` | Venció el plazo sin entrega: el cliente recupera su pago. |
| `evaluar(equipo, habilidad, puntos)` | Budi suma puntos por una evaluación calificada, con tope. |
| `anular(trabajo, motivo)` | Budi anula los puntos de un trabajo, con motivo público. |
| `puntaje(equipo, habilidad)` | Puntos vigentes, trabajos que cuentan y puntos por evaluación. |

En testnet (24 de noviembre de 2026), con XLM: Reputación [`CAWQV7EU...A3HL`](https://stellar.expert/explorer/testnet/contract/CAWQV7EU66XI77WRG4E6COJUTGR3RCR3HRIK5SKDH7VYJEGXB2AXA3HL), con la habilidad `soroban` abierta por la insignia `soroban_101` de Insignias [`CDRJ6CXB...2QPAY`](https://stellar.expert/explorer/testnet/contract/CDRJ6CXBFDOUZBKHGOXCZ6W5TVCIUYIF4T5X6WSJ27ACQJRPYOX2QPAY). El equipo 1 hizo un trabajo de 30 XLM (3 puntos), recibió una evaluación (2 puntos) y un segundo trabajo se anuló en una auditoría de prueba: puntaje 5.

Pendiente: qué pasa si el cliente no responde después de una entrega (hoy el pago se queda en custodia), la ventana de apelación antes de anular, y quién firma por el agente cuando use una cuenta con passkey.

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
- [x] Depósito de compromiso que regresa al terminar el curso (`contracts/matricula`, falta desplegarlo) y Bitácora solo para inscritos
- [x] Página para inscribirse con un botón: [stellar.budi.lat/inscribete](https://stellar.budi.lat/inscribete/) (`web/`)
- [x] El aula con la lección 1: explicador, ejercicio y revisión leyendo la red (`web/aula`)
- [x] Insignias que no se transfieren (`contracts/insignias`)
- [x] Reputación por habilidad para equipos humano + agente (`contracts/reputacion`)
- [ ] Explicadores de Stellar hechos con el motor de Budi, servidos por su API (Budi los genera; este repo los enlaza)
- [ ] El agente Budi dentro de la sesión por MCP y skills, para que te ayude a escribir y probar contratos
- [ ] Integración con la app de Budi: un botón «Abrir mi entorno Stellar» que llama a `openWorkspace`

## Licencia y marca

El código es abierto bajo la [licencia MIT](LICENSE). La marca Budi (nombre, logo, personaje, voz y videos) no es open source: es de Poktalabs y no está en este repo. Detalles en [TRADEMARKS.md](TRADEMARKS.md).
