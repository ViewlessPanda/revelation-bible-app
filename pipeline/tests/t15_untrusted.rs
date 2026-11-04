//! T15: untrusted input (DESIGN.md 9; findings cases 11, 12, 14–17).
//! XML-layer half: pre-scan, event loop, caps, quick-xml configuration.

mod common;

use common::fixtures;
use revelation_pipeline::profile::KJV_XML;
use revelation_pipeline::xmlscan::{self, Kind};

fn scan(bytes: &[u8]) -> Result<xmlscan::Stats, revelation_pipeline::error::Error> {
    xmlscan::scan(bytes, &KJV_XML, |_, _| Ok(()))
}

#[test]
fn quick_xml_configuration_field_by_field() {
    let r = xmlscan::new_reader("");
    let c = r.config();
    assert!(!c.trim_text_start);
    assert!(!c.trim_text_end);
    assert!(!c.expand_empty_elements);
    assert!(c.check_end_names);
    assert!(!c.allow_unmatched_ends);
    assert!(!c.allow_dangling_amp);
    assert!(c.check_comments);
    assert!(!c.trim_markup_names_in_closing_tags);
}

#[test]
fn every_xml_layer_fixture_fails_with_its_check() {
    for (name, bytes, want) in fixtures::xml_failures() {
        let e = scan(&bytes).expect_err(name);
        if want == "xml_or_attr" {
            assert!(e.check == "xml" || e.check == "attr", "{name}: {e}");
        } else {
            assert_eq!(e.check, want, "{name}: {e}");
        }
        assert_eq!(e.file.as_deref(), Some("kjv.osis.xml"), "{name}");
        assert!(e.offset.is_some(), "{name}: no offset");
    }
}

#[test]
fn four_million_and_one_tokens_fail() {
    let e = scan(&fixtures::too_many_tokens()).unwrap_err();
    assert_eq!(e.check, "cap", "{e}");
}

#[test]
fn offsets_are_file_offsets_and_text_is_raw() {
    let bytes = fixtures::one("alpha  <w lemma=\"strong:H01\">be\u{2019}ta</w> ");
    let mut seen = Vec::new();
    scan_collect(&bytes, &mut seen);
    let text: Vec<&(Kind, String, u64)> = seen.iter().filter(|t| t.0 == Kind::Text).collect();
    // Raw text, untrimmed, spaces kept.
    assert!(text.iter().any(|t| t.1 == "alpha  "));
    assert!(text.iter().any(|t| t.1 == " "));
    for (kind, s, at) in &seen {
        if *kind == Kind::Start {
            assert_eq!(bytes[*at as usize], b'<');
            assert_eq!(
                &bytes[*at as usize + 1..*at as usize + 1 + s.len()],
                s.as_bytes()
            );
        }
    }
}

fn scan_collect(bytes: &[u8], out: &mut Vec<(Kind, String, u64)>) {
    xmlscan::scan(bytes, &KJV_XML, |t, _| {
        let s = if t.kind == Kind::Text { t.text } else { t.name };
        out.push((t.kind, s.to_string(), t.start));
        Ok(())
    })
    .unwrap();
}

#[test]
fn real_kjv_scans_with_the_surveyed_shape() {
    let Some(raw) = common::sources_dir() else {
        return;
    };
    let v = revelation_pipeline::sources::verify(
        &revelation_pipeline::checksums_dir(),
        &raw,
        "BIB-KJV-CW31",
        revelation_pipeline::sources::CAP_KJV,
    )
    .unwrap();
    let mut gen11 = None;
    let stats = xmlscan::scan(&v.bytes, &KJV_XML, |t, _| {
        if t.kind == Kind::Empty && t.name == "verse" && t.attr("sID") == Some("Gen.1.1") {
            gen11 = Some(t.start);
        }
        Ok(())
    })
    .unwrap();
    // Survey excerpt K1: Gen 1:1's sID tag is at byte 4541.
    assert_eq!(gen11, Some(4541));
    assert_eq!(stats.max_depth, 8);
    println!("tokens (incl. Eof): {}", stats.tokens);
}
