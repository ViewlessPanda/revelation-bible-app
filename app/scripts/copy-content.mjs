// Release step: copy the pipeline's kjv.sqlite, kjv-search.sqlite (phrase search), content.sqlite (lexicons)
// and mhcc.sqlite (commentary)
// into `content\` next to the release exe, only if each one's SHA-256 equals
// the committed expected value in pipeline/tests/expected/output-sha256.txt
// (the same file build.rs compiles into the exe). Every source is checked
// before anything is copied. The `.sha256` sidecar beside a DB is never read
// (pipeline/DESIGN.md 10.2). Each copy is re-hashed after writing.
//
// The workspace's target/ is at the repository root (task D4b), so the exe is
// target\release\revelation.exe and the content goes to target\release\content.
//
//   node scripts/copy-content.mjs            (or: npm run copy-content)
//
// The GNU GPL v3 text goes beside kjv.sqlite (the project's internal review notes 8.4:
// the KJV data file is GPL-3.0 and carries its licence), checked against the
// FSF text's SHA-256 (tests/licence.test.ts checks licenses/GPL-3.0.txt too).
//
// Exit code 1 and nothing left in place on any mismatch.
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, renameSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const root = fileURLToPath(new URL("../..", import.meta.url));
const expectedFile = join(root, "pipeline", "tests", "expected", "output-sha256.txt");
const FILES = ["kjv.sqlite", "kjv-search.sqlite", "content.sqlite", "mhcc.sqlite"];
/** SHA-256 of https://www.gnu.org/licenses/gpl-3.0.txt (35,149 bytes, fetched 2026-10-24). */
const GPL_SHA256 = "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986";
const releaseDir = join(root, "target", "release");
const destDir = join(releaseDir, "content");

function fail(msg) {
  console.error(`copy-content: ${msg}`);
  process.exit(1);
}

/** Same rules as crates/revelation-content/src/hashfile.rs: comments and blank lines skipped, strict lines. */
function expectedHash(text, file) {
  let found = null;
  for (const [i, raw] of text.split("\n").entries()) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    if (line.trim() === "" || line.startsWith("#")) continue;
    const m = /^([0-9a-f]{64})(?:  | \*)([A-Za-z0-9._-]+)$/.exec(line);
    if (!m) fail(`${expectedFile} line ${i + 1} is malformed`);
    if (m[2] === file) {
      if (found) fail(`${file} is listed twice in ${expectedFile}`);
      found = m[1];
    }
  }
  if (!found) fail(`no expected SHA-256 for ${file} in ${expectedFile}`);
  return found;
}

const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");

if (!existsSync(join(releaseDir, "revelation.exe"))) {
  fail("build the release exe first: npx tauri build --no-bundle");
}
const expectedText = readFileSync(expectedFile, "utf8");
const plan = FILES.map((file) => {
  const want = expectedHash(expectedText, file);
  const source = join(root, "pipeline", "out", file);
  if (!existsSync(source)) fail(`${source} not found (run the pipeline build)`);
  const got = sha256(source);
  if (got !== want) fail(`${source} has SHA-256 ${got}, expected ${want}; nothing copied`);
  return { file, source, want };
});
{
  const source = join(root, "licenses", "GPL-3.0.txt");
  if (!existsSync(source)) fail(`${source} not found`);
  const got = sha256(source);
  if (got !== GPL_SHA256) fail(`${source} has SHA-256 ${got}, expected ${GPL_SHA256}; nothing copied`);
  plan.push({ file: "GPL-3.0.txt", source, want: GPL_SHA256 });
}

mkdirSync(destDir, { recursive: true });
for (const { file, source, want } of plan) {
  const dest = join(destDir, file);
  const tmp = `${dest}.partial`;
  copyFileSync(source, tmp);
  const copied = sha256(tmp);
  if (copied !== want) {
    rmSync(tmp, { force: true });
    fail(`the copy of ${file} has SHA-256 ${copied}, expected ${want}; removed`);
  }
  renameSync(tmp, dest);
  console.log(`copy-content: ${dest} (SHA-256 ${want})`);
}
