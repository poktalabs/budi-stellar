// Where things live inside a Budi para Stellar session. SDK file ops are
// rooted at /home/tenki, so they take the relative form.
export const LAYOUT = {
  home: "/home/tenki",
  // Baked into the image: this repo at the pinned commit, already built once
  // so cargo's registry and target dir are warm.
  starter: "/home/tenki/starter",
  // The builder's project for this session (their repo, or a copy of the starter).
  project: "/home/tenki/project",
  projectRel: "project",
  codeServer: "/home/tenki/.local/lib/code-server",
  runDir: "/home/tenki/run",
  runDirRel: "run",
} as const;

export const IDE_PORT = 8080;
