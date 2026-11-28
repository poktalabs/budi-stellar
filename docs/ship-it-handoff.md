# Ship It final hour: handoff for helper agents (Devin, Pi)

Deadline: the leaderboard freezes at 23:59 CST on 27 Nov 2026. Push your work by **23:45 CST** so Claude Code can claim it in time. Anything later will not count.

## Goal

Add new public Soroban contract functions to this repo. Each one scores 10 pts once it is claimed and approved. A function counts only if all three hold: it is a real write function (requires auth and changes state), it has a test in this repo, and it has been called once on **testnet**. Claude Code submits every claim from Mel's account; you never claim.

## Rules

- Work only in your own NEW contract folder (assignments below). Never edit another folder, the workspace `Cargo.toml` (it already includes every `contracts/*`), or any other contract.
- Function names must be unique across the whole repo. Run `grep -rhoE "pub fn [a-z_0-9]+" contracts/*/src/lib.rs | sort -u` before you name anything, and prefix names with your domain (for example `crear_grupo_estudio`, not `crear`).
- Each function calls `require_auth` on the right party, emits an event, and has a test in `src/test.rs`. Read functions don't score; add them only when they're needed.
- Testnet only: never mainnet. Create your own key with `stellar keys generate <yourname> --network testnet --fund`, deploy a fresh instance, and call each write function once. For tokens, use the native XLM SAC on testnet.
- Write real functions in Budi's domain (AI + blockchain education in Spanish: learners, mentors, cohorts, badges, agents for hire). No copy-paste templates under different names.
- Commit as `Mel <mel@innvertir.com>`. Run `git pull --rebase` before you push to `main`. Never force-push.

## Assignments (new folders only)

| Agent | Folder | Domain |
|---|---|---|
| Devin | `contracts/comunidad/` | Study groups and a Q&A forum for learners: create a group, join, leave, ask a question, answer, accept an answer, upvote, flag, moderate, pin, close a group, invite a member, remove a member, transfer group ownership. |
| Pi | `contracts/eventos/` | Live events and workshops: create an event, set capacity, register, cancel a registration, check in, issue an attendance proof, refund a deposit on attendance, keep a no-show deposit, add a speaker, remove a speaker, reschedule, cancel an event, rate an event. |

Folders already taken or in flight, so don't use them: agentes, bitacora, boveda, cohortes, cuenta, cuentas, examen, insignias, matricula, reputacion, retos, mentorias, proyectos, becas, empleos.

## Hand back claims

When your functions are tested and called on testnet, commit a file `docs/claims/<agent>.txt` along with your contract. Put one line per write function, as the function name and the testnet tx hash separated by a space:

```text
crear_grupo_estudio 0e8469033e0d2958e10a39358146f831f4919989b8ce77e9c24d7b9d485c586f
```

Push in batches as soon as you have 5 or more working functions; don't wait to finish all of them. Claude Code watches `main` and claims each batch.

## Done when

`cargo test -p <your-contract>` passes, every line in your claims file points to a successful testnet transaction, and both are pushed to `main` before 23:45 CST.
