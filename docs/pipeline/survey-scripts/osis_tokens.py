"""Shared, fail-closed helpers for the survey and oracle scripts.

- check_sha() / load_pinned(): read a pinned source once, hash those bytes,
  compare with its sources/checksums/<ID>.sha256 file, refuse on mismatch.
- tokenize(): strict regex tokenizer for the CrossWire KJV OSIS file.
  No XML parser, no DTD, no entity expansion. The file is treated as bytes.
- StrictWalker / strict_tokenize(): the one structural check every
  KJV-reading script runs over the whole token list before it writes anything:
  every element name and token kind is checked against the KJV profile
  (allowlist), every end tag must match the top of the stack, and nested,
  empty, milestone or unexpected-attribute `q` is rejected (pen-test findings
  M1-F3 and M1-F13).

Everything raises Refused (or ValueError from the tokenizer) on anything it
does not recognise, rather than guessing.
"""
import hashlib
import hmac
import os
import re


class Refused(ValueError):
    """A checksum, profile or structure check failed. Never catch and continue."""


# ---------------------------------------------------------------- checksums

# Exactly: 64 lowercase hex digits, two spaces, a plain file name, one LF.
SHA_LINE = re.compile(rb"([0-9a-f]{64})  ([A-Za-z0-9][A-Za-z0-9._-]{0,80})\n")
HEX64 = re.compile(r"[0-9a-f]{64}")
MAX_SOURCE_BYTES = 64 * 1024 * 1024  # the largest pinned file is about 28 MB


def read_checksum_file(sumfile, name):
    """Return the hex digest from a sources/checksums/<ID>.sha256 file.

    Refuses unless the file is exactly one line in the format above and the
    name in it is exactly `name`.
    """
    with open(sumfile, "rb") as f:
        raw = f.read(4097)
    m = SHA_LINE.fullmatch(raw)
    if not m:
        raise Refused("malformed checksum file %s; refusing" % sumfile)
    if m.group(2).decode("ascii") != name:
        raise Refused("checksum file %s names %r, not %r; refusing"
                      % (sumfile, m.group(2).decode("ascii"), name))
    return m.group(1).decode("ascii")


def check_sha(path, expected_hex):
    """Read `path` once, hash exactly those bytes, return them only if the
    SHA-256 equals `expected_hex` (taken from the checksum file)."""
    if not isinstance(expected_hex, str) or not HEX64.fullmatch(expected_hex):
        raise Refused("expected SHA-256 is not 64 lowercase hex digits; refusing")
    with open(path, "rb") as f:
        data = f.read(MAX_SOURCE_BYTES + 1)
    if len(data) > MAX_SOURCE_BYTES:
        raise Refused("%s is larger than %d bytes; refusing" % (path, MAX_SOURCE_BYTES))
    got = hashlib.sha256(data).hexdigest()
    if not hmac.compare_digest(got, expected_hex):
        raise Refused("SHA-256 mismatch for %s (got %s, expected %s); refusing"
                      % (path, got, expected_hex))
    return data


def load_pinned(path, sumfile):
    """check_sha() against the checksum file; the file name must match it."""
    return check_sha(path, read_checksum_file(sumfile, os.path.basename(path)))


# ---------------------------------------------------------------- tokenizer

TAG = re.compile(
    rb'<(/?)([A-Za-z_][\w.:-]*)((?:\s+[A-Za-z_][\w.:-]*="[^"<&]*")*)\s*(/?)>'
)
ATTR = re.compile(rb'([A-Za-z_][\w.:-]*)="([^"<&]*)"')
XMLDECL = b'<?xml version="1.0" encoding="UTF-8"?>'


class Tok:
    __slots__ = ("kind", "name", "attrs", "start", "end")

    def __init__(self, kind, name, attrs, start, end):
        self.kind = kind  # 'start' | 'end' | 'empty' | 'text'
        self.name = name
        self.attrs = attrs
        self.start = start
        self.end = end


def tokenize(data):
    if not data.startswith(XMLDECL):
        raise ValueError("unexpected XML declaration")
    if b"&" in data:
        raise ValueError("entity or character reference present")
    for bad in (b"<!", b"<?xml-", b"]]>"):
        if bad in data[len(XMLDECL):]:
            raise ValueError("unexpected markup %r" % bad)
    if data.count(b"<?") != 1:
        raise ValueError("unexpected processing instruction")
    toks = []
    pos = len(XMLDECL)
    n = len(data)
    while pos < n:
        lt = data.find(b"<", pos)
        if lt == -1:
            lt = n
        if lt > pos:
            toks.append(Tok("text", None, None, pos, lt))
        if lt == n:
            break
        m = TAG.match(data, lt)
        if not m:
            raise ValueError("unparseable tag at byte %d: %r" % (lt, data[lt:lt + 80]))
        slash, name, attrs, empty = m.groups()
        if slash and (attrs.strip() or empty):
            raise ValueError("bad end tag at %d" % lt)
        ad = {}
        for am in ATTR.finditer(attrs):
            k = am.group(1).decode()
            if k in ad:
                raise ValueError("duplicate attribute at %d" % lt)
            ad[k] = am.group(2).decode("utf-8")
        kind = "end" if slash else ("empty" if empty else "start")
        toks.append(Tok(kind, name.decode(), ad, lt, m.end()))
        pos = m.end()
    return toks


# ---------------------------------------------------------------- strict walker

# KJV profile (BIB-KJV-CW31): every element name in the file, and the token
# kinds it may appear as. "start" also allows its matching "end". Any name not
# listed, or any kind not listed for it, is refused. Derived from the survey
# (docs/pipeline/kjv-osis-survey.md) and pen-test finding M1-F3:
#   verse, milestone: Empty only; w: either; everything else: Start/End only.
START_END = frozenset({"start"})
KJV_PROFILE = {
    "verse": frozenset({"empty"}),
    "milestone": frozenset({"empty"}),
    "w": frozenset({"start", "empty"}),
}
for _name in ("q", "transChange", "divineName", "note", "inscription",
              "osis", "osisText", "header", "work", "title", "identifier",
              "refSystem", "scope", "revisionDesc", "date", "p", "div",
              "chapter", "catchWord", "rdg", "foreign", "abbr"):
    KJV_PROFILE[_name] = START_END
del _name

# The only q this profile supports: a container with exactly these attributes.
KJV_Q_ATTRS = {"who": "Jesus", "marker": ""}


class StrictWalker:
    """Feed every token to step(), then call finish(). Raises Refused.

    Public state: `stack` (open element names), `verse` (open verse sID or
    None), `q_open` (byte offset of the open q start tag, or None).
    """

    def __init__(self, profile=None, q_attrs=None):
        self.profile = KJV_PROFILE if profile is None else profile
        self.q_attrs = KJV_Q_ATTRS if q_attrs is None else q_attrs
        self.stack = []
        self.verse = None
        self.q_open = None
        self.note_depth = 0

    def step(self, t):
        if t.kind == "text":
            return
        name = t.name
        at = t.start
        allowed = self.profile.get(name)
        if allowed is None:
            raise Refused("element <%s> at byte %d is not in the profile" % (name, at))

        if t.kind == "end":
            if "start" not in allowed:
                raise Refused("end tag </%s> at byte %d: <%s> may only be empty"
                              % (name, at, name))
            if not self.stack or self.stack[-1] != name:
                raise Refused("mismatched end tag </%s> at byte %d (open: %s)"
                              % (name, at, self.stack[-1] if self.stack else None))
            self.stack.pop()
            if name == "q":
                self.q_open = None
            elif name == "note":
                self.note_depth -= 1
            return

        if name == "q":
            if "sID" in t.attrs or "eID" in t.attrs:
                raise Refused("milestone q (sID/eID) at byte %d is not supported" % at)
            if t.attrs != self.q_attrs:
                raise Refused("q at byte %d has attributes %r, expected exactly %r"
                              % (at, t.attrs, self.q_attrs))

        if t.kind not in allowed:
            raise Refused("<%s> at byte %d as %s token is not allowed (allowed: %s)"
                          % (name, at, t.kind, "/".join(sorted(allowed))))

        if name == "verse":
            s, e = t.attrs.get("sID"), t.attrs.get("eID")
            if (s is None) == (e is None):
                raise Refused("verse at byte %d needs exactly one of sID/eID" % at)
            if s is not None:
                if self.verse is not None:
                    raise Refused("verse %s opened inside %s at byte %d" % (s, self.verse, at))
                self.verse = s
            else:
                if e != self.verse:
                    raise Refused("verse eID %s at byte %d does not close %s" % (e, at, self.verse))
                self.verse = None

        if name == "q":
            if self.q_open is not None:
                raise Refused("nested q at byte %d (outer q at byte %d)" % (at, self.q_open))
            if self.note_depth:
                raise Refused("q inside note at byte %d" % at)

        if t.kind == "start":
            self.stack.append(name)
            if name == "q":
                self.q_open = at
            elif name == "note":
                self.note_depth += 1

    def finish(self):
        if self.stack:
            raise Refused("unclosed at EOF: %s" % self.stack)
        if self.verse is not None:
            raise Refused("verse %s not closed at EOF" % self.verse)


def strict_tokenize(data, profile=None):
    """tokenize() plus a full StrictWalker pass. Returns the token list only if
    the whole file passes, so callers can check everything before writing."""
    toks = tokenize(data)
    w = StrictWalker(profile)
    for t in toks:
        w.step(t)
    w.finish()
    return toks
