// The Budi para Stellar session image: everything a Soroban builder needs,
// pinned so every session is the same.
//
// Tenki facts this follows (proven on Budi's own session image):
// - A template cannot FROM an external image and takes exactly one Git
//   context; this repo is that context (the starter project).
// - `.run()` steps run as the non-root `tenki` user, so Rust, Stellar CLI and
//   code-server install into the home dir; only `.apt()` touches the system.
// - Never replace PATH in buildEnv; each step sources what it needs.
import { TemplateSpec } from "@tenkicloud/sandbox";
import { LAYOUT } from "./layout.js";

export const IMAGE_NAME = "budi-stellar-dev";
export const REPO = "https://github.com/poktalabs/budi-stellar";
// Keep in step with rust-toolchain.toml (Stellar CLI refuses 1.91.0).
export const RUST_VERSION = "1.92.0";
export const STELLAR_CLI_VERSION = "30.0.1";
export const CODE_SERVER_VERSION = "4.146.0";
// The learner's Budi agent (same version as Budi's own sessions). Needs Node >= 22.19.
export const PI_VERSION = "1.4.0";

const ARCH = `case "$(uname -m)" in x86_64) A=x86_64; C=amd64;; aarch64|arm64) A=aarch64; C=arm64;; *) echo "unsupported arch $(uname -m)"; exit 1;; esac`;
const CARGO_ENV = `. "$HOME/.cargo/env"`;

export function recipe(ref: string): TemplateSpec {
  return new TemplateSpec()
    .fromImage("sandbox")
    .withGitContext({ repo: REPO, ref, checkout: { dest: LAYOUT.starter, mode: "contents" } })
    .workdir(LAYOUT.starter)
    // These are also the floor for every session booted from the image, so
    // stay inside Tenki's free tier (2 vCPU / 4 GiB) or free accounts can't launch.
    .resources({ cpuCores: 2, memoryMb: 4096, diskSizeGb: 20 })
    .apt(["build-essential", "pkg-config", "curl", "git", "ca-certificates", "jq"])
    .run(
      `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} --target wasm32v1-none --component rustfmt,clippy`,
      { name: "install-rust", timeoutSeconds: 900 },
    )
    .run(
      `${ARCH}; mkdir -p "$HOME/.local/bin" && curl -fsSL "https://github.com/stellar/stellar-cli/releases/download/v${STELLAR_CLI_VERSION}/stellar-cli-${STELLAR_CLI_VERSION}-$A-unknown-linux-gnu.tar.gz" | tar xz -C "$HOME/.local/bin" stellar`,
      { name: "install-stellar-cli", timeoutSeconds: 300 },
    )
    .run(
      `${ARCH}; mkdir -p "$HOME/.local/lib" && curl -fsSL "https://github.com/coder/code-server/releases/download/v${CODE_SERVER_VERSION}/code-server-${CODE_SERVER_VERSION}-linux-$C.tar.gz" | tar xz -C "$HOME/.local/lib" && mv "$HOME/.local/lib/code-server-${CODE_SERVER_VERSION}-linux-$C" "${LAYOUT.codeServer}" && ln -sf "${LAYOUT.codeServer}/bin/code-server" "$HOME/.local/bin/code-server"`,
      { name: "install-code-server", timeoutSeconds: 600 },
    )
    .run(`npm install -g --prefix "$HOME/.local" @earendil-works/pi-coding-agent@${PI_VERSION}`, {
      name: "install-pi",
      timeoutSeconds: 600,
    })
    // Builds and tests the starter once: proves the toolchain and leaves the
    // cargo registry warm, so a builder's first build is fast.
    .run(
      `${CARGO_ENV}; export PATH="$HOME/.local/bin:$PATH"; node --version && pi --version && rustc --version && stellar --version && code-server --version && cargo test && stellar contract build`,
      { name: "verify", timeoutSeconds: 1800 },
    );
}
