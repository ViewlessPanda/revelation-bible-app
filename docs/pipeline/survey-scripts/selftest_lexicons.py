"""Self-test for the fail-closed checks in oracle_lexicons.py.

Usage:
  py -I -B selftest_lexicons.py <mgnt-prefix.bin> <work_dir>

<mgnt-prefix.bin> is pipeline/tests/fixtures/lexicon/mgnt-prefix.bin (the
first 1,090 bytes of strongsgreek.xml: declaration, LF, pinned DTD subset;
CC0). <work_dir> must not exist yet; the test creates it and writes only
small synthetic files there (placeholder words, no lexicon text). No pinned
source is read. Exit status 0 only if every check passes.

Checks: the strict walker accepts the two good fixtures and refuses each bad
one with the expected reason; main() refuses a tampered checksum and the
synthetic files' entry counts (they are not 8,674 / 5,624) and writes no
output in either case.
"""
import hashlib
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import oracle_lexicons  # noqa: E402
from osis_tokens import Refused  # noqa: E402

RESULTS = []

ROOT = (b'<lexicon xsi:schemaLocation="http://openscriptures.github.com/morphhb/namespace StrongSchema.xsd" '
        b'xmlns="http://openscriptures.github.com/morphhb/namespace" '
        b'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">')
H1 = (b'<entry id="H1">\r\n\t\t<w pos="n" pron="a" xlit="a" xml:lang="heb">alpha</w>\r\n'
      b'\t\t<source>a <w src="H2">2</w></source>\r\n\t\t<meaning><def>beta</def></meaning>\r\n\t</entry>')
H2 = b'<entry id="H2">\r\n\t\t<source/>\r\n\t\t<usage>gamma.</usage>\r\n\t</entry>'
G1 = (b'<entry strongs="00001">\n <strongs>1</strongs> <greek BETA="A" unicode="a" translit="a"/>'
      b' <strongs_def> delta\n epsilon</strongs_def> zeta <strongsref language="HEBREW" strongs="01"/></entry>')


def oshb(entries=H1 + b"\r\n\t" + H2):
    return b'<?xml version="1.0" encoding="utf-8"?>\r\n' + ROOT + b"\r\n\t" + entries + b"\r\n</lexicon>"


def mgnt(prefix, entries=G1, between=b""):
    return (prefix + b"\n\n" + between + b"<strongsdictionary><prologue>\n\tx\n</prologue><entries>"
            + entries + b"</entries></strongsdictionary>\n")


def check(label, ok, detail=""):
    RESULTS.append((label, ok))
    print("%s  %s%s" % ("PASS" if ok else "FAIL", label, (" -- " + detail) if detail and not ok else ""))


def refused(label, fn, *substrings):
    try:
        fn()
    except Refused as e:
        msg = str(e)
        check(label, all(s in msg for s in substrings), msg)
        return
    check(label, False, "accepted")


def main(prefix_path, work):
    os.makedirs(work)  # must not exist
    with open(prefix_path, "rb") as f:
        prefix = f.read()
    if len(prefix) != 1090:
        sys.exit("mgnt-prefix.bin must be 1,090 bytes")
    w = oracle_lexicons.walk
    he = w(oshb(), "OSHB")
    check("good OSHB fixture walks", [e["key"] for e in he] == ["H1", "H2"])
    check("OSHB CR kept in text", b"".join(he[0]["text"]).startswith(b"\r\n\t\t"))
    check("empty <source/> as <source></source> in the tree", b"<source></source>" in b"".join(he[1]["tree"]))
    ge = w(mgnt(prefix), "MGNT")
    check("good MGNT fixture walks", [e["key"] for e in ge] == ["G1"])
    check("MGNT attrs in document order", ge[0]["attrs"][:2] == ["entry@strongs=00001", "greek@BETA=A"])

    def o(old, new):
        return lambda: w(oshb(H1.replace(old, new) + b"\r\n\t" + H2), "OSHB")

    def g(old, new):
        return lambda: w(mgnt(prefix, G1.replace(old, new)), "MGNT")

    refused("unknown element", o(b"<def>beta</def>", b"<b>beta</b>"), "not in the profile")
    refused("element in the wrong parent", o(b"<def>beta</def>", b"<usage>beta</usage>"), "not allowed in <meaning>")
    refused("end tag mismatch", o(b"<def>beta</def>", b"<def>beta</note>"), "mismatched end tag")
    refused("attribute not allowed", o(b"<source>", b'<source x="1">'), "attribute x")
    refused("entry id padded", o(b'id="H1"', b'id="H01"'), "pinned spelling")
    refused("entries descending", lambda: w(oshb(H2 + b"\r\n\t" + H1), "OSHB"), "not ascending")
    refused("text outside entries", lambda: w(oshb(H1 + b"stray" + H2), "OSHB"), "text outside an entry")
    refused("entity", o(b"beta", b"be&amp;ta"), "'&'")
    refused("comment", o(b"beta", b"be<!-- x -->ta"), "'<!'")
    refused("OSHB declaration", lambda: w(oshb().replace(b'"utf-8"', b'"UTF-8"', 1), "OSHB"), "declaration")
    refused("greek as a container", g(b'translit="a"/>', b'translit="a"></greek>'), "as start is not allowed")
    refused("entry strongs not five digits", g(b'strongs="00001"', b'strongs="0001"'), "pinned spelling")
    changed = bytearray(mgnt(prefix))
    changed[300] ^= 0x20
    refused("DTD subset changed", lambda: w(bytes(changed), "MGNT"), "DTD subset differs")
    refused("DTD subset moved", lambda: w(prefix[:55] + b"\n" + mgnt(prefix)[55:], "MGNT"), "DTD subset differs")
    refused("DOCTYPE after byte 1090", lambda: w(mgnt(prefix, between=b"<!DOCTYPE x>"), "MGNT"), "'<!'")
    refused("PI after byte 1090", lambda: w(mgnt(prefix, between=b"<?x y?>"), "MGNT"), "'<?'")

    # main(): checksum first, counts before writing; no output either way.
    src = os.path.join(work, "src")
    os.makedirs(src)
    paths = {}
    for name, data in (("HebrewStrong.xml", oshb()), ("strongsgreek.xml", mgnt(prefix))):
        p = os.path.join(src, name)
        with open(p, "wb") as f:
            f.write(data)
        digest = hashlib.sha256(data).hexdigest()
        good = os.path.join(src, name + ".sha256")
        with open(good, "wb") as f:
            f.write(("%s  %s\n" % (digest, name)).encode("ascii"))
        bad = os.path.join(src, name + ".bad.sha256")
        with open(bad, "wb") as f:
            f.write(("%s  %s\n" % ("0" * 64, name)).encode("ascii"))
        paths[name] = (p, good, bad)
    out = os.path.join(work, "out")
    os.makedirs(out)
    h, g_ = paths["HebrewStrong.xml"], paths["strongsgreek.xml"]
    refused("main refuses a tampered checksum", lambda: oracle_lexicons.main(h[0], h[2], g_[0], g_[1], out), "SHA-256 mismatch")
    refused("main refuses the synthetic entry counts", lambda: oracle_lexicons.main(h[0], h[1], g_[0], g_[1], out), "unexpected entry counts")
    check("main wrote nothing", os.listdir(out) == [])

    failed = [label for label, ok in RESULTS if not ok]
    print("%d/%d checks passed" % (len(RESULTS) - len(failed), len(RESULTS)))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
