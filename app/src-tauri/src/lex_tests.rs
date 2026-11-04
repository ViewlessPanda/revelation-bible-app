//! Tests against the real lexicons, `pipeline/out/content.sqlite`, opened only
//! through `open_verified` with the SHA-256 compiled in from
//! `pipeline/tests/expected/output-sha256.txt`.
//!
//! - every entry the app returns equals its stored rows (nodes in `n` order,
//!   attributes in `n, i` order, text exactly);
//! - every Strong's key in the KJV and every cross-reference resolves;
//! - a missing or damaged lexicon never stops the Bible from opening, and
//!   lookups then answer "not available";
//! - the render golden `tests/golden/lexicon-entries.json` (machine-generated
//!   entries for the vitest render test) is up to date.
//!
//! Regenerate the golden (one command, in app/src-tauri):
//! `cargo test -- --ignored regenerate_lexicon_golden`

use crate::commands::{self, AppState, KJV_FILE, LEX_FILE, StrongsResult};
use crate::content::{self, LexAttr, LexNode, Lexicon, MAX_CONTENT_BYTES};
use crate::expected;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

fn content_dir() -> PathBuf {
    commands::content_dir().unwrap()
}

fn open_lex() -> Lexicon {
    Lexicon::open(&content_dir().join(LEX_FILE), expected::CONTENT_SQLITE).unwrap_or_else(|e| {
        panic!(
            "{e}. pipeline/out/content.sqlite must match pipeline/tests/expected/output-sha256.txt \
             (rebuild it with the pipeline, or rebuild the app after that file changed)"
        )
    })
}

/// A second, independent verified copy for raw table reads.
fn raw(file: &str, sha: &str) -> Connection {
    content::open_content(&content_dir().join(file), sha, MAX_CONTENT_BYTES).unwrap()
}

fn raw_nodes(c: &Connection, key: &str) -> Vec<LexNode> {
    let mut nodes: Vec<LexNode> = c
        .prepare("SELECT n, parent, name, text FROM lex_node WHERE key = ?1 ORDER BY n")
        .unwrap()
        .query_map([key], |r| {
            Ok(LexNode {
                n: r.get(0)?,
                parent: r.get(1)?,
                name: r.get(2)?,
                text: r.get(3)?,
                attrs: Vec::new(),
            })
        })
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    let mut st = c
        .prepare("SELECT n, name, value, ref_key FROM lex_attr WHERE key = ?1 ORDER BY n, i")
        .unwrap();
    let rows = st
        .query_map([key], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                LexAttr {
                    name: r.get(1)?,
                    value: r.get(2)?,
                    ref_key: r.get(3)?,
                },
            ))
        })
        .unwrap();
    for row in rows {
        let (n, a) = row.unwrap();
        nodes.iter_mut().find(|x| x.n == n).unwrap().attrs.push(a);
    }
    nodes
}

#[test]
fn every_entry_equals_its_stored_rows() {
    let lex = open_lex();
    let c = raw(LEX_FILE, expected::CONTENT_SQLITE);
    let keys: Vec<(String, String)> = c
        .prepare("SELECT key, lexicon FROM lex_entry ORDER BY lexicon, ord")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    let (mut heb, mut grc, mut not_by_name, mut text_bytes) = (0, 0, 0, 0usize);
    for (key, lexicon) in &keys {
        let e = lex.entry(key).unwrap().expect(key);
        assert_eq!(&e.key, key);
        assert_eq!(&e.lexicon, lexicon);
        assert_eq!(e.nodes, raw_nodes(&c, key), "{key}");
        for n in &e.nodes {
            text_bytes += n.text.as_deref().map_or(0, str::len);
            let mut names: Vec<&str> = n.attrs.iter().map(|a| a.name.as_str()).collect();
            let stored = names.clone();
            names.sort();
            if names != stored {
                not_by_name += 1;
            }
        }
        match lexicon.as_str() {
            "LEX-HEB-OSHB" => heb += 1,
            "LEX-GRC-MGNT19" => grc += 1,
            other => panic!("unknown lexicon {other}"),
        }
    }
    assert_eq!((heb, grc), (8674, 5624));
    // Attribute order is the source's (lex_attr.i), not alphabetical: MGNT
    // `greek` is BETA, unicode, translit.
    assert!(not_by_name > 5000, "{not_by_name}");
    assert!(text_bytes > 1_000_000, "{text_bytes}");
}

#[test]
fn every_kjv_key_and_every_cross_reference_resolves() {
    let lex = open_lex();
    let kjv = raw(KJV_FILE, expected::KJV_SQLITE);
    let keys: Vec<String> = kjv
        .prepare("SELECT DISTINCT key FROM word_key ORDER BY key")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert_eq!(keys.len(), 14075);
    for k in &keys {
        assert!(lex.entry(k).unwrap().is_some(), "{k}");
    }
    let c = raw(LEX_FILE, expected::CONTENT_SQLITE);
    let refs: Vec<String> = c
        .prepare("SELECT DISTINCT ref_key FROM lex_attr WHERE ref_key IS NOT NULL")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    assert!(refs.len() > 1000, "{}", refs.len());
    for k in &refs {
        assert!(lex.entry(k).unwrap().is_some(), "{k}");
    }
}

fn state_for(content: Result<commands::Content, String>) -> AppState {
    AppState {
        content: Mutex::new(content),
        user: Mutex::new(Err("not used".into())),
    }
}

/// Reading Scripture never depends on the lexicon: missing, damaged or
/// stale, the Bible opens and lookups say "not available"; with the right
/// file they work.
#[test]
fn a_missing_or_damaged_lexicon_keeps_the_bible_working() {
    let dir = content_dir();
    let tmp = crate::fixture::temp_dir("lex-missing");
    std::fs::copy(dir.join(KJV_FILE), tmp.join(KJV_FILE)).unwrap();

    let mut stale = expected::CONTENT_SQLITE.to_string();
    let last = if stale.ends_with('0') { "1" } else { "0" };
    stale.replace_range(63..64, last);

    let check_unavailable = |d: &std::path::Path, lex_sha: &str, why: &str| {
        let c = commands::load_content_from(d, expected::KJV_SQLITE, lex_sha)
            .unwrap_or_else(|e| panic!("{why}: the Bible must still open: {e}"));
        let err = c.lexicon.as_ref().err().cloned().expect(why);
        assert!(err.contains("content.sqlite"), "{why}: {err}");
        let state = state_for(Ok(c));
        let info = commands::info(&state);
        assert!(info.content_error.is_none(), "{why}");
        assert!(!info.lexicon_available, "{why}");
        assert_eq!(info.lexicon_error.as_deref(), Some(err.as_str()), "{why}");
        assert_eq!(info.translations.len(), 1, "{why}");
        let ch = commands::with_bible(&state, "KJV", |b| b.chapter(43, 3)).unwrap();
        assert_eq!(ch.verses.len(), 36, "{why}");
        match commands::strongs(&state, "H430").unwrap() {
            StrongsResult::NotAvailable { reason } => assert_eq!(reason, err, "{why}"),
            other => panic!("{why}: {other:?}"),
        }
    };
    // 1. no content.sqlite at all
    check_unavailable(&tmp, expected::CONTENT_SQLITE, "missing");
    // 2. one flipped bit
    let mut bytes = std::fs::read(dir.join(LEX_FILE)).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    std::fs::write(tmp.join(LEX_FILE), &bytes).unwrap();
    check_unavailable(&tmp, expected::CONTENT_SQLITE, "tampered");
    // 3. the real file, but the app expects another hash (stale build)
    check_unavailable(&dir, &stale, "stale hash");
    // 4. a directory in its place
    std::fs::remove_file(tmp.join(LEX_FILE)).unwrap();
    std::fs::create_dir(tmp.join(LEX_FILE)).unwrap();
    check_unavailable(&tmp, expected::CONTENT_SQLITE, "directory");

    // The real pair works.
    let state = state_for(commands::load_content_from(
        &dir,
        expected::KJV_SQLITE,
        expected::CONTENT_SQLITE,
    ));
    let info = commands::info(&state);
    assert!(info.lexicon_available && info.lexicon_error.is_none());
    match commands::strongs(&state, "H430").unwrap() {
        StrongsResult::Ok { entry } => assert_eq!(entry.lexicon, "LEX-HEB-OSHB"),
        other => panic!("{other:?}"),
    }
    match commands::strongs(&state, "G26").unwrap() {
        StrongsResult::Ok { entry } => assert_eq!(entry.lexicon, "LEX-GRC-MGNT19"),
        other => panic!("{other:?}"),
    }
    // canonical but absent; and malformed keys never reach SQL
    for k in ["H9999", "G99999"] {
        assert!(matches!(
            commands::strongs(&state, k).unwrap(),
            StrongsResult::NotFound { .. }
        ));
    }
    for k in ["", "H0430", "h430", "H430 ", "X1", "H1234567"] {
        assert!(commands::strongs(&state, k).is_err(), "{k:?}");
    }
    // A Bible failure still wins: no text, and lookups are unavailable too.
    let broken = state_for(Err("kjv.sqlite: failed its checksum".into()));
    assert!(matches!(
        commands::strongs(&broken, "H430").unwrap(),
        StrongsResult::NotAvailable { .. }
    ));
}

// ---------------------------------------------------------------------------
// Render golden for the UI test (vitest reads this JSON)
// ---------------------------------------------------------------------------

const LEX_GOLDEN_FILE: &str = "tests/golden/lexicon-entries.json";

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(LEX_GOLDEN_FILE)
}

/// The entries in the golden: four well-known ones, plus the first entry (by
/// lexicon and document order) of each shape the panel must handle. Chosen by
/// query, never typed in.
fn golden_keys(c: &Connection) -> Vec<(String, String)> {
    let first = |why: &str, sql: &str| -> (String, String) {
        let k: String = c
            .query_row(sql, [], |r| r.get(0))
            .unwrap_or_else(|e| panic!("{why}: {e}"));
        (k, why.to_string())
    };
    let mut out: Vec<(String, String)> = [
        ("H430", "well-known Hebrew entry; CRLF + TAB layout nodes"),
        (
            "H3068",
            "well-known Hebrew entry with w@src cross-references",
        ),
        ("G26", "well-known Greek entry"),
        ("G2316", "well-known Greek entry with strongsref"),
        (
            "G3588",
            "with G2316, the two keys of one word in John 3:2 (multi-key test)",
        ),
    ]
    .iter()
    .map(|(k, w)| (k.to_string(), w.to_string()))
    .collect();
    out.push(first(
        "first Hebrew entry with an OSHB note",
        "SELECT e.key FROM lex_entry e WHERE e.lexicon = 'LEX-HEB-OSHB' AND EXISTS \
         (SELECT 1 FROM lex_node n WHERE n.key = e.key AND n.name = 'note') ORDER BY e.ord LIMIT 1",
    ));
    out.push(first(
        "first Greek entry with text directly inside <entry>",
        "SELECT e.key FROM lex_entry e WHERE e.lexicon = 'LEX-GRC-MGNT19' AND EXISTS \
         (SELECT 1 FROM lex_node n WHERE n.key = e.key AND n.parent IS NULL AND n.text IS NOT NULL \
          AND trim(n.text, ' ' || char(10) || char(13) || char(9)) <> '') ORDER BY e.ord LIMIT 1",
    ));
    out.push(first(
        "first Greek entry with a line feed inside a field",
        "SELECT e.key FROM lex_entry e WHERE e.lexicon = 'LEX-GRC-MGNT19' AND EXISTS \
         (SELECT 1 FROM lex_node n WHERE n.key = e.key AND n.parent IS NOT NULL \
          AND instr(n.text, char(10)) > 0) ORDER BY e.ord LIMIT 1",
    ));
    out.push(first(
        "first Greek entry referring to a Hebrew entry",
        "SELECT e.key FROM lex_entry e WHERE e.lexicon = 'LEX-GRC-MGNT19' AND EXISTS \
         (SELECT 1 FROM lex_attr a WHERE a.key = e.key AND a.ref_key GLOB 'H*') ORDER BY e.ord LIMIT 1",
    ));
    out.push(first(
        "first Greek 'Not Used' placeholder",
        "SELECT e.key FROM lex_entry e WHERE e.lexicon = 'LEX-GRC-MGNT19' AND \
         (SELECT count(*) FROM lex_node n WHERE n.key = e.key AND n.name IS NOT NULL) = 1 \
         ORDER BY e.ord LIMIT 1",
    ));
    let mut seen = std::collections::HashSet::new();
    out.retain(|(k, _)| seen.insert(k.clone()));
    out
}

fn lexicon_golden() -> String {
    let lex = open_lex();
    let c = raw(LEX_FILE, expected::CONTENT_SQLITE);
    let mut s = String::from("{\n");
    s.push_str("\"note\": \"GENERATED from pipeline/out/content.sqlite through open_verified (expected SHA-256 compiled in from pipeline/tests/expected/output-sha256.txt); each entry is exactly what get_strongs returns. Do not edit. Regenerate in app/src-tauri: cargo test -- --ignored regenerate_lexicon_golden\",\n");
    s.push_str("\"entries\": [\n");
    let keys = golden_keys(&c);
    for (i, (key, why)) in keys.iter().enumerate() {
        let e = lex.entry(key).unwrap().expect(key);
        assert_eq!(e.nodes, raw_nodes(&c, key));
        s.push_str(&format!(
            "{{\"why\": {}, \"entry\": {}}}{}\n",
            serde_json::to_string(why).unwrap(),
            serde_json::to_string(&e).unwrap(),
            if i + 1 < keys.len() { "," } else { "" }
        ));
    }
    s.push_str("]\n}\n");
    s
}

#[test]
#[ignore = "regenerates tests/golden/lexicon-entries.json; run by hand after the DB changes"]
fn regenerate_lexicon_golden() {
    let p = golden_path();
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, lexicon_golden()).unwrap();
    println!("wrote {}", p.display());
}

#[test]
fn lexicon_golden_is_up_to_date() {
    let committed = std::fs::read_to_string(golden_path())
        .expect("tests/golden/lexicon-entries.json missing: run the regenerate command")
        .replace("\r\n", "\n");
    assert!(
        committed == lexicon_golden(),
        "{LEX_GOLDEN_FILE} differs from the DB: run `cargo test -- --ignored regenerate_lexicon_golden` \
         and review the diff"
    );
    // It really covers the shapes the panel must handle.
    let v: serde_json::Value = serde_json::from_str(&committed).unwrap();
    let entries = v["entries"].as_array().unwrap();
    assert!(entries.len() >= 9, "{}", entries.len());
    let texts: Vec<&str> = entries
        .iter()
        .flat_map(|e| e["entry"]["nodes"].as_array().unwrap())
        .filter_map(|n| n["text"].as_str())
        .collect();
    assert!(texts.iter().any(|t| t.contains("\r\n\t")));
    assert!(
        texts
            .iter()
            .any(|t| t.contains('\n') && !t.trim().is_empty())
    );
    let refs = entries
        .iter()
        .flat_map(|e| e["entry"]["nodes"].as_array().unwrap())
        .flat_map(|n| n["attrs"].as_array().unwrap())
        .filter(|a| a["ref_key"].is_string())
        .count();
    assert!(refs >= 3, "{refs}");
}
