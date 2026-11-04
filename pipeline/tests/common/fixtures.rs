//! Byte-exact synthetic fixtures (placeholder words, no Scripture) for T6,
//! T9, T10, T12, T13 and T15. Each is built from Rust literals, so the bytes
//! are exactly what this file says; the M1-F3 set also exists as files in
//! `pipeline/tests/fixtures/` and in `docs/pipeline/survey-scripts/fixtures/`
//! (the "both" set, run against the Python walker too).
#![allow(dead_code)]

pub const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

/// A one-book, one-chapter document around `chapter_body`.
pub fn doc(chapter_body: &str) -> Vec<u8> {
    format!(
        "{DECL}<osis><osisText><div type=\"book\" osisID=\"Test\">\n<chapter osisID=\"Test.1\">\n{chapter_body}\n</chapter>\n</div></osisText></osis>\n"
    )
    .into_bytes()
}

/// One verse `Test.1.<n>` with `body` as its content.
pub fn v(n: u32, body: &str) -> String {
    format!("<verse osisID=\"Test.1.{n}\" sID=\"Test.1.{n}\"/>{body}<verse eID=\"Test.1.{n}\"/>")
}

/// A document with one verse.
pub fn one(body: &str) -> Vec<u8> {
    doc(&v(1, body))
}

pub const Q: &str = "<q who=\"Jesus\" marker=\"\">";

/// A `w` with a valid KJV-profile Hebrew lemma.
pub fn w(text: &str) -> String {
    format!("<w lemma=\"strong:H01\">{text}</w>")
}

/// Fixtures that must fail at the XML layer (pre-scan or event loop), with
/// the check that must fire.
pub fn xml_failures() -> Vec<(&'static str, Vec<u8>, &'static str)> {
    let mut out: Vec<(&'static str, Vec<u8>, &'static str)> = Vec::new();
    let mut with = |name, bytes: Vec<u8>, check| out.push((name, bytes, check));

    let pre = |s: &str| format!("{DECL}{s}").into_bytes();
    with("doctype", pre("<!DOCTYPE osis><osis/>\n"), "dtd");
    with(
        "billion-laughs",
        pre("<!DOCTYPE osis [<!ENTITY a \"aaaa\"><!ENTITY b \"&a;&a;&a;&a;\">]><osis>&b;</osis>\n"),
        "entity",
    );
    with(
        "external-entity",
        pre("<!DOCTYPE osis [<!ENTITY x SYSTEM \"file:///C:/Windows/win.ini\">]><osis>x</osis>\n"),
        "dtd",
    );
    with(
        "parameter-entity",
        pre("<!DOCTYPE osis [<!ENTITY % p \"x\"> %p;]><osis/>\n"),
        "dtd",
    );
    with("amp-entity", one("alpha &amp; beta"), "entity");
    with("char-ref-dec", one("alpha &#65; beta"), "entity");
    with("char-ref-rlo", one("alpha &#x202E; beta"), "entity");
    with("char-ref-cr", one("alpha &#13; beta"), "entity");
    with("lt-entity", one("alpha &lt; beta"), "entity");
    with("unknown-entity", one("alpha &unknown; beta"), "entity");
    with("lone-amp", one("alpha & beta"), "entity");
    with("cdata", one("alpha <![CDATA[beta]]>"), "dtd");
    with("comment", one("alpha <!-- beta -->"), "dtd");
    with("pi", one("alpha <?xml-stylesheet href=\"x\"?>"), "decl");
    with("second-decl", one("<?xml version=\"1.0\"?>alpha"), "decl");
    let mut bom = b"\xEF\xBB\xBF".to_vec();
    bom.extend_from_slice(&one("alpha"));
    with("bom", bom, "decl");
    let mut bad = one("alpha");
    let at = bad.len() - 20;
    let mut overlong = bad.clone();
    overlong.splice(at..at, [0xC0, 0xAF]);
    with("utf8-overlong", overlong, "utf8");
    let mut surrogate = bad.clone();
    surrogate.splice(at..at, [0xED, 0xA0, 0x80]);
    with("utf8-surrogate", surrogate, "utf8");
    bad.extend_from_slice(&[0xE2, 0x80]);
    with("utf8-truncated-at-eof", bad, "utf8");
    with("nul-byte", one("alpha\u{0}beta"), "bytes");
    with("lone-cr", one("alpha\rbeta"), "bytes");
    with("crlf", one("alpha\r\nbeta"), "bytes");
    with("u-fffe", one("alpha\u{fffe}beta"), "bytes");

    // Depth 17 and 100,000 (the own stack stops at 17).
    let deep = |n: usize| {
        let mut s = String::from(DECL);
        for _ in 0..n {
            s.push_str("<div>");
        }
        for _ in 0..n {
            s.push_str("</div>");
        }
        s.into_bytes()
    };
    with("depth-17", deep(17), "cap");
    with("depth-100000", deep(100_000), "cap");
    // 17 and 1,000,000 attributes on one start tag.
    let attrs = |n: usize| {
        let mut s = format!("{DECL}<w");
        for i in 0..n {
            s.push_str(&format!(" a{i}=\"x\""));
        }
        s.push_str("/>\n");
        s.into_bytes()
    };
    with("attrs-17", attrs(17), "cap");
    with("attrs-1000000", attrs(1_000_000), "cap");
    // `w` may be Empty or a container, so these reach the attribute checks.
    with("duplicate-attribute", pre("<w a=\"1\" a=\"2\"/>\n"), "attr");
    let long_name = "n".repeat(65);
    with("name-65-bytes", pre(&format!("<{long_name}/>\n")), "cap");
    let long_attr = format!("a{}", "b".repeat(64));
    with(
        "attr-name-65-bytes",
        pre(&format!("<w {long_attr}=\"x\"/>\n")),
        "cap",
    );
    let long_value = "v".repeat(4097);
    with(
        "attr-value-4097",
        pre(&format!("<w a=\"{long_value}\"/>\n")),
        "cap",
    );
    let long_text = "t".repeat(64 * 1024 + 1);
    with(
        "text-65537",
        pre(&format!("<osis>{long_text}</osis>\n")),
        "cap",
    );
    with("single-quoted-attribute", pre("<w a='x'/>\n"), "attr");
    with("lt-in-attribute", pre("<w a=\"x<y\"/>\n"), "xml_or_attr");
    with("cdata-end-in-text", one("alpha ]]> beta"), "xml");
    with("second-root", pre("<w/><w/>\n"), "xml");
    with("text-after-root", pre("<w/>stray\n"), "xml");
    with("text-before-root", pre("stray<w/>\n"), "xml");
    with("unknown-element", pre("<unknown/>\n"), "form");
    with("unclosed-at-eof", pre("<osis><osisText>\n"), "xml");
    with(
        "mismatched-end",
        pre("<osis><div></osisText></osis>\n"),
        "xml",
    );
    with(
        "namespaced-q",
        one("<x:q who=\"Jesus\" marker=\"\">alpha</x:q>"),
        "form",
    );
    with(
        "empty-q",
        one("alpha <q who=\"Jesus\" marker=\"\"/>beta"),
        "form",
    );
    with(
        "empty-transChange",
        one("alpha <transChange type=\"added\"/>beta"),
        "form",
    );
    with("empty-divineName", one("alpha <divineName/>beta"), "form");
    with(
        "container-verse",
        doc("<verse osisID=\"Test.1.1\">alpha beta</verse>"),
        "form",
    );
    with(
        "milestone-q-empty",
        doc(&format!(
            "{}\n{}",
            v(1, "alpha <q sID=\"q1\" who=\"Jesus\" marker=\"\"/>beta"),
            v(2, "gamma<q eID=\"q1\"/> delta")
        )),
        "form",
    );
    out
}

/// 4,000,001 tokens (needs about 36 MB; built only by the test that uses it).
pub fn too_many_tokens() -> Vec<u8> {
    let mut s = String::with_capacity(40_000_000);
    s.push_str(DECL);
    s.push_str("<osis>");
    // <osis> + 3,999,999 empty elements + </osis> + Eof = 4,000,002 events.
    for _ in 0..3_999_999 {
        s.push_str("<verse/>");
    }
    s.push_str("</osis>");
    s.into_bytes()
}
