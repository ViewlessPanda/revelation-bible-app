// KJV GPL notices (licenses/KJV-NOTICE.md): the README addition and the About
// "License" paragraph must equal the wording in that file, read from the file
// itself (an edit on any side fails until all agree), and licenses/GPL-3.0.txt
// must be the FSF text, byte for byte.
import { describe, expect, it } from "vitest";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { KJV_LICENCE_NOTICE } from "../src/lib/attributions";

const repo = (p: string) => readFileSync(resolve(process.cwd(), "..", p), "utf8").replace(/\r\n/g, "\n");
/** https://www.gnu.org/licenses/gpl-3.0.txt, fetched 2026-10-24 (35,149 bytes; also in scripts/copy-content.mjs). */
const GPL_SHA256 = "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986";

/** The blockquotes of licenses/KJV-NOTICE.md, each joined into one paragraph. */
function drafts(): string[] {
  const doc = repo("licenses/KJV-NOTICE.md");
  const out: string[] = [];
  let cur: string[] = [];
  for (const l of doc.split("\n")) {
    if (l.startsWith(">")) cur.push(l.slice(1).trim());
    else if (cur.length) { out.push(cur.join(" ")); cur = []; }
  }
  if (cur.length) out.push(cur.join(" "));
  return out;
}
const flat = (s: string) => s.replace(/\s+/g, " ").trim();
/** Markdown to the text a reader sees: **bold**, `code`, [label](link). */
const plain = (s: string) => s.replace(/\*\*([^*]+)\*\*/g, "$1").replace(/`([^`]+)`/g, "$1").replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");

describe("KJV GPL notices equal licenses/KJV-NOTICE.md", () => {
  const [about, readme] = drafts();

  it("finds both drafts", () => {
    expect(about.startsWith("**License.** The King James Version text")).toBe(true);
    expect(readme.startsWith("**King James Version.** The text")).toBe(true);
  });

  it("About: the License paragraph, word for word", () => {
    expect(`${KJV_LICENCE_NOTICE.label} ${KJV_LICENCE_NOTICE.text}`).toBe(plain(about));
  });

  it("README: the License section's KJV bullet carries the paragraph, Markdown and all", () => {
    const md = repo("README.md");
    const lic = flat(md.slice(md.indexOf("## License")));
    expect(lic).toContain(`- ${flat(readme)} - **Strong's Hebrew Dictionary.**`);
  });
});

describe("licenses/GPL-3.0.txt", () => {
  it("is the FSF's GPL v3 text, unchanged", () => {
    const bytes = readFileSync(resolve(process.cwd(), "../licenses/GPL-3.0.txt"));
    expect(bytes.length).toBe(35149);
    expect(createHash("sha256").update(bytes).digest("hex")).toBe(GPL_SHA256);
    const text = bytes.toString("utf8");
    expect(text.startsWith("                    GNU GENERAL PUBLIC LICENSE\n                       Version 3, 29 June 2007\n")).toBe(true);
    expect(text).toContain("Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>");
    expect(text.includes("\r")).toBe(false);
  });

  it("copy-content places it beside kjv.sqlite, hash-checked", () => {
    const s = readFileSync(resolve(process.cwd(), "scripts/copy-content.mjs"), "utf8");
    expect(s).toContain(`const GPL_SHA256 = "${GPL_SHA256}"`);
    expect(s).toMatch(/join\(root, "licenses", "GPL-3\.0\.txt"\)/);
    expect(s).toMatch(/plan\.push\(\{ file: "GPL-3\.0\.txt"/);
  });
});
