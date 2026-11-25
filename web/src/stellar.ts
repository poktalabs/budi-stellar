// Budi para Stellar en el navegador: el alumno paga el curso (queda en custodia en
// Matrícula) y registra hitos en la Bitácora que pide ese curso. Firma con Freighter.
import { getNetworkDetails, isConnected, requestAccess, signTransaction } from "@stellar/freighter-api";
import { Address, Contract, nativeToScVal, Networks, rpc, scValToNative, TransactionBuilder, xdr } from "@stellar/stellar-sdk";

export const RED = Networks.TESTNET;
export const RPC_URL = "https://soroban-testnet.stellar.org";
export const MATRICULA = "CAUATYLPFCJTSFWKQQUC7LO5R5RIIPY7WXE6EEYZLMA7Z7HCW7LHYDAA";
export const BITACORA = "CDOI7II5MEP5M765BOWH4LQQ45YBE4HHLH7CN2534UUZLMBDNFUB4AUQ";
export const CURSO = "soroban_101";
export const EXPLORER = "https://stellar.expert/explorer/testnet";

const server = new rpc.Server(RPC_URL);

export async function conectar(): Promise<string> {
  const hay = await Promise.race([
    isConnected().then((r) => r.isConnected),
    new Promise<boolean>((ok) => setTimeout(() => ok(false), 3000)),
  ]);
  if (!hay) throw new Error("No encontramos Freighter. Instálala en freighter.app, cámbiala a Testnet y vuelve a intentar.");
  const acceso = await requestAccess();
  if (acceso.error || !acceso.address) {
    throw new Error("No se pudo conectar Freighter. ¿Está instalada y desbloqueada?");
  }
  const red = await getNetworkDetails();
  if (red.networkPassphrase !== RED) {
    throw new Error("Cambia Freighter a Testnet y vuelve a conectar.");
  }
  return acceso.address;
}

/** Fondea la cuenta en testnet con Friendbot (solo si aún no existe). */
export async function fondear(cuenta: string): Promise<void> {
  const r = await fetch(`https://friendbot.stellar.org/?addr=${encodeURIComponent(cuenta)}`);
  if (!r.ok && r.status !== 400) throw new Error("Friendbot no respondió. Intenta de nuevo.");
}

export async function existe(cuenta: string): Promise<boolean> {
  try {
    await server.getAccount(cuenta);
    return true;
  } catch {
    return false;
  }
}

/** Lee un valor del contrato sin firmar (simulación). */
async function leer(cuenta: string, contrato: string, metodo: string, args: xdr.ScVal[]) {
  const fuente = await server.getAccount(cuenta);
  const tx = new TransactionBuilder(fuente, { fee: "100", networkPassphrase: RED })
    .addOperation(new Contract(contrato).call(metodo, ...args))
    .setTimeout(30)
    .build();
  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) throw new Error(sim.error);
  return sim.result ? scValToNative(sim.result.retval) : undefined;
}

/** Firma con Freighter, envía y espera la confirmación. Regresa el hash. */
async function escribir(cuenta: string, contrato: string, metodo: string, args: xdr.ScVal[]) {
  const fuente = await server.getAccount(cuenta);
  const tx = new TransactionBuilder(fuente, { fee: "100000", networkPassphrase: RED })
    .addOperation(new Contract(contrato).call(metodo, ...args))
    .setTimeout(120)
    .build();
  const lista = await server.prepareTransaction(tx);
  const firmada = await signTransaction(lista.toXDR(), { networkPassphrase: RED, address: cuenta });
  if (firmada.error) throw new Error("Cancelaste la firma en Freighter.");
  const enviada = await server.sendTransaction(TransactionBuilder.fromXDR(firmada.signedTxXdr, RED));
  if (enviada.status === "ERROR") throw new Error("La red rechazó la transacción.");
  for (let i = 0; i < 30; i++) {
    const r = await server.getTransaction(enviada.hash);
    if (r.status === rpc.Api.GetTransactionStatus.SUCCESS) return enviada.hash;
    if (r.status === rpc.Api.GetTransactionStatus.FAILED) throw new Error("La transacción falló.");
    await new Promise((ok) => setTimeout(ok, 1000));
  }
  throw new Error("La red tardó demasiado. Revisa el enlace en unos segundos.");
}

const yo = (c: string) => new Address(c).toScVal();
const curso = () => nativeToScVal(CURSO, { type: "symbol" });

export async function precioXlm(cuenta: string): Promise<number> {
  const c = (await leer(cuenta, MATRICULA, "curso", [curso()])) as { precio: bigint };
  return Number(c.precio) / 10_000_000;
}

export async function inscrito(cuenta: string): Promise<boolean> {
  return Boolean(await leer(cuenta, MATRICULA, "inscrito", [yo(cuenta), curso()]));
}

export async function inscribir(cuenta: string): Promise<string> {
  return escribir(cuenta, MATRICULA, "inscribir", [yo(cuenta), curso()]);
}

export async function registrar(cuenta: string, hito: string): Promise<string> {
  return escribir(cuenta, BITACORA, "registrar", [yo(cuenta), nativeToScVal(hito, { type: "string" })]);
}

export async function hitos(cuenta: string): Promise<string[]> {
  return ((await leer(cuenta, BITACORA, "hitos", [yo(cuenta)])) as string[]) ?? [];
}
