// About dialog and the bundled attribution lines (lib/attributions.ts).
// Every credit string must equal the "Required attribution" text of its
// SOURCES.md row (read here from sources/SOURCES.md, so an edit on either side
// fails until both agree); the font lines must quote the row's copyright line
// and upstream link (SOURCES.md F3). The app's copyright line must be LICENSE's.
import { describe, expect, it } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import AboutDialog from "../src/components/AboutDialog.svelte";
import {
  APP_COPYRIGHT, FONT_CREDITS, KJV_CREDIT, LEXICON_CREDITS, MGNT_CREDIT, MHCC_CREDIT, OSHB_CREDIT,
} from "../src/lib/attributions";

const repo = (p: string) => readFileSync(resolve(process.cwd(), "..", p), "utf8").replace(/\r\n/g, "\n");
const SOURCES = repo("sources/SOURCES.md");

/** The text of one `### <ID>:` row, up to the next `### ` heading. */
function row(id: string): string {
  const start = SOURCES.indexOf(`### ${id}:`);
  expect(start, id).toBeGreaterThan(0);
  const end = SOURCES.indexOf("\n### ", start + 4);
  return SOURCES.slice(start, end);
}

/** The blockquote paragraphs under the row's "Required attribution" field. */
function requiredAttribution(id: string): string[] {
  const r = row(id);
  const lines = r.slice(r.indexOf("- **Required attribution:**")).split("\n").slice(1);
  const paras: string[] = [];
  let cur: string[] = [];
  for (const l of lines) {
    if (!l.startsWith("  >")) break;
    const s = l.slice(3).trim();
    if (s === "") { paras.push(cur.join(" ")); cur = []; } else cur.push(s);
  }
  if (cur.length) paras.push(cur.join(" "));
  return paras;
}

describe("attribution lines equal SOURCES.md", () => {
  it.each([
    ["BIB-KJV-CW31", KJV_CREDIT],
    ["LEX-HEB-OSHB", OSHB_CREDIT],
    ["LEX-GRC-MGNT19", MGNT_CREDIT],
    ["CMT-MHCC-CW20", MHCC_CREDIT],
  ] as const)("%s", (id, credit) => {
    expect(credit.id).toBe(id);
    const want = requiredAttribution(id);
    expect(want.length).toBeGreaterThan(0);
    expect(credit.paragraphs).toEqual(want);
  });

  it("the KJV credit keeps both paragraphs (CrossWire notice and Crown rights)", () => {
    expect(KJV_CREDIT.paragraphs.length).toBe(2);
    expect(LEXICON_CREDITS["LEX-HEB-OSHB"]).toBe(OSHB_CREDIT);
    expect(LEXICON_CREDITS["LEX-GRC-MGNT19"]).toBe(MGNT_CREDIT);
  });

  it.each(FONT_CREDITS.map((f) => [f.id, f] as const))("%s: copyright line, licence and link from the row (F3)", (id, f) => {
    const r = row(id);
    const licence = r.split("\n").find((l) => l.startsWith("- **License:**"))!;
    expect(licence).toContain("SIL OFL 1.1");
    expect(licence).toContain(`"${f.copyright}"`);
    const publisher = r.split("\n").find((l) => l.startsWith("- **Publisher / maintainer:**"))!;
    expect(publisher.endsWith(` ${f.url}`)).toBe(true);
    expect(r).toMatch(/- \*\*Required attribution:\*\* F2\/F3/);
    expect(f.licence).toBe("SIL Open Font License 1.1");
    expect(SOURCES).toContain("copyright line, \"SIL Open Font License 1.1\", upstream link"); // F3 wording
  });

  it("the app's copyright line is LICENSE's", () => {
    expect(repo("LICENSE").split("\n")[2]).toBe(APP_COPYRIGHT);
  });
});

describe("About dialog (wave-2-layout.md 14)", () => {
  const settle = async () => { for (let i = 0; i < 3; i++) { await Promise.resolve(); flushSync(); } };
  const button = (d: HTMLElement, label: string) => [...d.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === label)!;

  it("main view: bundled mark, the name as live text, version, credit; focus on Close", () => {
    document.body.innerHTML = '<div id="t"></div>';
    let closed = 0;
    const c = mount(AboutDialog, { target: document.getElementById("t")!, props: { onclose: () => { closed++; } } });
    flushSync();
    const d = document.querySelector<HTMLDialogElement>("dialog.about")!;
    expect(d.open).toBe(false);
    (c as unknown as { open: () => void }).open();
    flushSync();
    expect(d.open).toBe(true);
    expect(d.getAttribute("aria-labelledby")).toBe("about-title");
    const name = d.querySelector("#about-title")!;
    expect(name.tagName).toBe("H2");
    expect(name.textContent).toBe("Revelation"); // set in type, never an image
    const img = d.querySelector<HTMLImageElement>("img.about-mark")!;
    expect(img.getAttribute("alt")).toBe(""); // decorative: the name follows as text
    expect(img.getAttribute("src")).toBe("brand/revelation-mark-192.png"); // bundled, relative: no network
    expect(img.getAttribute("srcset")).toBe("brand/revelation-mark-192.png 1x, brand/revelation-mark-288.png 1.5x, brand/revelation-mark-384.png 2x");
    for (const s of [192, 288, 384]) expect(existsSync(resolve(process.cwd(), `public/brand/revelation-mark-${s}.png`))).toBe(true);
    const version = JSON.parse(readFileSync(resolve(process.cwd(), "src-tauri/tauri.conf.json"), "utf8")).version;
    expect(d.querySelector(".about-version")?.textContent).toBe(`Version ${version}`);
    expect(d.querySelector(".about-desc")?.textContent).toBe("An open-source Bible app that works offline.");
    expect(d.querySelector(".about-by")?.textContent).toBe("Made by C. Ethan Key");
    expect(d.querySelector(".about-notice")?.textContent).toBe("Icons: Lucide (ISC); includes icons from Feather (MIT)");
    expect(d.querySelectorAll("a, iframe").length).toBe(0);
    expect([...d.querySelectorAll("button")].map((b) => b.textContent)).toEqual(["Sources and licences", "Close"]);
    expect(document.activeElement?.textContent).toBe("Close");
    button(d, "Close").click();
    flushSync();
    expect(d.open).toBe(false);
    expect(closed).toBe(1); // the opener gets focus back
    unmount(c);
  });

  it("Sources and licences: the app licence, every credit and the GPL notice; the GPL text offline", async () => {
    document.body.innerHTML = '<div id="t"></div>';
    const c = mount(AboutDialog, { target: document.getElementById("t")!, props: {} });
    flushSync();
    (c as unknown as { open: () => void }).open();
    flushSync();
    const d = document.querySelector<HTMLDialogElement>("dialog.about")!;
    button(d, "Sources and licences").click();
    await settle();
    expect(d.querySelector("#about-title")?.textContent).toBe("Sources and licences");
    const text = d.textContent ?? "";
    expect(text).toContain("Copyright (c) 2026 C. Ethan Key");
    expect(text).toContain("MIT License");
    const paras = [...d.querySelectorAll<HTMLElement>(".about-credit p")].map((p) => p.textContent);
    for (const credit of [KJV_CREDIT, OSHB_CREDIT, MGNT_CREDIT, MHCC_CREDIT]) {
      for (const p of credit.paragraphs) expect(paras).toContain(p);
    }
    for (const f of FONT_CREDITS) {
      expect(d.querySelector(`[data-source="${f.id}"]`)?.textContent).toBe(`${f.name}: ${f.copyright}. ${f.licence}. ${f.url}`);
    }
    // the License paragraph sits directly under the KJV credit
    const kjv = d.querySelector('[data-source="BIB-KJV-CW31"]')!;
    expect(kjv.nextElementSibling?.classList.contains("about-licence-notice")).toBe(true);
    expect(d.querySelectorAll("a, img, iframe").length).toBe(0);
    button(d, "Read the GNU General Public License").click();
    await settle();
    const pre = d.querySelector<HTMLPreElement>("pre.about-gpl")!;
    const gpl = readFileSync(resolve(process.cwd(), "../licenses/GPL-3.0.txt"), "utf8");
    expect(pre.textContent).toBe(gpl); // the bundled file, exactly
    expect([...pre.childNodes].every((n) => n.nodeType === Node.TEXT_NODE)).toBe(true); // text nodes only
    expect(d.querySelectorAll("a, iframe").length).toBe(0);
    button(d, "Back").click();
    await settle();
    expect(d.querySelector("#about-title")?.textContent).toBe("Sources and licences");
    d.close();
    (c as unknown as { open: () => void }).open(); // reopens on the main view
    flushSync();
    expect(d.querySelector("#about-title")?.textContent).toBe("Revelation");
    d.close();
    unmount(c);
  });
});
