"""Self-test for the fail-closed checks in osis_tokens.py and their use by the
survey/oracle scripts (pen-test findings M1-F3 and M1-F13).

Usage:
  py -I -B selftest_strict.py <fixtures_dir> <work_dir>

<fixtures_dir> is docs/pipeline/survey-scripts/fixtures/. <work_dir> must not
exist yet; the test creates it and writes only small synthetic files there.
No pinned source is read. Exit status 0 only if every check passes.

What it checks:
 1. Every fixture is listed below, and every listed fixture exists. Fixtures
    are ASCII, LF only, synthetic placeholder text.
 2. The plain tokenizer accepts every fixture (so the failures below come from
    the strict walker, not from the tokenizer).
 3. strict_tokenize() accepts the ok-* fixtures and refuses each fail-*
    fixture with the expected reason.
 4. check_sha() / read_checksum_file() refuse a wrong digest and every
    malformed checksum-file shape tried.
 5. Wiring: every script that reads a pinned source refuses a tampered
    checksum and writes no output; every KJV-reading script refuses each
    fail-* fixture (with a correct checksum) and writes no output; survey_kjv
    still produces its report for the ok-* fixtures.
"""
import hashlib
import io
import os
import sys
import contextlib

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import osis_tokens  # noqa: E402
from osis_tokens import Refused, check_sha, read_checksum_file, strict_tokenize, tokenize  # noqa: E402
import basic_facts  # noqa: E402
import golden_excerpts  # noqa: E402
import oracle_kjv  # noqa: E402
import survey_excerpts  # noqa: E402
import survey_kjv  # noqa: E402
import survey_kjv_detail  # noqa: E402
import survey_lexicons  # noqa: E402
import survey_misc  # noqa: E402

# fixture -> None (must pass) or the substrings the refusal message must contain.
EXPECT = {
    "ok-all-elements.xml": None,
    "ok-q-across-verses.xml": None,
    "fail-q-empty.xml": ("<q> at byte", "as empty token is not allowed"),
    "fail-q-milestone.xml": ("milestone q (sID/eID)",),
    "fail-q-milestone-start.xml": ("milestone q (sID/eID)",),
    "fail-q-nested.xml": ("nested q",),
    "fail-q-in-note.xml": ("q inside note",),
    "fail-q-who-lowercase.xml": ("has attributes", "'jesus'"),
    "fail-q-extra-attr.xml": ("has attributes", "'level'"),
    "fail-q-namespaced.xml": ("<x:q>", "is not in the profile"),
    "fail-transchange-empty.xml": ("<transChange> at byte", "as empty token is not allowed"),
    "fail-divinename-empty.xml": ("<divineName> at byte", "as empty token is not allowed"),
    "fail-note-empty.xml": ("<note> at byte", "as empty token is not allowed"),
    "fail-inscription-empty.xml": ("<inscription> at byte", "as empty token is not allowed"),
    "fail-verse-container.xml": ("<verse> at byte", "as start token is not allowed"),
    "fail-milestone-container.xml": ("<milestone> at byte", "as start token is not allowed"),
    "fail-end-tag-mismatch.xml": ("mismatched end tag </divineName>",),
    "fail-end-tag-mismatch-q.xml": ("mismatched end tag </q>",),
    "fail-verse-eid-mismatch.xml": ("verse eID Test.1.2", "does not close Test.1.1"),
    "fail-unclosed-at-eof.xml": ("unclosed at EOF",),
}

results = []


def check(label, ok, detail=""):
    results.append(ok)
    print("%s  %s%s" % ("PASS" if ok else "FAIL", label, (" -- " + detail) if detail else ""))


def expect_refused(label, fn, substrings=(), must_not_exist=()):
    """fn() must raise Refused whose message contains every substring, and
    none of the paths in must_not_exist may exist (or be non-empty dirs) after."""
    sink = io.StringIO()
    try:
        with contextlib.redirect_stdout(sink):
            fn()
    except Refused as e:
        msg = str(e)
        missing = [s for s in substrings if s not in msg]
        leaked = [p for p in must_not_exist
                  if os.path.exists(p) and not (os.path.isdir(p) and not os.listdir(p))]
        check(label, not missing and not leaked and not sink.getvalue(),
              "missing %r" % missing if missing else
              "output written: %r" % leaked if leaked else
              "printed output before refusing" if sink.getvalue() else msg)
        return
    except Exception as e:  # any other exception is a test failure: wrong check fired
        check(label, False, "raised %s: %s (expected Refused)" % (type(e).__name__, e))
        return
    check(label, False, "did not raise")


def sha_line(data, name):
    return ("%s  %s\n" % (hashlib.sha256(data).hexdigest(), name)).encode("ascii")


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "xb") as f:
        f.write(data)


def make_sources(root, kjv_data, tamper=False):
    """A minimal sources/ tree: raw/<ID>/<name> plus checksums/<ID>.sha256.
    Lexicon and versification files are placeholders; only their checksums
    matter here. With tamper=True the KJV checksum is for different bytes."""
    files = {
        ("BIB-KJV-CW31", "kjv.osis.xml"): kjv_data,
        ("LEX-HEB-OSHB", "HebrewStrong.xml"): b"placeholder alpha\n",
        ("LEX-GRC-MGNT19", "strongsgreek.xml"): b"placeholder beta\n",
        ("VRS-ENG-PALASO", "eng.vrs.txt"): b"placeholder gamma\n",
    }
    for (sid, name), data in files.items():
        write(os.path.join(root, "raw", sid, name), data)
        summed = data + b"tampered" if (tamper and sid == "BIB-KJV-CW31") else data
        write(os.path.join(root, "checksums", sid + ".sha256"), sha_line(summed, name))
    return root


def paths(root):
    raw = os.path.join(root, "raw")
    cs = os.path.join(root, "checksums")
    return {
        "kjv": os.path.join(raw, "BIB-KJV-CW31", "kjv.osis.xml"),
        "kjv_sum": os.path.join(cs, "BIB-KJV-CW31.sha256"),
        "heb": os.path.join(raw, "LEX-HEB-OSHB", "HebrewStrong.xml"),
        "heb_sum": os.path.join(cs, "LEX-HEB-OSHB.sha256"),
        "grc": os.path.join(raw, "LEX-GRC-MGNT19", "strongsgreek.xml"),
        "grc_sum": os.path.join(cs, "LEX-GRC-MGNT19.sha256"),
        "vrs": os.path.join(raw, "VRS-ENG-PALASO", "eng.vrs.txt"),
        "vrs_sum": os.path.join(cs, "VRS-ENG-PALASO.sha256"),
    }


def kjv_script_runs(p, out):
    """(script label, callable, outputs that must not appear) for every script
    that tokenizes the KJV."""
    os.makedirs(os.path.join(out, "survey_kjv"))
    os.makedirs(os.path.join(out, "oracle"))
    return [
        ("oracle_kjv", lambda: oracle_kjv.main(p["kjv"], p["kjv_sum"], os.path.join(out, "oracle")),
         [os.path.join(out, "oracle", "kjv-chapters.tsv"), os.path.join(out, "oracle", "kjv-q.tsv")]),
        ("survey_kjv", lambda: survey_kjv.main(p["kjv"], p["kjv_sum"], os.path.join(out, "survey_kjv")),
         [os.path.join(out, "survey_kjv")]),
        ("survey_kjv_detail", lambda: survey_kjv_detail.main(
            p["kjv"], p["kjv_sum"], p["vrs"], p["vrs_sum"], os.path.join(out, "detail.txt")),
         [os.path.join(out, "detail.txt")]),
        ("survey_misc", lambda: survey_misc.main(os.path.dirname(os.path.dirname(p["kjv_sum"])),
                                                 os.path.join(out, "misc.txt")),
         [os.path.join(out, "misc.txt")]),
        ("golden_excerpts", lambda: golden_excerpts.main(
            p["kjv"], p["kjv_sum"], os.path.join(out, "no-golden-file.md"), os.path.join(out, "golden.md")),
         [os.path.join(out, "golden.md")]),
    ]


def main(fixtures_dir, work_dir):
    if os.path.exists(work_dir):
        sys.exit("work_dir %s already exists; give a new path" % work_dir)
    os.makedirs(work_dir)

    # 1. inventory and byte shape
    present = sorted(n for n in os.listdir(fixtures_dir) if not n.startswith("."))
    check("fixture inventory matches EXPECT", present == sorted(EXPECT),
          "unlisted %s, missing %s" % (sorted(set(present) - set(EXPECT)), sorted(set(EXPECT) - set(present))))
    data = {}
    for name in sorted(EXPECT):
        path = os.path.join(fixtures_dir, name)
        if not os.path.isfile(path):
            continue
        with open(path, "rb") as f:
            d = f.read()
        data[name] = d
        check("bytes %s: ASCII, LF only, XML declaration first" % name,
              b"\r" not in d and max(d) < 128 and d.startswith(osis_tokens.XMLDECL))

    # 2. the tokenizer alone accepts every fixture
    for name, d in data.items():
        try:
            tokenize(d)
            check("tokenize() alone accepts %s" % name, True)
        except Exception as e:
            check("tokenize() alone accepts %s" % name, False, str(e))

    # 3. strict walker
    for name, d in data.items():
        if EXPECT[name] is None:
            try:
                strict_tokenize(d)
                check("strict_tokenize accepts %s" % name, True)
            except Exception as e:
                check("strict_tokenize accepts %s" % name, False, str(e))
        else:
            expect_refused("strict_tokenize refuses %s" % name,
                           lambda d=d: strict_tokenize(d), EXPECT[name])

    # 4. checksum helpers
    ck = os.path.join(work_dir, "checksum")
    good = b"alpha beta gamma\n"
    write(os.path.join(ck, "data.txt"), good)
    hexd = hashlib.sha256(good).hexdigest()
    sums = {
        "good.sha256": sha_line(good, "data.txt"),
        "wrong-digest.sha256": sha_line(good + b"x", "data.txt"),
        "uppercase.sha256": ("%s  data.txt\n" % hexd.upper()).encode(),
        "crlf.sha256": ("%s  data.txt\r\n" % hexd).encode(),
        "one-space.sha256": ("%s data.txt\n" % hexd).encode(),
        "no-lf.sha256": ("%s  data.txt" % hexd).encode(),
        "two-lines.sha256": ("%s  data.txt\n%s  data.txt\n" % (hexd, hexd)).encode(),
        "other-name.sha256": sha_line(good, "other.txt"),
        "star-binary.sha256": ("%s *data.txt\n" % hexd).encode(),
    }
    for n, b in sums.items():
        write(os.path.join(ck, n), b)
    dpath = os.path.join(ck, "data.txt")
    try:
        got = osis_tokens.load_pinned(dpath, os.path.join(ck, "good.sha256"))
        check("load_pinned accepts a correct checksum file", got == good)
    except Exception as e:
        check("load_pinned accepts a correct checksum file", False, str(e))
    expect_refused("load_pinned refuses wrong digest",
                   lambda: osis_tokens.load_pinned(dpath, os.path.join(ck, "wrong-digest.sha256")),
                   ("SHA-256 mismatch",))
    for n, why in (("uppercase", "malformed"), ("crlf", "malformed"), ("one-space", "malformed"),
                   ("no-lf", "malformed"), ("two-lines", "malformed"), ("star-binary", "malformed"),
                   ("other-name", "names 'other.txt'")):
        expect_refused("read_checksum_file refuses %s" % n,
                       lambda n=n: read_checksum_file(os.path.join(ck, n + ".sha256"), "data.txt"), (why,))
    expect_refused("check_sha refuses a non-hex expected value",
                   lambda: check_sha(dpath, "z" * 64), ("not 64 lowercase hex",))
    expect_refused("check_sha refuses uppercase expected value",
                   lambda: check_sha(dpath, hexd.upper()), ("not 64 lowercase hex",))

    # 5a. wiring: a tampered KJV checksum is refused by every script, no output
    ok_kjv = data.get("ok-all-elements.xml", b"")
    root = make_sources(os.path.join(work_dir, "tampered", "sources"), ok_kjv, tamper=True)
    p = paths(root)
    out = os.path.join(work_dir, "tampered", "out")
    for label, fn, outs in kjv_script_runs(p, out):
        expect_refused("%s refuses a tampered checksum" % label, fn, ("SHA-256 mismatch",), outs)
    expect_refused("basic_facts refuses a tampered checksum",
                   lambda: basic_facts.main(p["kjv"], p["kjv_sum"]), ("SHA-256 mismatch",))
    expect_refused("survey_excerpts refuses a tampered checksum",
                   lambda: survey_excerpts.main(root, os.path.join(out, "excerpts.md")),
                   ("SHA-256 mismatch",), [os.path.join(out, "excerpts.md")])
    # Lexicon script: tamper the Hebrew checksum in a separate tree.
    lroot = make_sources(os.path.join(work_dir, "tampered-lex", "sources"), ok_kjv)
    lp = paths(lroot)
    with open(lp["heb_sum"], "wb") as f:
        f.write(sha_line(b"different bytes", "HebrewStrong.xml"))
    expect_refused("survey_lexicons refuses a tampered checksum",
                   lambda: survey_lexicons.main(lp["heb"], lp["heb_sum"], lp["grc"], lp["grc_sum"],
                                                os.path.join(work_dir, "tampered-lex", "lex.txt")),
                   ("SHA-256 mismatch",), [os.path.join(work_dir, "tampered-lex", "lex.txt")])

    # 5b. wiring: each fail-* fixture, correctly checksummed, is refused by
    # every KJV-reading script before it writes anything.
    for name, d in data.items():
        tag = name[:-4]
        root = make_sources(os.path.join(work_dir, "wired", tag, "sources"), d)
        p = paths(root)
        out = os.path.join(work_dir, "wired", tag, "out")
        if EXPECT[name] is None:
            continue
        for label, fn, outs in kjv_script_runs(p, out):
            expect_refused("%s refuses %s" % (label, name), fn, EXPECT[name], outs)

    # 5c. positive control: survey_kjv runs on the ok-* fixtures.
    for name in ("ok-all-elements.xml", "ok-q-across-verses.xml"):
        if name not in data:
            continue
        root = os.path.join(work_dir, "wired", name[:-4], "sources")
        p = paths(root)
        out = os.path.join(work_dir, "wired", name[:-4], "survey_kjv")
        os.makedirs(out)
        try:
            survey_kjv.main(p["kjv"], p["kjv_sum"], out)
            with open(os.path.join(out, "survey.txt"), "rb") as f:
                rep = f.read()
            check("survey_kjv reports on %s" % name, b"q spans: 1\n" in rep and b"problems: 0\n" in rep)
        except Exception as e:
            check("survey_kjv reports on %s" % name, False, "%s: %s" % (type(e).__name__, e))

    n_fail = results.count(False)
    print("\n%d checks, %d passed, %d failed" % (len(results), len(results) - n_fail, n_fail))
    return 1 if n_fail or not results else 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1], sys.argv[2]))
