"""Self-test for the strict MHCC reader mhcc_osis.py (DESIGN.md 17.6, T24).

Usage:
  py -I -B selftest_mhcc.py <fixtures_dir>

<fixtures_dir> is pipeline/tests/fixtures/mhcc/ (good.xml, mutations.tsv, the
same files the Rust test pipeline/tests/t24_cmt_fixtures.rs uses). No pinned
source is read and nothing is written. Exit status 0 only if:
 1. read(good.xml) succeeds;
 2. every mutations.tsv line's <old> occurs in good.xml exactly once;
 3. read() followed by parse_ref() of every osisRef (as survey_mhcc.py does)
    raises Refused for every line marked "both" (lines marked "rust" need the
    KJV versification table, which this reader does not have).
"""
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mhcc_osis  # noqa: E402
from osis_tokens import Refused  # noqa: E402

ESC = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\"}


def unescape(s):
    out, i = [], 0
    while i < len(s):
        c = s[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        k = s[i + 1]
        if k in ESC:
            out.append(ESC[k])
            i += 2
        elif k == "u" and s[i + 2] == "{":
            j = s.index("}", i)
            out.append(chr(int(s[i + 3:j], 16)))
            i = j + 1
        else:
            raise SystemExit(f"bad escape in {s!r}")
    return "".join(out)


def strict(data):
    doc = mhcc_osis.read(data, fixture=True)
    for value, _, _ in doc.refs:
        mhcc_osis.parse_ref(value)


def main(argv):
    if len(argv) != 2:
        raise SystemExit(__doc__)
    d = argv[1]
    with open(os.path.join(d, "good.xml"), "rb") as f:
        good = f.read()
    strict(good)
    with open(os.path.join(d, "mutations.tsv"), encoding="utf-8", newline="") as f:
        lines = f.read().split("\n")
    fails, both, rust = [], 0, 0
    for line in lines:
        if not line or line.startswith("#"):
            continue
        cols = line.split("\t")
        if len(cols) not in (4, 5):
            fails.append(f"bad line {line!r}")
            continue
        mid, _check, who, old = cols[:4]
        new = unescape(cols[4]) if len(cols) == 5 else ""
        old_b, new_b = unescape(old).encode("utf-8"), new.encode("utf-8")
        if good.count(old_b) != 1:
            fails.append(f"{mid}: <old> occurs {good.count(old_b)} times")
            continue
        if who == "rust":
            rust += 1
            continue
        if who != "both":
            fails.append(f"{mid}: who {who!r}")
            continue
        both += 1
        try:
            strict(good.replace(old_b, new_b, 1))
        except Refused:
            continue
        fails.append(f"{mid}: accepted")
    for f in fails:
        print("FAIL", f)
    print(f"selftest_mhcc: {both} 'both' mutations refused, {rust} 'rust' skipped, "
          f"{len(fails)} failures")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
