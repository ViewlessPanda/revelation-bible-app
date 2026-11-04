//! Commentary against the real pipeline output (`pipeline/out/mhcc.sqlite`),
//! opened only through `open_verified` with the compiled-in hash; plus the
//! golden `tests/golden/mhcc-render.json` that the vitest render test reads.
//!
//! Regenerate the golden (in app/src-tauri):
//! `cargo test -- --ignored regenerate_commentary_golden`

use crate::commands::{self, AppState, KJV_FILE};
use crate::commentary::{
    CmtGroup, CmtSource, Commentary, CommentaryResult, MHCC_FILE, NEUTRAL_TITLE,
};
use crate::content::{self, MAX_CONTENT_BYTES};
use crate::expected;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

fn state(content: Result<commands::Content, String>) -> AppState {
    AppState {
        content: Mutex::new(content),
        user: Mutex::new(Err("not used".into())),
    }
}

fn real() -> AppState {
    state(commands::load_content_from(
        &commands::content_dir().unwrap(),
        expected::KJV_SQLITE,
        expected::CONTENT_SQLITE,
    ))
}

fn raw() -> Connection {
    content::open_content(
        &commands::content_dir().unwrap().join(MHCC_FILE),
        expected::MHCC_SQLITE,
        MAX_CONTENT_BYTES,
    )
    .unwrap()
}

fn ok(r: CommentaryResult) -> (CmtSource, bool, Vec<CmtGroup>) {
    match r {
        CommentaryResult::Ok {
            source,
            whole_chapter,
            groups,
            ..
        } => (source, whole_chapter, groups),
        CommentaryResult::Unavailable { reason } => panic!("unavailable: {reason}"),
    }
}

fn get(s: &AppState, b: u32, c: u32, v: u32) -> (CmtSource, bool, Vec<CmtGroup>) {
    ok(commands::commentary_for(s, b, c, v).unwrap())
}

fn block_ids(g: &[CmtGroup]) -> Vec<i64> {
    g.iter()
        .flat_map(|g| g.blocks.iter().map(|b| b.id))
        .collect()
}

fn ids(raw: &Connection, sql: &str, p: impl rusqlite::Params) -> Vec<i64> {
    let mut st = raw.prepare(sql).unwrap();
    st.query_map(p, |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

type NodeRow = (u32, Option<u32>, Option<String>, Option<String>);
type AttrRow = (u32, String, String, Option<i64>, Option<i64>);

/// Every node and attribute sent equals its stored row exactly.
fn check_exact(raw: &Connection, groups: &[CmtGroup]) {
    for g in groups {
        for b in &g.blocks {
            let mut st = raw
                .prepare("SELECT n, parent, name, text FROM cmt_node WHERE block = ?1 ORDER BY n")
                .unwrap();
            let rows: Vec<NodeRow> = st
                .query_map([b.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            let got: Vec<_> = b
                .nodes
                .iter()
                .map(|n| (n.n, n.parent, n.name.clone(), n.text.clone()))
                .collect();
            assert_eq!(got, rows, "block {}", b.id);
            let mut st = raw
                .prepare(
                    "SELECT n, name, value, ref_start, ref_end FROM cmt_attr WHERE block = ?1 ORDER BY n, i",
                )
                .unwrap();
            let rows: Vec<AttrRow> = st
                .query_map([b.id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            let got: Vec<_> = b
                .nodes
                .iter()
                .flat_map(|n| {
                    n.attrs
                        .iter()
                        .map(|a| (n.n, a.name.clone(), a.value.clone(), a.ref_start, a.ref_end))
                })
                .collect();
            assert_eq!(got, rows, "block {} attrs", b.id);
        }
    }
}

#[test]
fn opens_verified_with_the_owner_approved_label() {
    let s = real();
    let info = commands::info(&s);
    assert!(info.commentary_available, "{:?}", info.commentary_error);
    let (src, _, _) = get(&s, 43, 3, 3);
    // W2-LBL: label.status is approved_by_owner in the committed file
    // (owner, 2026-10-27): the author (with tradition) and date lines are
    // sent exactly as the scholar wrote them. The gate itself is tested on a
    // pending fixture in commentary.rs.
    assert_eq!(
        src,
        CmtSource {
            title: "Matthew Henry's Concise Commentary".into(),
            approved: true,
            author_line: Some(
                "Matthew Henry \u{b7} Presbyterian (English Nonconformist), with Thomas Scott \u{b7} Anglican"
                    .into()
            ),
            date_line: Some(
                "Abridgement c. 1838 (Religious Tract Society), from Henry's Exposition (1708\u{2013}1710)"
                    .into()
            ),
        }
    );
    assert_ne!(src.title, NEUTRAL_TITLE);
}

/// The gate on the real rows: the committed meta with `label.status` set
/// back to the scholar's pending value sends no author or date line.
#[test]
fn the_real_label_rows_with_a_pending_status_are_gated() {
    let raw = raw();
    let mut m: std::collections::BTreeMap<String, String> = raw
        .prepare("SELECT key, value FROM meta")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        m.get("label.status").map(String::as_str),
        Some(crate::commentary::LABEL_APPROVED)
    );
    m.insert(
        "label.status".into(),
        "recommended_pending_owner_approval".into(),
    );
    assert_eq!(
        crate::commentary::source_from_meta(&m),
        CmtSource {
            title: "Matthew Henry's Concise Commentary".into(),
            approved: false,
            author_line: None,
            date_line: None,
        }
    );
}

#[test]
fn a_verse_in_an_anchor_gets_that_anchors_blocks() {
    let (s, raw) = (real(), raw());
    let (_, whole, g) = get(&s, 43, 3, 3);
    assert!(!whole);
    assert_eq!(g.len(), 1);
    assert_eq!(
        (g[0].first, g[0].last),
        (Some(43_003_001), Some(43_003_008))
    );
    let want = ids(
        &raw,
        "SELECT b.id FROM cmt_anchor_verse v JOIN cmt_block b ON b.anchor = v.anchor
          WHERE v.verse_id = 43003003 ORDER BY b.id",
        [],
    );
    assert_eq!(block_ids(&g), want);
    check_exact(&raw, &g);
    // A verse two anchors list: both, in document order.
    let two: i64 = raw
        .query_row(
            "SELECT verse_id FROM cmt_anchor_verse GROUP BY verse_id
             HAVING count(DISTINCT anchor) = 2 ORDER BY verse_id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let (b, c, v) = (
        (two / 1_000_000) as u32,
        (two / 1000 % 1000) as u32,
        (two % 1000) as u32,
    );
    let (_, whole, g) = get(&s, b, c, v);
    assert!(!whole);
    assert_eq!(g.len(), 2, "verse {two}");
    assert!(block_ids(&g).windows(2).all(|w| w[0] < w[1]));
    check_exact(&raw, &g);
}

#[test]
fn a_verse_in_no_anchor_gets_the_whole_chapter() {
    let (s, raw) = (real(), raw());
    // John 3:16 (DESIGN.md 17.4: the source's John 3 anchors are 1-8, 22-36).
    for (b, c, v) in [(43, 3, 16), (3, 19, 1), (19, 108, 1)] {
        let (_, whole, g) = get(&s, b, c, v);
        assert!(whole, "{b} {c}:{v}");
        assert_eq!(g.len(), 1);
        assert_eq!((g[0].first, g[0].last), (None, None));
        let want = ids(
            &raw,
            "SELECT id FROM cmt_block WHERE book = ?1 AND chapter = ?2 ORDER BY id",
            [b, c],
        );
        assert!(!want.is_empty());
        assert_eq!(block_ids(&g), want);
        check_exact(&raw, &g);
    }
}

#[test]
fn descending_references_are_sent_as_written() {
    let (s, raw) = (real(), raw());
    let (block, book, chapter): (i64, u32, u32) = raw
        .query_row(
            "SELECT b.id, b.book, b.chapter FROM cmt_attr a JOIN cmt_block b ON b.id = a.block
              WHERE a.value = 'Exod.13.2-Exod.10.13'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    let anchor_verse: i64 = raw
        .query_row(
            "SELECT v.verse_id FROM cmt_block b JOIN cmt_anchor_verse v ON v.anchor = b.anchor
              WHERE b.id = ?1 ORDER BY v.k LIMIT 1",
            [block],
            |r| r.get(0),
        )
        .unwrap_or(i64::from(book) * 1_000_000 + i64::from(chapter) * 1000 + 1);
    let (_, _, g) = get(&s, book, chapter, (anchor_verse % 1000) as u32);
    let a = g
        .iter()
        .flat_map(|g| &g.blocks)
        .flat_map(|b| &b.nodes)
        .flat_map(|n| &n.attrs)
        .find(|a| a.value == "Exod.13.2-Exod.10.13")
        .expect("the reference is sent");
    assert_eq!((a.ref_start, a.ref_end), (Some(2_013_002), Some(2_010_013)));
}

#[test]
fn arguments_are_checked() {
    let s = real();
    for (b, c, v) in [(0, 1, 1), (67, 1, 1), (43, 0, 1), (43, 3, 0), (43, 1000, 1)] {
        assert_eq!(
            commands::commentary_for(&s, b, c, v).unwrap_err(),
            "invalid verse"
        );
    }
    assert_eq!(
        commands::commentary_for(&s, 43, 3, 99).unwrap_err(),
        "no such verse"
    );
}

#[test]
fn missing_or_wrong_file_is_unavailable_and_the_bible_is_not_affected() {
    let dir = std::env::temp_dir().join(format!("rvl-cmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(
        commands::content_dir().unwrap().join(KJV_FILE),
        dir.join(KJV_FILE),
    )
    .unwrap();
    let s = state(commands::load_content_from(
        &dir,
        expected::KJV_SQLITE,
        expected::CONTENT_SQLITE,
    ));
    assert!(matches!(
        commands::commentary_for(&s, 43, 3, 16).unwrap(),
        CommentaryResult::Unavailable { .. }
    ));
    assert!(!commands::info(&s).commentary_available);
    assert!(commands::info(&s).content_error.is_none());
    // Another file under the commentary's name: refused by the hash.
    std::fs::copy(dir.join(KJV_FILE), dir.join(MHCC_FILE)).unwrap();
    assert!(Commentary::open(&dir.join(MHCC_FILE), expected::MHCC_SQLITE).is_err());
    // A content file whose hash is given but that isn't the commentary:
    // refused by the identity check.
    let e = Commentary::open(&dir.join(MHCC_FILE), expected::KJV_SQLITE)
        .err()
        .unwrap();
    assert!(e.contains("not the expected commentary"), "{e}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------- golden

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/mhcc-render.json")
}

/// John 3:3 (anchor 1-8), John 3:16 (whole chapter) and Psalm 23:1, exactly
/// as the command returns them, for tests/commentary-render.test.ts.
fn golden() -> serde_json::Value {
    let s = real();
    let mut out = serde_json::Map::new();
    for (key, b, c, v) in [
        ("John.3.3", 43, 3, 3),
        ("John.3.16", 43, 3, 16),
        ("Ps.23.1", 19, 23, 1),
    ] {
        out.insert(
            key.into(),
            serde_json::to_value(commands::commentary_for(&s, b, c, v).unwrap()).unwrap(),
        );
    }
    serde_json::Value::Object(out)
}

#[test]
fn golden_equals_the_real_commentary() {
    let text = std::fs::read_to_string(golden_path()).expect("run regenerate_commentary_golden");
    let file: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(file, golden());
}

#[test]
#[ignore]
fn regenerate_commentary_golden() {
    let mut text = serde_json::to_string_pretty(&golden()).unwrap();
    text.push('\n');
    std::fs::write(golden_path(), text).unwrap();
}
