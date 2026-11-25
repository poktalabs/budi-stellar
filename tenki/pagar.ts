// Pay in XLM on Stellar testnet from the Budi para Stellar app, with a memo
// (for example a course payment or a challenge code). The payer signs with
// STELLAR_SECRET from .env, which never goes into git.
import { Asset, Horizon, Keypair, Memo, Networks, Operation, TransactionBuilder } from "@stellar/stellar-sdk";

const HORIZON = "https://horizon-testnet.stellar.org";

export async function pagar(opts: { secret: string; to: string; amount: string; memo?: string }) {
  const server = new Horizon.Server(HORIZON);
  const payer = Keypair.fromSecret(opts.secret);
  const account = await server.loadAccount(payer.publicKey());
  let builder = new TransactionBuilder(account, { fee: "1000", networkPassphrase: Networks.TESTNET })
    .addOperation(Operation.payment({ destination: opts.to, asset: Asset.native(), amount: opts.amount }))
    .setTimeout(60);
  if (opts.memo) builder = builder.addMemo(Memo.text(opts.memo));
  const tx = builder.build();
  tx.sign(payer);
  const res = await server.submitTransaction(tx);
  return { hash: res.hash, url: `https://stellar.expert/explorer/testnet/tx/${res.hash}` };
}
