# AGENTS: budi-stellar

Context for any agent (Claude Code, Devin, Codex, Cursor) working in this repo.

- This repo is the Ship It: Builder Challenge entry (BAF × Stellar México, 23 to 27 Nov 2026). The challenge scores commits on `main` of this public repo (merge commits do not count), so work lands on `main` as one commit per milestone. This is a deliberate exception to the usual branch rule and applies to this repo only.
- Commits are authored as `mel <mel@innvertir.com>`. No empty or filler commits.
- Secrets never land in tracked files. `.env` is gitignored; `.env.example` holds only the public image ref.
- Every live Tenki run is bounded: sessions have `maxDurationMs`, and `--check` runs always close their session.
- Keep `RUST_VERSION` in `tenki/recipe.ts` in step with `rust-toolchain.toml`. Rebuild the image (`pnpm image:build`) after changing the starter or the recipe, then update `.env.example`.
- The Budi app lives in the private `poktalabs/budi` repo; changes there go through a PR Mel approves.
- **Budi's core stays private.** Never copy Budi's explainer engine, the Budi extension, or any other code from Budi's private repos into this repo. Budi para Stellar is the first client of Budi's platform: it reaches explainers and the extension only through Budi's skills, MCP server and API, and this repo holds pointers to them (URLs, skill names, API routes), never their source.
- **The Budi brand is not open source** (see `TRADEMARKS.md`). Never add brand assets here: the logo, the Budi character drawings, brand themes or fonts, Budi's voice clips, or videos and pages in the Budi brand. They live in Budi's private repos; Budi para Stellar videos live in the private `budi-stellar-video` folder of the Budi project.
- Content made by Budi's engines (explainer pages, narrated videos) lives on Budi's side and is linked from here; generated output is not committed here unless Mel says so.
- No em-dashes in written output. Prose in `.md` files is one continuous line per paragraph.
