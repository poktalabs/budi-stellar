# Budi para Stellar en mainnet

En la red principal de Stellar corren cuatro contratos de Budi para Stellar, y una sola página los usa: https://stellar.budi.lat/boveda/mainnet/ (pantalla `web/boveda/mainnet/index.html`, lógica `web/src/boveda-mainnet.ts`).

| Contrato | Qué hace | Dirección en mainnet |
| --- | --- | --- |
| Bóveda (`contracts/boveda`) | El bono de bienvenida: guarda XLM y se abre con un código (solo guarda su sha256). | [`CDT2SSF7…V547G24N`](https://stellar.expert/explorer/public/contract/CDT2SSF7IM5UDNZHFOAKUFAQUG5UQSZN4JORYOKOW2RJXKK4V547G24N) |
| Examen (`contracts/examen`) | El quiz de la lección 1: revisa las respuestas (solo guarda su huella) y, si pasas, te otorga la insignia. | [`CC5NAMY4…464UKV4FI`](https://stellar.expert/explorer/public/contract/CC5NAMY4QKTV7OMSKQVM4KOX2IAYNM6VJ7KOBGAWGIA2R3X464UKV4FI) |
| Bitácora (`contracts/bitacora`) | Tus hitos de aprendizaje, firmados por ti; puedes corregirlos y la corrección queda a la vista. | [`CCC2GF7E…XPXTB7Y3`](https://stellar.expert/explorer/public/contract/CCC2GF7E5A3KEH5KDFWIR5HOWKETNZH62QQR7SITOGEARE2VXPXTB7Y3) |
| Insignias (`contracts/insignias`) | Las insignias: NFT que no se transfieren. Su único emisor es Examen, así que solo se ganan pasando el quiz. | [`CATZGDIX…YH4IPAJ4B`](https://stellar.expert/explorer/public/contract/CATZGDIXUR5BJUCJRQFAC6B7K4O7ASJFCIJYNEFU7ESDBWYYH4IPAJ4B) |

La Bóveda usa el XLM nativo (Stellar Asset Contract `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA`). La página lee los ids de `/boveda/mainnet/contrato.json`.

Todas las llamadas salen del frontend: cada botón arma la transacción con `escribir()` (`web/src/stellar.ts`), la firma la wallet del usuario en Freighter (Mainnet) y se envía por RPC. `usarMainnet()` cambia la página a la red principal (`web/src/boveda-mainnet.ts:9`).

## Las 5 funciones

### 1. `depositar(de, monto)` · Bóveda

Mueve `monto` XLM de la wallet `de` a la bóveda y regresa el saldo nuevo. Pide la firma de `de`.

- Contrato: `contracts/boveda/src/lib.rs:66`
- Pantalla: paso 2, «Llena la bóveda», botón «Depositar XLM» (`web/boveda/mainnet/index.html:43`)
- Llamada: `web/src/boveda-mainnet.ts:100`
- Invocación en mainnet: [b8d9a61f…](https://stellar.expert/explorer/public/tx/b8d9a61f2ee79be3d116af75e126c8c36ee6f67137c90a8dc4e2525c961bfa81)

### 2. `reclamar(quien, codigo)` · Bóveda

Si el código es el correcto, manda todo el saldo a `quien`. Una vez por wallet. Pide la firma de `quien`.

- Contrato: `contracts/boveda/src/lib.rs:78`
- Pantalla: paso 3, «Ábrela con el código», botón «Abrir la bóveda» (`web/boveda/mainnet/index.html:56`)
- Llamada: `web/src/boveda-mainnet.ts:115`
- Invocación en mainnet: [59adc4a8…](https://stellar.expert/explorer/public/tx/59adc4a8a44b95f29d344c955be3b86ef8f3c836812acfefe5aa9bc21629902a)

### 3. `presentar(alumno, respuestas)` · Examen

El alumno firma sus respuestas al quiz. El contrato compara su sha256 con la huella guardada al desplegar; si las 3 son correctas, llama a `otorgar` en Insignias (`contracts/insignias/src/lib.rs:88`), que acuña la insignia a nombre del alumno, y regresa su id. Una insignia por persona; si falla puede reintentar.

- Contrato: `contracts/examen/src/lib.rs:68`
- Pantalla: paso 5, «Tu examen de la lección 1 y tu insignia», botón «Presentar examen» (`web/boveda/mainnet/index.html:99`)
- Llamada: `web/src/boveda-mainnet.ts:170`
- Invocación en mainnet: [c19ea6a7…](https://stellar.expert/explorer/public/tx/c19ea6a7c764a6ec220f8f92bbcf008d9be234c5aace482d14174723ea74949d) (insignia #1 otorgada)

### 4. `registrar(builder, hito)` · Bitácora

Guarda un hito de aprendizaje a nombre de `builder` y regresa su número. Solo el propio builder firma; el texto pasa por reglas (no vacío, máximo 140 caracteres) y se publica un evento.

- Contrato: `contracts/bitacora/src/lib.rs:106`
- Pantalla: paso 6, «Registra tu hito y corrígelo», botón «Registrar hito» (`web/boveda/mainnet/index.html:112`)
- Llamada: `web/src/boveda-mainnet.ts:215`
- Invocación en mainnet: [af4dd342…](https://stellar.expert/explorer/public/tx/af4dd342e77cc7321c30ff2fb29dfde0187ae8698667fcd0837eb68c8a80414e)

### 5. `corregir(builder, numero, hito)` · Bitácora

Corrige un hito ya registrado. Solo el propio builder firma; el texto nuevo pasa las mismas reglas y el evento `HitoCorregido` guarda el texto de antes y el de ahora, así que la corrección queda a la vista.

- Contrato: `contracts/bitacora/src/lib.rs:146`
- Pantalla: paso 6, botón «Corregir mi último hito» (`web/boveda/mainnet/index.html:116`)
- Llamada: `web/src/boveda-mainnet.ts:230`
- Invocación en mainnet: [f0cedc98…](https://stellar.expert/explorer/public/tx/f0cedc98f741c040f602cf297f39e92dee931da2b6793b697fd54fd1a4d65b0c)

## También en la página

- `saldo()` (Bóveda) muestra cuántos XLM guarda la bóveda: [ef554edb…](https://stellar.expert/explorer/public/tx/ef554edb4fe6ab02096d960ecaf15dc98765e8e8a106604a150ab7d81e9f65af).
- `insignias(titular)` (Insignias) muestra tus insignias con su medalla: [aaa565c7…](https://stellar.expert/explorer/public/tx/aaa565c724affff20668bd4de650b56bc2666de3965c7f46f1d0949b05b1607e).
