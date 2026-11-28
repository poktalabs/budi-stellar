# Budi para Stellar en mainnet

La Bóveda, el bono de bienvenida de Budi para Stellar, corre en la red principal de Stellar. Es el mismo contrato que en testnet (`contracts/boveda`): guarda XLM y quien presenta el código se lleva todo el saldo, una vez por wallet. El contrato guarda solo el sha256 del código normalizado (minúsculas, sin espacios), nunca el código.

- Contrato en mainnet: [`CDT2SSF7IM5UDNZHFOAKUFAQUG5UQSZN4JORYOKOW2RJXKK4V547G24N`](https://stellar.expert/explorer/public/contract/CDT2SSF7IM5UDNZHFOAKUFAQUG5UQSZN4JORYOKOW2RJXKK4V547G24N)
- Despliegue: [tx `6effa829…`](https://stellar.expert/explorer/public/tx/6effa8296c6cb97a3d8d5a01c39a3bbbea9450fccc3902a4ac57d4a31312d3f5)
- Token: XLM nativo (Stellar Asset Contract `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA`)
- Frontend: https://stellar.budi.lat/boveda/mainnet/ (pantalla `web/boveda/mainnet/index.html`, lógica `web/src/boveda-mainnet.ts`)

Todas las llamadas salen del frontend: el botón arma la transacción con `escribir()` (`web/src/stellar.ts`), la firma la wallet del usuario en Freighter (Mainnet) y se envía por RPC. `usarMainnet()` cambia la página a la red principal (`web/src/boveda-mainnet.ts:9`).

## Las 5 funciones

### 1. `depositar(de, monto)`

Mueve `monto` XLM de la wallet `de` al contrato y regresa el saldo nuevo. Pide la firma de `de`. Llena la bóveda.

- Contrato: `contracts/boveda/src/lib.rs:66`
- Pantalla: paso 2, «Llena la bóveda», botón «Depositar XLM» (`web/boveda/mainnet/index.html:42`)
- Llamada: `web/src/boveda-mainnet.ts:93`
- Invocación en mainnet: PENDIENTE

### 2. `reclamar(quien, codigo)`

Si el código es el correcto, manda todo el saldo a `quien`. Una vez por wallet. Pide la firma de `quien`.

- Contrato: `contracts/boveda/src/lib.rs:78`
- Pantalla: paso 3, «Ábrela con el código», botón «Abrir la bóveda» (`web/boveda/mainnet/index.html:55`)
- Llamada: `web/src/boveda-mainnet.ts:108`
- Invocación en mainnet: PENDIENTE

### 3. `saldo()`

Cuántos XLM guarda la bóveda. La página la usa para mostrar el saldo, y el botón la envía como transacción firmada para que la consulta quede en el historial.

- Contrato: `contracts/boveda/src/lib.rs:102`
- Pantalla: paso 4, «Deja constancia en la red», botón «Saldo» (`web/boveda/mainnet/index.html:67`)
- Llamada: `web/src/boveda-mainnet.ts:118` y `web/src/boveda-mainnet.ts:128`
- Invocación en mainnet: PENDIENTE

### 4. `reclamado(quien)`

Cuántos XLM se llevó una wallet al abrir la bóveda (0 si no la ha abierto).

- Contrato: `contracts/boveda/src/lib.rs:107`
- Pantalla: paso 4, botón «Lo que me llevé» (`web/boveda/mainnet/index.html:68`)
- Llamada: `web/src/boveda-mainnet.ts:119` y `web/src/boveda-mainnet.ts:128`
- Invocación en mainnet: PENDIENTE

### 5. `admin()`

La cuenta que administra la bóveda (la cuenta de Budi que la desplegó).

- Contrato: `contracts/boveda/src/lib.rs:111`
- Pantalla: paso 4, botón «Quién la administra» (`web/boveda/mainnet/index.html:69`)
- Llamada: `web/src/boveda-mainnet.ts:120` y `web/src/boveda-mainnet.ts:128`
- Invocación en mainnet: PENDIENTE
