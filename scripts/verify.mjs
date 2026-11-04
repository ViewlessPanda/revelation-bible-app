// One command that runs every gate CI runs (except the pipeline rebuild and the oracle job), with capped output.
// Usage (from app/):  npm run verify                       (all gates)
//                     npm run verify -- --only fmt,vitest  (subset while iterating; use all gates before a commit)
// Prints one PASS/FAIL/SKIP line per gate, the first failing test/error for each failure, and a RESULT line.
// Exit code 1 if any gate failed, else 0.
//
// Gates (same commands as .github/workflows/ci.yml, Windows leg):
//   fmt, vitest, svelte-check, build, clippy, cargo-test, contrast, db-hashes, content-bytes, tracked-files
// Read-only: it edits no source file and touches no ACL. Side effects are only those of the gates themselves
// (npm run build refreshes the git-ignored app/dist; cargo refreshes target/). It does not call preflight-acl.ps1.
// Not included, on purpose: rebuilding pipeline/out (CI "Build content databases") because that rewrites the
// databases; db-hashes checks the databases already in pipeline/out against the committed hashes instead.
// Plain node (no packages). Replaces the earlier PowerShell version.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { closeSync, existsSync, openSync, readdirSync, readFileSync, readSync } from 'node:fs';
import { delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const app = join(root, 'app');
const win = process.platform === 'win32';

// --only a,b  or  --only=a,b
const argv = process.argv.slice(2);
const only = [];
for (let i = 0; i < argv.length; i++) {
  const m = /^--only(?:=(.*))?$/.exec(argv[i]);
  if (m) only.push(...(m[1] ?? argv[++i] ?? '').split(',').filter(Boolean));
}

// Toolchain: uses the environment as it is (RUSTUP_HOME, CARGO_HOME, PATH). Optional: set REVELATION_TOOLS_HOME
// to a folder that holds rustup\, cargo\, node\ and tmp\ and verify uses those for anything the environment
// leaves unset (it adds cargo\bin and node\* to PATH and uses tmp\ for TEMP). Nothing is assumed otherwise.
const env = { ...process.env };
const toolsHome = env.REVELATION_TOOLS_HOME;
const pathKey = Object.keys(env).find((k) => k.toLowerCase() === 'path') ?? 'PATH';
if (toolsHome && existsSync(toolsHome)) {
  env.RUSTUP_HOME ||= join(toolsHome, 'rustup');
  env.CARGO_HOME ||= join(toolsHome, 'cargo');
  const tmp = join(toolsHome, 'tmp');
  if (win && existsSync(tmp) && !(env.TEMP ?? '').toLowerCase().startsWith(toolsHome.toLowerCase())) {
    env.TMP = tmp;
    env.TEMP = tmp;
  }
  const extra = [join(env.CARGO_HOME, 'bin')];
  const nodeParent = join(toolsHome, 'node');
  if (existsSync(nodeParent)) {
    for (const d of readdirSync(nodeParent, { withFileTypes: true })) if (d.isDirectory()) extra.push(join(nodeParent, d.name));
  }
  for (const p of extra) {
    if (existsSync(p) && !(env[pathKey] ?? '').toLowerCase().includes(p.toLowerCase())) {
      env[pathKey] = p + delimiter + (env[pathKey] ?? '');
    }
  }
}
env.REVELATION_SOURCES ||= join(root, 'sources', 'raw');

const pathDirs = (env[pathKey] ?? '').split(delimiter).filter(Boolean);
function findOnPath(names) {
  const found = [];
  for (const d of pathDirs) for (const n of names) if (existsSync(join(d, n))) found.push(join(d, n));
  return found;
}

// Git's bash, never WSL's launcher (System32\bash.exe) or the WindowsApps stub.
let bash = findOnPath(win ? ['bash.exe'] : ['bash']).find((p) => !/\\(System32|WindowsApps)\\/i.test(p));
if (!bash && win) {
  for (const g of findOnPath(['git.exe'])) {
    for (const rel of ['..\\bin\\bash.exe', '..\\..\\bin\\bash.exe', '..\\..\\usr\\bin\\bash.exe']) {
      const c = resolve(dirname(g), rel);
      if (!bash && existsSync(c)) bash = c;
    }
  }
}
const py = win ? findOnPath(['py.exe', 'python.exe'])[0] : findOnPath(['python3', 'python'])[0];

function firstFailure(lines) {
  // First failing test name or first error line, whichever the gate printed; falls back to the last 3 lines.
  const pats = [
    /^---- (.+) stdout ----/, // cargo: failing test name
    /^test (.+) \.\.\. FAILED/, // cargo (non-quiet)
    /^\s*(FAIL|\u00D7|\u2717)\s+.+/, // vitest
    /^\s*(Error|error(\[\w+\])?:)\s*.+/, // rustc, svelte-check, node
    /^Diff in .+/, // rustfmt
    /^\s*FAIL\b.+|^::error.+|^.*\bNOT\b.+|^.*mismatch.+/i,
  ];
  for (const p of pats) {
    const hit = lines.find((l) => p.test(l));
    if (hit) return [hit];
  }
  return lines.filter((l) => l.trim()).slice(-3);
}

function detail(name, lines) {
  if (name === 'cargo-test') {
    let n = 0;
    for (const l of lines) {
      const m = /test result: ok\. (\d+) passed/.exec(l);
      if (m) n += Number(m[1]);
    }
    if (n) return `${n} tests`;
  } else if (name === 'vitest') {
    const l = lines.filter((x) => /Tests\s+.*passed/.test(x)).pop();
    if (l) return l.trim().replace(/\s+/g, ' ');
  }
  return '';
}

let fail = 0;
let ran = 0;
const selected = (name) => !only.length || only.includes(name);

function report(name, code, sec, lines) {
  if (code === 0) {
    console.log(`PASS  ${name.padEnd(13)} ${String(sec).padStart(4)}s  ${detail(name, lines)}`.trimEnd());
  } else {
    fail++;
    console.log(`FAIL  ${name.padEnd(13)} ${String(sec).padStart(4)}s  exit ${code}`);
    for (const l of firstFailure(lines)) {
      const t = l.trim();
      console.log(`      first failure: ${t.length > 150 ? t.slice(0, 150) + '...' : t}`);
    }
  }
}

// cmd is [program, ...args]; npm is a .cmd on Windows, which needs a shell.
function gate(name, cwd, cmd) {
  if (!selected(name)) return;
  ran++;
  const t0 = Date.now();
  const useShell = win && cmd[0] === 'npm';
  const r = spawnSync(useShell ? 'npm.cmd' : cmd[0], cmd.slice(1), {
    cwd,
    env,
    encoding: 'utf8',
    maxBuffer: 512 * 1024 * 1024,
    shell: useShell,
    windowsHide: true,
  });
  const text = `${r.stdout ?? ''}\n${r.stderr ?? ''}${r.error ? `\nError: ${r.error.message}` : ''}`;
  const lines = text.split(/\r?\n/);
  const code = r.status ?? (r.error ? 127 : 1);
  report(name, code, Math.floor((Date.now() - t0) / 1000), lines);
}

function gateFn(name, fn) {
  if (!selected(name)) return;
  ran++;
  const t0 = Date.now();
  const lines = [];
  let code = 1;
  try {
    code = fn(lines) ? 0 : 1;
  } catch (e) {
    lines.push(`Error: ${e.message}`);
  }
  report(name, code, Math.floor((Date.now() - t0) / 1000), lines);
}

function skip(name, why) {
  if (!selected(name)) return;
  console.log(`SKIP  ${name.padEnd(13)}        ${why}`);
}

function sha256File(path) {
  const h = createHash('sha256');
  const fd = openSync(path, 'r');
  try {
    const buf = Buffer.alloc(1 << 20);
    let n;
    while ((n = readSync(fd, buf, 0, buf.length, null)) > 0) h.update(buf.subarray(0, n));
  } finally {
    closeSync(fd);
  }
  return h.digest('hex');
}

// Order follows CI: fmt, UI tests/check/build (app/dist must exist before the app crate compiles), clippy, test.
gate('fmt', root, ['cargo', 'fmt', '--all', '--check']);
gate('vitest', app, ['npm', 'test', '--silent']);
gate('svelte-check', app, ['npm', 'run', 'check', '--silent']);
gate('build', app, ['npm', 'run', 'build', '--silent']);
gate('clippy', root, ['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--quiet', '--', '-D', 'warnings']);
gate('cargo-test', root, ['cargo', 'test', '--workspace', '--locked', '--quiet']);
if (py) gate('contrast', root, [py, '-I', '-B', 'docs/design/check-contrast.py', '--strict']);
else skip('contrast', 'python/py not found');

// CI: grep -v '^#' output-sha256.txt | sha256sum --strict -c -   (run inside pipeline/out). Same check, read-only.
gateFn('db-hashes', (out) => {
  const outDir = join(root, 'pipeline', 'out');
  let bad = 0;
  let n = 0;
  for (const line of readFileSync(join(root, 'pipeline', 'tests', 'expected', 'output-sha256.txt'), 'utf8').split(/\r?\n/)) {
    if (/^\s*#/.test(line) || !line.trim()) continue;
    const m = /^([0-9a-f]{64}) [ *](.+)$/.exec(line);
    if (!m) {
      out.push(`malformed line: ${line}`);
      bad++;
      continue;
    }
    n++;
    const file = join(outDir, m[2]);
    if (!existsSync(file)) {
      out.push(`missing: ${m[2]}`);
      bad++;
    } else if (sha256File(file) !== m[1]) {
      out.push(`HASH MISMATCH: ${m[2]}`);
      bad++;
    }
  }
  if (n === 0) {
    out.push('no hashes found');
    bad++;
  }
  return bad === 0;
});

if (bash) {
  gate('content-bytes', root, [bash, '.github/scripts/check-content-bytes.sh']);
  gate('tracked-files', root, [bash, '.github/scripts/check-tracked-files.sh']);
} else {
  skip('content-bytes', 'bash not found');
  skip('tracked-files', 'bash not found');
}

if (ran === 0) {
  console.log(`RESULT: no gate matched --only ${only.join(',')}`);
  process.exit(1);
}
if (fail) {
  console.log(`RESULT: ${fail} of ${ran} gates FAILED`);
  process.exit(1);
}
console.log(`RESULT: all ${ran} gates passed`);
