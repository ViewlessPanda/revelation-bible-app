//! T14 and the lexicon half of T15 on synthetic fixtures (placeholder
//! words, no lexicon text): byte-exact storage, fail closed with no output
//! file, each failure asserting which check fires. The MGNT fixtures start
//! with the real file's first 1,090 bytes (declaration, LF, the pinned DTD
//! subset; CC0), copied byte for byte to `tests/fixtures/lexicon/`.

mod common;

use std::fs;
use std::path::PathBuf;

use revelation_pipeline::db::LEX_DEST;
use revelation_pipeline::lexicon::{self, LexKind, NodeKind};
use revelation_pipeline::open::open_verified;
use revelation_pipeline::run;

const OSHB_DECL: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";
const ROOT: &str = "<lexicon xsi:schemaLocation=\"http://openscriptures.github.com/morphhb/namespace StrongSchema.xsd\" xmlns=\"http://openscriptures.github.com/morphhb/namespace\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">";

/// An OSHB document around `entries` (CRLF + TAB layout as in the file).
fn oshb_doc(entries: &str) -> Vec<u8> {
    format!("{OSHB_DECL}\r\n{ROOT}\r\n\t{entries}\r\n</lexicon>").into_bytes()
}

const H1: &str = "<entry id=\"H1\">\r\n\t\t<w pos=\"n-m\" pron=\"alef\" xlit=\"\u{02BC}alef\" xml:lang=\"heb\">\u{05D0}\u{05B8}</w>\r\n\t\t<source>a word;</source>\r\n\t\t<meaning><def>alpha</def>, beta</meaning>\r\n\t\t<usage>gamma.</usage>\r\n\t</entry>";
const H2: &str = "<entry id=\"H2\">\r\n\t\t<w pos=\"v\" pron=\"bet\" xlit=\"bet\" xml:lang=\"arc\">\u{05D1}</w>\r\n\t\t<source>from <w src=\"H1\">1</w></source>\r\n\t\t<source/>\r\n\t\t<note>delta</note>\r\n\t</entry>";

fn oshb_good() -> Vec<u8> {
    oshb_doc(&format!("{H1}\r\n\t{H2}"))
}

fn mgnt_prefix() -> Vec<u8> {
    let p =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lexicon/mgnt-prefix.bin");
    let b = fs::read(p).unwrap();
    assert_eq!(b.len(), 1090);
    b
}

fn mgnt_doc(entries: &str) -> Vec<u8> {
    let mut b = mgnt_prefix();
    b.extend_from_slice(
        format!("\n\n<strongsdictionary><prologue>\n\tAlpha prologue.\n</prologue><entries>{entries}</entries></strongsdictionary>\n")
            .as_bytes(),
    );
    b
}

const G1: &str = "<entry strongs=\"00001\">\n\n <strongs>1</strongs>   <greek BETA=\"*A\" unicode=\"\u{0391}\" translit=\"A\"/>   <pronunciation strongs=\"al'-fah\"/>\n\n <strongs_derivation>of <strongsref language=\"HEBREW\" strongs=\"01\"/>;</strongs_derivation><strongs_def> the first\n (alpha): </strongs_def><kjv_def>--Alpha.</kjv_def> Often used <strongsref language=\"GREEK\" strongs=\"2\"/>.\n\n\n</entry>";
const G2: &str = "<entry strongs=\"00002\">\n <strongs>2</strongs>  <greek BETA=\"B\" unicode=\"\u{03B2}\" translit=\"b\"/> <kjv_def>--beta.</kjv_def>\n</entry>";

fn mgnt_good() -> Vec<u8> {
    mgnt_doc(&format!("{G1}{G2}"))
}

fn fresh(label: &str) -> PathBuf {
    common::temp_dir(label)
}

#[test]
fn fixtures_build_and_store_every_byte() {
    let out = fresh("lexfix-ok");
    let hash =
        run::build_lex_bytes(&oshb_good(), &mgnt_good(), &out).unwrap_or_else(|e| panic!("{e}"));
    let (c, h) = open_verified(&out.join(LEX_DEST), Some(&hash)).unwrap();
    assert_eq!(h, hash);
    // Whitespace nodes keep CR LF TAB; text is raw; empty <source/> stored as
    // an element with no children; attribute order kept.
    let ws: String = c
        .query_row(
            "SELECT text FROM lex_node WHERE key = 'H1' AND n = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(ws, "\r\n\t\t");
    let names: Vec<String> = c
        .prepare("SELECT name FROM lex_attr WHERE key = 'H1' AND n = 1 ORDER BY i")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(names, ["pos", "pron", "xlit", "xml:lang"]);
    let refs: Vec<(String, String)> = c
        .prepare("SELECT key, ref_key FROM lex_attr WHERE ref_key IS NOT NULL ORDER BY key, n")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        refs,
        [
            ("G1".into(), "H1".into()),
            ("G1".into(), "G2".into()),
            ("H2".into(), "H1".into())
        ]
    );
    let g1: String = c
        .query_row(
            "SELECT group_concat(text, '') FROM (SELECT text FROM lex_node WHERE key = 'G1' AND text IS NOT NULL ORDER BY n)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    // "1", then the 3 + 3 spaces around the two empty elements.
    assert_eq!(
        g1,
        "\n\n 1      \n\n of ; the first\n (alpha): --Alpha. Often used .\n\n\n"
    );
    drop(c);
    // Determinism on fixtures.
    let out2 = fresh("lexfix-ok2");
    assert_eq!(
        run::build_lex_bytes(&oshb_good(), &mgnt_good(), &out2).unwrap(),
        hash
    );
    let m = lexicon::parse(&mgnt_good(), &lexicon::fixture(lexicon::mgnt())).unwrap();
    assert!(!m.entries[0].placeholder);
    assert_eq!(m.kind, LexKind::Mgnt);
    assert!(
        matches!(&m.entries[0].nodes[1].kind, NodeKind::Element { name, .. } if name == "strongs")
    );
    for d in [out, out2] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn placeholder_is_recognised() {
    let ph = "<entry strongs=\"00003\">\n <strongs>3</strongs>  Not Used\n</entry>";
    let m = lexicon::parse(
        &mgnt_doc(&format!("{G2}{ph}")),
        &lexicon::fixture(lexicon::mgnt()),
    )
    .unwrap();
    assert!(m.entries[1].placeholder);
    // A reference to a placeholder fails the cross-reference check.
    let g = mgnt_doc(&format!(
        "{}{ph}",
        G2.replace(
            "<kjv_def>--beta.</kjv_def>",
            "<kjv_def><strongsref language=\"GREEK\" strongs=\"3\"/></kjv_def>"
        )
    ));
    let out = fresh("lexfix-ph");
    let f = run::build_lex_bytes(&oshb_good(), &g, &out).unwrap_err();
    assert!(f.has("lex-ref"), "{f}");
    assert!(common::list(&out).is_empty());
    let _ = fs::remove_dir_all(out);
}

/// Every failing fixture: (name, OSHB bytes, MGNT bytes, check).
fn failures() -> Vec<(&'static str, Vec<u8>, Vec<u8>, &'static str)> {
    let o = |s: &str| oshb_doc(s);
    let g = |s: &str| mgnt_doc(s);
    let ok_o = oshb_good;
    let ok_g = mgnt_good;
    let h1 = |from: &str, to: &str| o(&format!("{}\r\n\t{H2}", H1.replace(from, to)));
    let g1 = |from: &str, to: &str| g(&format!("{}{G2}", G1.replace(from, to)));
    let mut v: Vec<(&'static str, Vec<u8>, Vec<u8>, &'static str)> = vec![
        (
            "CR in a non-whitespace node",
            h1("<def>alpha</def>", "<def>al\r\npha</def>"),
            ok_g(),
            "lex-char",
        ),
        (
            "lone CR in whitespace",
            h1("\r\n\t\t<source>", "\r\t\t<source>"),
            ok_g(),
            "lex-char",
        ),
        (
            "bare LF",
            h1("\r\n\t\t<source>", "\n\t\t<source>"),
            ok_g(),
            "lex-char",
        ),
        (
            "TAB in text",
            h1("<def>alpha</def>", "<def>al\tpha</def>"),
            ok_g(),
            "lex-char",
        ),
        (
            "H01 and H1 in one file",
            o(&format!(
                "{H1}\r\n\t{}",
                H2.replace("id=\"H2\"", "id=\"H01\"")
            )),
            ok_g(),
            "lex-key",
        ),
        (
            "duplicate entry",
            o(&format!("{H1}\r\n\t{H1}")),
            ok_g(),
            "lex-key",
        ),
        (
            "descending entries",
            o(&format!("{H2}\r\n\t{H1}")),
            ok_g(),
            "lex-key",
        ),
        (
            "entry id without H",
            h1("id=\"H1\"", "id=\"1\""),
            ok_g(),
            "lex-key",
        ),
        (
            "w@src padded",
            o(&format!(
                "{H1}\r\n\t{}",
                H2.replace("src=\"H1\"", "src=\"H01\"")
            )),
            ok_g(),
            "lex-ref",
        ),
        (
            "w@src dangling",
            o(&format!(
                "{H1}\r\n\t{}",
                H2.replace("src=\"H1\"", "src=\"H9\"")
            )),
            ok_g(),
            "lex-ref",
        ),
        (
            "U+202E in lexicon text",
            h1("<def>alpha</def>", "<def>al\u{202E}pha</def>"),
            ok_g(),
            "lex-char",
        ),
        (
            "U+200B in an attribute",
            h1("pron=\"alef\"", "pron=\"al\u{200B}ef\""),
            ok_g(),
            "lex-char",
        ),
        (
            "NBSP in text",
            h1("<usage>gamma.</usage>", "<usage>gamma\u{00A0}.</usage>"),
            ok_g(),
            "lex-char",
        ),
        (
            "text directly in an OSHB entry",
            h1("\r\n\t\t<source>", "\r\n\t\tstray<source>"),
            ok_g(),
            "lex-structure",
        ),
        (
            "unknown element",
            h1("<def>alpha</def>", "<b>alpha</b>"),
            ok_g(),
            "form",
        ),
        (
            "element in the wrong parent",
            h1("<def>alpha</def>", "<usage>alpha</usage>"),
            ok_g(),
            "lex-structure",
        ),
        (
            "nested entry",
            h1(
                "<usage>gamma.</usage>",
                "<usage>gamma.</usage><entry id=\"H5\"></entry>",
            ),
            ok_g(),
            "lex-structure",
        ),
        (
            "unknown attribute",
            h1("<source>", "<source x=\"1\">"),
            ok_g(),
            "lex-attr",
        ),
        (
            "namespace-prefixed name",
            h1("xml:lang=\"heb\"", "x:lang=\"heb\""),
            ok_g(),
            "lex-attr",
        ),
        (
            "text outside entries",
            o(&format!("{H1}\r\n\tstray\r\n\t{H2}")),
            ok_g(),
            "lex-structure",
        ),
        (
            "single-quoted attribute",
            h1("pos=\"n-m\"", "pos='n-m'"),
            ok_g(),
            "attr",
        ),
        (
            "duplicate attribute",
            h1("pos=\"n-m\"", "pos=\"n-m\" pos=\"v\""),
            ok_g(),
            "attr",
        ),
        (
            "DOCTYPE in OSHB",
            format!("{OSHB_DECL}\r\n<!DOCTYPE lexicon>{ROOT}{H1}</lexicon>").into_bytes(),
            ok_g(),
            "dtd",
        ),
        (
            "entity in OSHB",
            h1("<def>alpha</def>", "<def>al&amp;pha</def>"),
            ok_g(),
            "entity",
        ),
        (
            "BOM before the declaration",
            {
                let mut b = b"\xEF\xBB\xBF".to_vec();
                b.extend(oshb_good());
                b
            },
            ok_g(),
            "decl",
        ),
        // MGNT
        (
            "strongsref language=\"Greek\"",
            ok_o(),
            g1("language=\"GREEK\"", "language=\"Greek\""),
            "lex-ref",
        ),
        (
            "strongs=\"0\"",
            ok_o(),
            g1("strongs=\"2\"/>", "strongs=\"0\"/>"),
            "lex-ref",
        ),
        (
            "Greek ref padded",
            ok_o(),
            g1("strongs=\"2\"/>", "strongs=\"02\"/>"),
            "lex-ref",
        ),
        (
            "Hebrew ref unpadded",
            ok_o(),
            g1("strongs=\"01\"/>", "strongs=\"1\"/>"),
            "lex-ref",
        ),
        (
            "Hebrew ref double-padded",
            ok_o(),
            g1("strongs=\"01\"/>", "strongs=\"001\"/>"),
            "lex-ref",
        ),
        (
            "strongsref without strongs",
            ok_o(),
            g1(" strongs=\"2\"/>", "/>"),
            "lex-attr",
        ),
        (
            "entry strongs not five digits",
            ok_o(),
            g1("strongs=\"00001\"", "strongs=\"0001\""),
            "lex-key",
        ),
        (
            "<strongs> differs from the number",
            ok_o(),
            g1("<strongs>1</strongs>", "<strongs>01</strongs>"),
            "lex-structure",
        ),
        (
            "two <strongs>",
            ok_o(),
            g1(
                "<strongs>1</strongs>",
                "<strongs>1</strongs><strongs>1</strongs>",
            ),
            "lex-structure",
        ),
        (
            "empty greek as a container",
            ok_o(),
            g1("translit=\"A\"/>", "translit=\"A\"></greek>"),
            "form",
        ),
        (
            "TAB in an MGNT entry",
            ok_o(),
            g1(" Often used", "\tOften used"),
            "lex-char",
        ),
        (
            "CR in MGNT",
            ok_o(),
            g1(" Often used", "\r\nOften used"),
            "bytes",
        ),
        (
            "17 attributes",
            ok_o(),
            g1(
                "<greek BETA=\"*A\"",
                &format!(
                    "<greek BETA=\"*A\"{}",
                    (0..16).map(|i| format!(" a{i}=\"x\"")).collect::<String>()
                ),
            ),
            "cap",
        ),
        (
            "prologue twice",
            ok_o(),
            {
                let mut b = mgnt_prefix();
                b.extend_from_slice(b"\n\n<strongsdictionary><prologue>a</prologue><prologue>b</prologue><entries></entries></strongsdictionary>\n");
                b
            },
            "lex-structure",
        ),
    ];
    // DTD subset variants (findings case 19).
    let mut changed = mgnt_good();
    changed[200] ^= 0x20;
    v.push(("DTD subset changed", ok_o(), changed, "dtd"));
    let mut moved = mgnt_good();
    moved.insert(55, b'\n');
    v.push(("DTD subset moved", ok_o(), moved, "dtd"));
    let mut no_lf = mgnt_good();
    no_lf[55] = b' ';
    v.push(("no LF before the subset", ok_o(), no_lf, "dtd"));
    v.push((
        "second DOCTYPE after byte 1090",
        ok_o(),
        g(&format!("<!DOCTYPE x>{G1}")),
        "dtd",
    ));
    v.push((
        "comment after byte 1090",
        ok_o(),
        g(&format!("<!-- x -->{G1}")),
        "dtd",
    ));
    v.push((
        "ENTITY after byte 1090",
        ok_o(),
        g(&format!("<!ENTITY a \"b\">{G1}")),
        "dtd",
    ));
    v.push((
        "CDATA after byte 1090",
        ok_o(),
        g(&format!("<![CDATA[x]]>{G1}")),
        "dtd",
    ));
    v.push((
        "PI after byte 1090",
        ok_o(),
        g(&format!("<?x y?>{G1}")),
        "decl",
    ));
    let mut truncated = mgnt_prefix();
    truncated.truncate(600);
    v.push(("file ends inside the subset", ok_o(), truncated, "dtd"));
    v
}

#[test]
fn lexicon_fixtures_fail_closed_with_the_named_check_and_no_output() {
    for (i, (name, o, g, want)) in failures().into_iter().enumerate() {
        let out = fresh(&format!("lexfix-bad{i}"));
        let f = run::build_lex_bytes(&o, &g, &out).expect_err(name);
        assert!(f.has(want), "{name}: expected {want}, got {f}");
        assert!(common::list(&out).is_empty(), "{name}: output written");
        let _ = fs::remove_dir_all(out);
    }
}

#[test]
fn root_attributes_are_pinned() {
    let s = String::from_utf8(oshb_good())
        .unwrap()
        .replace("StrongSchema.xsd", "Other.xsd");
    let f = lexicon::parse(s.as_bytes(), &lexicon::fixture(lexicon::oshb())).unwrap_err();
    assert!(f.has("lex-attr"), "{f}");
}
