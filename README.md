# Budi para Stellar

**Lo difícil, fácil de aprender y ganar.** Budi para Stellar es educación nativa de IA, en español, para quien quiere aprender IA y blockchain: aprendes construyendo, tu agente Budi escribe el código y tú demuestras que entiendes los conceptos. Lo que construyes corre en Stellar.

**Demo (73 s):** https://youtu.be/KZla30awd5U · **Pruébalo:** https://stellar.budi.lat

Proyecto para Ship It: Builder Challenge (BAF × Stellar México, 23 al 27 de noviembre de 2026).

## Qué es

Los recursos para aprender Stellar existen, pero están dispersos y casi todos en inglés. Hoy, además, la IA escribe el código: lo que vale es entenderlo. Budi para Stellar junta las dos cosas en un aula:

1. **Aprende.** Cada lección es un explicador en español con sus fuentes oficiales, de qué es Stellar a tu primer contrato Soroban.
2. **Construye con tu agente.** Entras con tu cuenta de Budi y en menos de un minuto tienes tu espacio de trabajo en [Tenki](https://tenki.cloud): VS Code en el navegador con Rust, Stellar CLI y tu agente Budi adentro. Tú no escribes el código: le pides al agente lo que quieres y, antes de escribir una línea, te pregunta qué entiendes del concepto y anota tu respuesta como evidencia (`EVIDENCIA.md`).
3. **Queda en la red.** Las pruebas corren, el contrato se despliega en testnet y el aula lee tu avance directo de Stellar.

## Cómo se usa

1. Abre https://stellar.budi.lat y entra a la lección 1, «Tu primer contrato Soroban».
2. Lee el explicador y abre tu espacio de trabajo (botón Editor) con tu cuenta de Budi.
3. Pídele al agente el cambio que quieres en tu Bitácora, responde sus preguntas y déjalo escribir, probar y desplegar.
4. Pega la dirección de tu Bitácora en el aula: cuando tiene al menos un hito en testnet, la lección queda completa.

## Qué corre en Stellar

Contratos Soroban en `contracts/`, todos con pruebas:

- **Bitácora**: tus hitos de aprendizaje, firmados por ti (`registrar`, `corregir`, `hitos`). Puede pedir que hayas pagado el curso en Matrícula.
- **Matrícula**: tu primer pago en Stellar. Pagas el curso y queda en custodia; se te regresa cuando terminas.
- **Insignias**: la credencial en Stellar para quien termina una ruta (lo que sigue: que Budi valide lo que entendiste y la otorgue).
- **Reputación**: equipos de una persona y su agente que toman trabajos, entregan, cobran y acumulan puntaje por habilidad.
- **Bóveda**: el bono de bienvenida. Guarda XLM y se abre con un código. Corre en testnet (`/boveda/`) y en mainnet (`/boveda/mainnet/`, ver [docs/mainnet.md](docs/mainnet.md)).

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

En [stellar.budi.lat/aula](https://stellar.budi.lat/aula/) hay un escritorio con ventanas: la lección (un explicador animado), el ejercicio, tu Bitácora y el editor de tu workspace. La lección 1, «Tu primer contrato Soroban», termina cuando despliegas tu propia Bitácora en testnet y registras un hito. El aula lo revisa leyendo los eventos de tu contrato en la red: no tienes que mandarnos nada. El código está en `web/` (`pnpm web:dev`, `pnpm web:build`).

El editor: entras con tu cuenta de Budi, abres tu workspace y en menos de un minuto aparece VS Code de tu sesión dentro de la ventana. Tienes uno a la vez, con tiempo límite, y lo apagas desde la misma ventana. Lo abre el servicio de `server/` (`pnpm server` en local; en producción corre en Railway con el `Dockerfile` de la raíz), que revisa tu sesión de Budi y llama a `openWorkspace`. Tu proyecto trae solo lo de la lección: Bitácora (y Matrícula, que usan sus pruebas), Rust y Stellar CLI listos.

Tu agente Budi abre solo en la terminal del editor. Es [Pi](https://pi.dev) sobre un modelo de Nebius, con las instrucciones de `tenki/agente/APPEND_SYSTEM.md`: **el agente escribe todo el código y tú decides y explicas**. Antes de construir cada paso te pregunta qué quieres y por qué (quién puede llamar cada función, qué se guarda, qué puede salir mal), y anota tus respuestas en `EVIDENCIA.md`. Lo que se aprende aquí es arquitectura, seguridad y diseño, no a teclear código. El agente llama al modelo a través del servicio, con un token y un presupuesto por workspace: la llave de Nebius nunca entra a tu máquina.

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

## Un bono que se abre con un código: Bóveda

`contracts/boveda` guarda XLM y se los da a quien presente el código correcto. Budi la llena con `depositar`; quien escriba el código en https://stellar.budi.lat/boveda/ (o llame `reclamar`) se lleva todo lo que haya dentro, una vez por wallet. El contrato guarda solo el sha256 del código normalizado (minúsculas, sin espacios), nunca el código. Sirve como bono de bienvenida o premio para quien termina una lección. Nació en el reto exprés de Ship It del 26 de noviembre (5555 XLM de testnet, código «Ship It»).

```bash
cargo test -p boveda
stellar contract build --package boveda
```

La página es `web/boveda/` (`pnpm boveda:build`) y lee el id del contrato de `/boveda/contrato.json`.

## Reputación por habilidad: Reputación

Reputación guarda la reputación de un equipo humano + agente Budi, una por habilidad (por ejemplo, contratos Soroban). Las reglas siguen el modelo de los consejos médicos: la insignia es la licencia y el puntaje es el certificado que vence.

- **La insignia abre la habilidad.** Para ganar puntos en una habilidad, el humano del equipo necesita su insignia (contrato `insignias`). La insignia nunca suma puntos.
- **Solo cuenta el trabajo pagado y entregado.** Un cliente contrata al equipo y su pago queda en custodia. El equipo entrega con una prueba (un enlace o un hash). Cuando el cliente acepta, el pago pasa al humano y el equipo suma puntos.
- **El silencio no traba el pago.** Después de la entrega, el cliente tiene un plazo de revisión. Si no acepta ni responde, el humano cobra y el equipo suma como si hubiera aceptado.
- **El cliente puede disputar.** Dentro del plazo de revisión, el cliente puede rechazar la entrega con un motivo. El pago se congela y Budi decide, con un motivo público: le paga al humano (y el equipo suma) o le regresa el pago al cliente (y nadie suma).
- **Piso y tope.** Un trabajo por debajo del piso se paga pero no suma. Cada trabajo suma como máximo un tope. Las evaluaciones de Budi también suman, con un tope total para que no carguen el puntaje.
- **Los puntos vencen.** Sin trabajo reciente, el puntaje baja solo.
- **Anular es aparte de vencer.** Budi puede anular los puntos de un trabajo, con un motivo público.
- **El equipo es la pareja.** Budi forma el equipo y firman los tres. La reputación no se transfiere y un agente solo está en un equipo. Si el humano retira al agente, el historial se queda.

| Función | Qué hace |
| --- | --- |
| `definir_habilidad(habilidad, reglas)` | El admin (Budi) fija la insignia que la abre, el piso, el tope por trabajo, la vigencia, el tope de evaluaciones y el plazo de revisión del cliente. |
| `formar_equipo(humano, agente)` | Budi, el humano y el agente firman. Regresa el número de equipo. |
| `retirar(equipo)` | El humano retira a su agente: no más trabajos nuevos, el historial se queda. |
| `contratar(cliente, equipo, habilidad, monto, plazo)` | El cliente paga y el pago queda en custodia. Pide la insignia de la habilidad. |
| `entregar(trabajo, prueba)` | El humano entrega antes del plazo, con una prueba. |
| `aceptar(trabajo)` | El cliente acepta (o retira su disputa): el pago pasa al humano y el equipo suma puntos. |
| `cobrar_sin_respuesta(trabajo)` | Venció la revisión y el cliente no respondió: el humano cobra y el equipo suma puntos. |
| `disputar(trabajo, motivo)` | El cliente rechaza la entrega dentro del plazo de revisión; el pago se congela. |
| `resolver(trabajo, paga_al_equipo, motivo)` | Budi resuelve la disputa: cobra el humano y el equipo suma, o el cliente recupera su pago. |
| `reembolsar(trabajo)` | Venció el plazo sin entrega: el cliente recupera su pago. |
| `evaluar(equipo, habilidad, puntos)` | Budi suma puntos por una evaluación calificada, con tope. |
| `anular(trabajo, motivo)` | Budi anula los puntos de un trabajo, con motivo público. |
| `puntaje(equipo, habilidad)` | Puntos vigentes, trabajos que cuentan y puntos por evaluación. |

En testnet (25 de noviembre de 2026), con XLM: Reputación [`CCNHF6DU...SFCL`](https://stellar.expert/explorer/testnet/contract/CCNHF6DUBUDY5MG2AQYXZU4OHHG2NXWVIFDZQLG4Z3XDJ4N5KFDASFCL), con la habilidad `soroban` abierta por la insignia `soroban_101` de Insignias [`CDRJ6CXB...2QPAY`](https://stellar.expert/explorer/testnet/contract/CDRJ6CXBFDOUZBKHGOXCZ6W5TVCIUYIF4T5X6WSJ27ACQJRPYOX2QPAY). Dos trabajos del equipo 1 terminaron en disputa: en el primero (30 XLM) Budi le dio la razón al equipo, que cobró y sumó 3 puntos; en el segundo (20 XLM) se la dio al cliente, que recuperó su pago. El despliegue anterior [`CC4PRNDA...2AVK`](https://stellar.expert/explorer/testnet/contract/CC4PRNDAP4LH6EUFIBAEPY5BA2T4GMLPCEM546U5KTLOKDV6H2RQ2AVK) tiene el resto de las funciones en uso: contratar, entregar, aceptar, cobrar sin respuesta, reembolsar, evaluar, anular y retirar.

Pendiente: la ventana de apelación antes de anular, y quién firma por el agente cuando use una cuenta con passkey.

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
- [x] Abrir tu workspace desde el aula con tu cuenta de Budi (`server/` y la ventana Editor)

## Licencia y marca

El código es abierto bajo la [licencia MIT](LICENSE). La marca Budi (nombre, logo, personaje, voz y videos) no es open source: es de Poktalabs y no está en este repo. Detalles en [TRADEMARKS.md](TRADEMARKS.md).
