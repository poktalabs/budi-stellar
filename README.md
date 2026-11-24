# Budi para Stellar

Budi te da un entorno de desarrollo Stellar listo en minutos: una sesión en [Tenki](https://tenki.cloud) con Rust, Stellar CLI y tu repo, que tu agente Budi prepara y maneja desde la app. Sin instalar nada en tu compu: abres Budi, eliges tu proyecto y empiezas a escribir contratos Soroban.

Proyecto para Ship It: Builder Challenge (BAF × Stellar México, 23 al 27 de noviembre de 2026).

## Estado

En construcción. Ver la sección de avance abajo.

## El contrato de ejemplo: Bitácora

Cada builder registra sus hitos onchain. Es el contrato que Budi abre en cada sesión nueva: corto, pero con autorización, almacenamiento persistente, errores propios y eventos, para que el entorno pruebe el ciclo completo desde el primer minuto.

| Función | Qué hace |
| --- | --- |
| `registrar(builder, hito)` | Guarda un hito (máximo 140 bytes). Solo firma el propio builder. Regresa el número del hito. |
| `hitos(builder)` | Todos los hitos del builder, en orden. |
| `contar(builder)` | Cuántos hitos lleva. |

Para compilarlo y probarlo en tu compu (Rust se fija solo con `rust-toolchain.toml`; necesitas [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 30 o más):

```bash
cargo test
stellar contract build
```

Rust 1.91.0 no compila contratos (Stellar CLI lo bloquea), por eso el repo fija 1.92.0.

## Avance

- [x] Repo público
- [x] Contrato Soroban de ejemplo con pruebas (`contracts/bitacora`)
- [x] Imagen de Tenki con Rust, `wasm32v1-none`, Stellar CLI y VS Code en el navegador (`tenki/recipe.ts`), publicada como `dnai/budi-stellar-dev:main`
- [ ] Lanzador: abre una sesión, clona tu repo, compila y prueba tus contratos
- [ ] Integración con la app de Budi
