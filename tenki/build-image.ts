// Build the session image on Tenki, publish it public, and print the digest
// ref to pin in BUDI_STELLAR_IMAGE. The image holds only this public repo and
// public toolchains: no secrets, so any Tenki account can boot it.
//   pnpm image:build                        (starter at origin/main)
//   BUDI_STELLAR_REF=<sha> pnpm image:build
import { execFileSync } from "node:child_process";
import { TenkiSandbox } from "@tenkicloud/sandbox";
import { IMAGE_NAME, recipe } from "./recipe.js";

if (!process.env.TENKI_API_KEY) throw new Error("TENKI_API_KEY is not set");
const ref =
  process.env.BUDI_STELLAR_REF?.trim() ||
  execFileSync("git", ["rev-parse", "origin/main"], { encoding: "utf8" }).trim();
const tag = process.env.BUDI_STELLAR_TAG?.trim() || "main";

const tenki = new TenkiSandbox({ authToken: process.env.TENKI_API_KEY });
try {
  const spec = recipe(ref);
  const problems = spec.validate();
  if (problems.length) throw new Error(`invalid recipe: ${JSON.stringify(problems)}`);

  const existing = (await tenki.listTemplates()).find((t) => t.name === IMAGE_NAME);
  const template = existing
    ? await tenki.updateTemplate(existing, { spec })
    : await tenki.createTemplate({ name: IMAGE_NAME, spec });
  console.log(`template ${template.id} (${existing ? "updated" : "created"}), starter ${ref.slice(0, 7)}; building...`);

  const build = await tenki.buildTemplate(template, { waitForCompletion: true });
  if (build.state !== "READY") {
    console.error(build.error, "\n", build.buildLogTail);
    process.exitCode = 1;
  } else {
    console.log(build.buildLogTail.split("\n").slice(-12).join("\n"));
    const owner = await tenki.resolveRegistryRef(build.image?.digestRef ?? "");
    const published = await tenki.publishRegistryImage({
      workspaceId: owner.owningWorkspaceId,
      fromTemplateId: template.id,
      name: IMAGE_NAME,
      tag,
      visibility: "public",
    });
    console.log(`published ${owner.owningWorkspaceSlug}/${IMAGE_NAME}:${tag} (${published.image?.visibility ?? "?"})`);
    console.log(`\nBUDI_STELLAR_IMAGE=${published.digestRef}`);
  }
} finally {
  tenki.close();
}
