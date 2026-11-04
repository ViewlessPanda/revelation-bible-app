// Runs vitest with its user-data directory on this drive. Vitest 5 writes a
// `.vitest-secret-token` into %LOCALAPPDATA%\vitest (C:) otherwise; it reads
// LOCALAPPDATA, so point that at node_modules/.cache for the test process only.
import { spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const dataDir = fileURLToPath(new URL("../node_modules/.cache/vitest-data", import.meta.url));
mkdirSync(dataDir, { recursive: true });
const vitest = fileURLToPath(new URL("../node_modules/vitest/vitest.mjs", import.meta.url));
const r = spawnSync(process.execPath, [vitest, ...process.argv.slice(2)], {
  stdio: "inherit",
  env: { ...process.env, LOCALAPPDATA: dataDir },
});
process.exit(r.status ?? 1);
