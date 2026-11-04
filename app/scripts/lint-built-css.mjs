// Runs the colour-lock and motion lint on the built CSS (dist/assets/*.css).
// Part of `npm run build`, so `tauri build` fails if the built CSS breaks it.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { lintCss } from "./csslint.mjs";

const dir = new URL("../dist/assets/", import.meta.url);
const files = readdirSync(dir).filter((f) => f.endsWith(".css"));
if (files.length === 0) {
  console.error("lint-built-css: no CSS found in dist/assets");
  process.exit(1);
}
let failed = false;
for (const f of files) {
  const problems = lintCss(readFileSync(join(dir.pathname.replace(/^\/([A-Za-z]:)/, "$1"), f), "utf8"));
  for (const p of problems) console.error(`${f}: ${p}`);
  if (problems.length) failed = true;
  else console.log(`lint-built-css: ${f} OK (colour lock + motion)`);
}
process.exit(failed ? 1 : 0);
