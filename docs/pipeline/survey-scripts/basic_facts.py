"""Byte-level facts about an XML file. No XML parser is used.

Usage: py -I -B basic_facts.py <file> <ID.sha256>
Refuses to run unless the file's SHA-256 matches its checksum file.
"""
import os
import re
import sys
from collections import Counter

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned  # noqa: E402

sys.stdout.reconfigure(encoding="utf-8", newline="\n")


def main(path, sumfile):
    data = load_pinned(path, sumfile)
    print("file:", path)
    print("size:", len(data))
    print("utf8 BOM:", data.startswith(b"\xef\xbb\xbf"))
    print("utf16 BOM:", data[:2] in (b"\xff\xfe", b"\xfe\xff"))
    try:
        data.decode("utf-8", errors="strict")
        print("valid UTF-8: yes")
    except UnicodeDecodeError as e:
        print("valid UTF-8: NO", e)
    crlf = data.count(b"\r\n")
    lf = data.count(b"\n")
    cr = data.count(b"\r")
    print("CRLF:", crlf, "LF total:", lf, "bare LF:", lf - crlf, "bare CR:", cr - crlf)
    print("ends with newline:", data.endswith(b"\n"), "last bytes:", data[-20:])
    print("NUL bytes:", data.count(b"\x00"))
    print("TAB bytes:", data.count(b"\t"))
    ctrl = Counter(b for b in data if b < 0x20 and b not in (9, 10, 13))
    print("other C0 control bytes:", dict(ctrl))
    print("XML decl:", data[: data.find(b"?>") + 2] if data.startswith(b"<?xml") else None)
    for kw in (b"<!DOCTYPE", b"<!ENTITY", b"<!ELEMENT", b"<!ATTLIST", b"<!NOTATION",
               b"SYSTEM", b"PUBLIC", b"<![CDATA[", b"<?", b"<!--", b"xi:include", b"xinclude"):
        print("count", kw.decode(), ":", data.count(kw))
    # entity and character references
    ents = Counter(m.group(0) for m in re.finditer(rb"&[^;\s<>&]{0,40};?", data))
    print("entity/char references (distinct, top 50):")
    for k, v in ents.most_common(50):
        print("   ", k, v)
    # non-ASCII characters
    text = data.decode("utf-8", errors="replace")
    na = Counter(ch for ch in text if ord(ch) > 127)
    print("distinct non-ASCII chars:", len(na))
    for ch, n in sorted(na.items(), key=lambda kv: -kv[1])[:60]:
        print("    U+%04X %r %d" % (ord(ch), ch, n))
    # element names
    tags = Counter(m.group(1) for m in re.finditer(rb"<([A-Za-z_][\w:.-]*)", data))
    print("start/empty tag names:")
    for k, v in tags.most_common():
        print("   ", k.decode(), v)
    ends = Counter(m.group(1) for m in re.finditer(rb"</([A-Za-z_][\w:.-]*)", data))
    print("end tag names:")
    for k, v in ends.most_common():
        print("   ", k.decode(), v)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
