// Grouping of an entry's top-level nodes into panel blocks (lib/lexicon.ts).
// Synthetic placeholder nodes only (not lexicon text); the real entries are
// covered by lexicon-render.test.ts.
import { describe, expect, it } from "vitest";
import { displayAttrs, isLayoutWhitespace, lexBlocks, refKey } from "../src/lib/lexicon";
import type { LexEntry, LexNode } from "../src/lib/types";

const t = (n: number, parent: number | null, text: string): LexNode => ({ n, parent, name: null, text, attrs: [] });
const el = (n: number, parent: number | null, name: string, attrs: LexNode["attrs"] = []): LexNode =>
  ({ n, parent, name, text: null, attrs });

describe("lexicon blocks", () => {
  it("drops layout whitespace between blocks, keeps it inside a running paragraph", () => {
    const e: LexEntry = {
      key: "G1", lexicon: "LEX-GRC-MGNT19",
      nodes: [
        t(0, null, "\n "), el(1, null, "strongs"), t(2, 1, "1"), t(3, null, "   "),
        el(4, null, "greek", [{ name: "BETA", value: "b" }, { name: "unicode", value: "u" }, { name: "translit", value: "tr" }]),
        t(5, null, "\r\n\t"), el(6, null, "kjv_def"), t(7, 6, "placeholder-def"),
        t(8, null, " placeholder-tail-a "), el(9, null, "greek", [{ name: "unicode", value: "u2" }]),
        t(10, null, " "), el(11, null, "strongsref", [{ name: "language", value: "GREEK" }, { name: "strongs", value: "2", ref_key: "G2" }]),
        t(12, null, "\n\n"),
      ],
    };
    const b = lexBlocks(e);
    expect(b.map((x) => x.kind)).toEqual(["head", "field", "para"]);
    expect(b[0].kind === "head" && b[0].nodes.map((n) => n.n)).toEqual([1, 4]);
    expect(b[1].kind === "field" && [b[1].label, b[1].node.n]).toEqual(["KJV usage", 6]);
    // the " " between two paragraph nodes is kept; the trailing "\n\n" is not
    expect(b[2].kind === "para" && b[2].nodes.map((n) => n.n)).toEqual([8, 9, 10, 11]);
  });

  it("display attributes, references and layout whitespace", () => {
    const g = el(0, null, "greek", [{ name: "BETA", value: "b" }, { name: "unicode", value: "u" }, { name: "translit", value: "tr" }]);
    expect(displayAttrs(g).map((a) => a.name)).toEqual(["unicode", "translit"]); // stored order, BETA is not display text
    const w = el(1, null, "w", [{ name: "pos", value: "n" }, { name: "pron", value: "p" }, { name: "xlit", value: "x" }, { name: "xml:lang", value: "heb" }]);
    expect(displayAttrs(w).map((a) => a.name)).toEqual(["pron", "xlit"]);
    const r = el(2, 1, "w", [{ name: "src", value: "H1", ref_key: "H1" }]);
    expect(refKey(r)).toBe("H1");
    expect(displayAttrs(r)).toEqual([]);
    expect(refKey(g)).toBeNull();
    for (const s of ["", " ", "\r\n\t\t", "\n\n "]) expect(isLayoutWhitespace(s)).toBe(true);
    for (const s of ["a", "  ", " ", " x "]) expect(isLayoutWhitespace(s)).toBe(false);
  });
});
