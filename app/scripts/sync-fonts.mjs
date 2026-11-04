// Copies the bundled fonts and their licence files from /assets/fonts into
// public/fonts (Vite copies public/ into dist/ byte for byte, names unchanged),
// after checking every file against assets/fonts/fonts.sha256. Fails closed:
// a missing or changed file stops the build (sources/SOURCES.md F1, F2).
//
//   node scripts/sync-fonts.mjs               copy (part of npm run dev/build)
//   node scripts/sync-fonts.mjs --check-dist  verify dist/fonts after vite build
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = fileURLToPath(new URL("..", import.meta.url));
const assets = join(app, "..", "assets", "fonts");
const manifest = join(assets, "fonts.sha256");
const checkDist = process.argv.includes("--check-dist");
const target = join(app, checkDist ? "dist" : "public", "fonts");

const fail = (m) => { console.error(`sync-fonts: ${m}`); process.exit(1); };
const sha256 = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");

const entries = [];
for (const [i, raw] of readFileSync(manifest, "utf8").split("\n").entries()) {
  const line = raw.replace(/\r$/, "");
  if (line.trim() === "" || line.startsWith("#")) continue;
  const m = /^([0-9a-f]{64}) {2}([a-z0-9-]+\/[A-Za-z0-9.,\[\]_-]+)$/.exec(line);
  if (!m) fail(`${manifest} line ${i + 1} is malformed`);
  entries.push({ hash: m[1], path: m[2] });
}
if (entries.length === 0) fail("no fonts listed");

if (!checkDist) {
  rmSync(target, { recursive: true, force: true });
  for (const e of entries) {
    const src = join(assets, e.path);
    if (!existsSync(src)) fail(`${src} missing`);
    if (sha256(src) !== e.hash) fail(`${src} does not match fonts.sha256`);
    const dest = join(target, e.path);
    mkdirSync(dirname(dest), { recursive: true });
    copyFileSync(src, dest);
    if (sha256(dest) !== e.hash) fail(`copy of ${e.path} differs`);
  }
  console.log(`sync-fonts: ${entries.length} files verified and copied to public/fonts`);
} else {
  const listed = new Set(entries.map((e) => e.path));
  for (const fam of readdirSync(target)) {
    for (const f of readdirSync(join(target, fam))) {
      if (!listed.has(`${fam}/${f}`)) fail(`dist/fonts/${fam}/${f} is not in fonts.sha256`);
    }
  }
  for (const e of entries) {
    const p = join(target, e.path);
    if (!existsSync(p) || sha256(p) !== e.hash) fail(`dist/fonts/${e.path} missing or changed`);
  }
  console.log(`sync-fonts: dist/fonts has exactly the ${entries.length} listed files, unmodified`);
}
