// Builds the app's icon set and the About dialog's mark from the owner's
// master, assets/brand/revelation-icon-1024.png (wave-2-layout.md 14).
//
//   node scripts/make-icon.mjs        (Windows: needs PowerShell for the crop)
//
// - src-tauri/icons/: the desktop set from `tauri icon` (full canvas), except
//   that 16, 24 and 32 px come from a tighter crop (the mark's bounding box +
//   6% padding, scripts/make-icon-small.ps1), so the star and the book stay
//   legible on the taskbar. icon.ico = 16, 24, 32 (tight) + 48, 64, 256 (full),
//   PNG-compressed entries.
// - public/brand/revelation-mark-{192,288,384}.png: the About mark at 1x, 1.5x
//   and 2x of 96 px, downscaled from the master (never upscaled).
//
// All resampling is `tauri icon`'s; this script only crops (WPF), copies and
// packs the ICO directory. The mark is never recoloured or redrawn.
//
// The mark is not under the MIT License (assets/brand/LICENSE; sources/SOURCES.md
// row BRAND-LOGO).
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const master = resolve(app, "../assets/brand/revelation-icon-1024.png");
const tmp = join(process.env.TMP || process.env.TEMP || resolve(app, "../.."), "revelation-icons");
const icons = join(app, "src-tauri/icons");
const brand = join(app, "public/brand");

rmSync(tmp, { recursive: true, force: true });
mkdirSync(tmp, { recursive: true });
const tauriIcon = (input, out, sizes = []) =>
  execFileSync(process.execPath, [join(app, "node_modules/@tauri-apps/cli/tauri.js"), "icon", input, "-o", out, ...sizes.flatMap((s) => ["-p", String(s)])], { stdio: "inherit", cwd: app });

// 1. tight crop for the small sizes
const tight = join(tmp, "mark-tight.png");
execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-STA", "-ExecutionPolicy", "Bypass", "-File", join(app, "scripts/make-icon-small.ps1"), "-Src", master, "-Out", tight], { stdio: "inherit" });

// 2. resampling (tauri icon)
tauriIcon(master, join(tmp, "full"));
tauriIcon(master, join(tmp, "full-px"), [48, 64, 256, 192, 288, 384]);
tauriIcon(tight, join(tmp, "tight"), [16, 24, 32]);

// 3. the desktop set (no android/ios: Revelation is a Windows desktop app)
mkdirSync(icons, { recursive: true });
for (const f of ["128x128.png", "128x128@2x.png", "icon.png", "icon.icns"]) copyFileSync(join(tmp, "full", f), join(icons, f));
copyFileSync(join(tmp, "tight", "32x32.png"), join(icons, "32x32.png"));

// 4. icon.ico (PNG entries; Windows Vista and later)
const entries = [
  ...[16, 24, 32].map((s) => [s, join(tmp, "tight", `${s}x${s}.png`)]),
  ...[48, 64, 256].map((s) => [s, join(tmp, "full-px", `${s}x${s}.png`)]),
].map(([s, f]) => ({ s, data: readFileSync(f) }));
const head = Buffer.alloc(6);
head.writeUInt16LE(0, 0); head.writeUInt16LE(1, 2); head.writeUInt16LE(entries.length, 4);
let offset = 6 + 16 * entries.length;
const dir = entries.map(({ s, data }) => {
  const e = Buffer.alloc(16);
  e[0] = s >= 256 ? 0 : s; e[1] = s >= 256 ? 0 : s;
  e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6);
  e.writeUInt32LE(data.length, 8); e.writeUInt32LE(offset, 12);
  offset += data.length;
  return e;
});
writeFileSync(join(icons, "icon.ico"), Buffer.concat([head, ...dir, ...entries.map((e) => e.data)]));

// 5. the About mark
mkdirSync(brand, { recursive: true });
for (const s of [192, 288, 384]) copyFileSync(join(tmp, "full-px", `${s}x${s}.png`), join(brand, `revelation-mark-${s}.png`));

rmSync(tmp, { recursive: true, force: true });
console.log("wrote src-tauri/icons/* and public/brand/revelation-mark-{192,288,384}.png");
