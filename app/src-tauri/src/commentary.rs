//! Commentary (pipeline/DESIGN.md 17; wave-2-layout.md 7): Matthew Henry's
//! Concise Commentary, `mhcc.sqlite`.
//!
//! - Opened only through `open_verified` with the hash compiled in from
//!   `output-sha256.txt`, then the content identity and this file's own
//!   identity (`meta.schema_version`, `meta.commentary_source`,
//!   `label.id`). Missing, damaged or replaced: the panel says "commentary
//!   unavailable" and the Bible is not affected.
//! - Blocks go to the UI as their stored node trees (text exactly as stored);
//!   the UI draws text nodes as text, never markup.
//! - A verse listed by an anchor gets that anchor's blocks (every anchor that
//!   lists it, in document order). A verse in no anchor (2,384, e.g. John
//!   3:16) gets the whole chapter's blocks, flagged `whole_chapter`, never a
//!   guessed anchor (DESIGN.md 17.5).
//! - Author, tradition and date lines are sent only when `label.status` is
//!   exactly [`LABEL_APPROVED`] (17.7).

use crate::content::{MAX_CONTENT_BYTES, open_content};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The commentary's file name inside the content folder.
pub const MHCC_FILE: &str = "mhcc.sqlite";
/// `meta.commentary_source` and `label.id` of that file.
pub const MHCC_SOURCE: &str = "CMT-MHCC-CW20";
/// The one `label.status` value that means the owner approved the labels
/// (pipeline `labels::LABEL_APPROVED`).
pub const LABEL_APPROVED: &str = "approved_by_owner";
/// The neutral heading used while the label is not approved and the
/// display name is not quoted by any citation (DESIGN.md 17.7).
pub const NEUTRAL_TITLE: &str = "Commentary";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CmtAttr {
    pub name: String,
    pub value: String,
    /// `reference@osisRef` only: the verse ids of the range ends, as written
    /// (a descending range keeps `ref_start > ref_end`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CmtNode {
    pub n: u32,
    pub parent: Option<u32>,
    /// Element name, or None for a text node.
    pub name: Option<String>,
    /// Text exactly as stored, or None for an element.
    pub text: Option<String>,
    pub attrs: Vec<CmtAttr>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CmtBlock {
    pub id: i64,
    pub nodes: Vec<CmtNode>,
}

/// One anchor's blocks (or, for the chapter fallback, the chapter's).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CmtGroup {
    /// First and last verse ids the anchor lists (osisID order); for the
    /// chapter fallback, `None`.
    pub first: Option<i64>,
    pub last: Option<i64>,
    pub blocks: Vec<CmtBlock>,
}

/// What the source bar may show (wave-2-layout.md 7, DESIGN.md 17.7).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CmtSource {
    pub title: String,
    /// `label.status` is exactly `approved_by_owner`.
    pub approved: bool,
    /// Only when approved.
    pub author_line: Option<String>,
    /// Only when approved.
    pub date_line: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CommentaryResult {
    Ok {
        source: CmtSource,
        verse_id: i64,
        /// The verse is in no anchor: `groups` holds the chapter's blocks.
        whole_chapter: bool,
        groups: Vec<CmtGroup>,
    },
    /// The file is missing, damaged or not the expected one.
    Unavailable { reason: String },
}

pub struct Commentary {
    conn: Connection,
    pub source: CmtSource,
}

fn sql(e: rusqlite::Error) -> String {
    format!("{MHCC_FILE}: {e}")
}

/// The source bar's lines from the `meta` label rows (DESIGN.md 17.7).
pub fn source_from_meta(meta: &BTreeMap<String, String>) -> CmtSource {
    let approved = meta.get("label.status").map(String::as_str) == Some(LABEL_APPROVED);
    let name = meta.get("label.display_name").filter(|s| !s.is_empty());
    let quoted = name.is_some_and(|n| {
        meta.iter()
            .any(|(k, v)| k.starts_with("label.citations.") && v.contains(n.as_str()))
    });
    let title = match name {
        Some(n) if approved || quoted => n.clone(),
        _ => NEUTRAL_TITLE.to_string(),
    };
    let approved_line = |k: &str| {
        if approved {
            meta.get(k).filter(|s| !s.is_empty()).cloned()
        } else {
            None
        }
    };
    CmtSource {
        title,
        approved,
        author_line: approved_line("label.author_label"),
        date_line: approved_line("label.date_label"),
    }
}

/// Nodes numbered `0..k-1`, each parent an earlier element node (the
/// pipeline's check stage requires the same; checked again so a malformed
/// block is refused, not drawn). `n == i` first keeps `nodes[p]` in bounds.
fn tree_ok(nodes: &[CmtNode]) -> bool {
    nodes.first().is_some_and(|r| r.name.is_some())
        && nodes.iter().enumerate().all(|(i, node)| {
            node.n as usize == i
                && (node.name.is_some() != node.text.is_some())
                && match node.parent {
                    None => i == 0,
                    Some(p) => p < node.n && nodes[p as usize].name.is_some(),
                }
        })
}

impl Commentary {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Commentary, String> {
        let conn =
            open_content(path, expected_sha256, MAX_CONTENT_BYTES).map_err(|e| e.to_string())?;
        let mut meta = BTreeMap::new();
        {
            let mut st = conn.prepare("SELECT key, value FROM meta").map_err(sql)?;
            let rows = st
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(sql)?;
            for row in rows {
                let (k, v) = row.map_err(sql)?;
                meta.insert(k, v);
            }
        }
        let want = |k: &str, v: &str| {
            if meta.get(k).map(String::as_str) == Some(v) {
                Ok(())
            } else {
                Err(format!("{MHCC_FILE}: not the expected commentary ({k})"))
            }
        };
        want("schema_version", "1")?;
        want("commentary_source", MHCC_SOURCE)?;
        want("label.id", MHCC_SOURCE)?;
        let source = source_from_meta(&meta);
        Ok(Commentary { conn, source })
    }

    fn block(&self, id: i64) -> Result<CmtBlock, String> {
        let mut st = self
            .conn
            .prepare("SELECT n, parent, name, text FROM cmt_node WHERE block = ?1 ORDER BY n")
            .map_err(sql)?;
        let mut nodes: Vec<CmtNode> = st
            .query_map([id], |r| {
                Ok(CmtNode {
                    n: r.get(0)?,
                    parent: r.get(1)?,
                    name: r.get(2)?,
                    text: r.get(3)?,
                    attrs: Vec::new(),
                })
            })
            .map_err(sql)?
            .collect::<Result<_, _>>()
            .map_err(sql)?;
        if !tree_ok(&nodes) {
            return Err(format!("{MHCC_FILE}: malformed commentary block"));
        }
        let mut st = self
            .conn
            .prepare(
                "SELECT n, name, value, ref_start, ref_end FROM cmt_attr WHERE block = ?1 ORDER BY n, i",
            )
            .map_err(sql)?;
        let rows = st
            .query_map([id], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    CmtAttr {
                        name: r.get(1)?,
                        value: r.get(2)?,
                        ref_start: r.get(3)?,
                        ref_end: r.get(4)?,
                    },
                ))
            })
            .map_err(sql)?;
        for row in rows {
            let (n, a) = row.map_err(sql)?;
            match nodes.get_mut(n as usize) {
                Some(node) if node.name.is_some() => node.attrs.push(a),
                _ => return Err(format!("{MHCC_FILE}: malformed commentary block")),
            }
        }
        Ok(CmtBlock { id, nodes })
    }

    fn blocks(&self, sql_text: &str, p: impl rusqlite::Params) -> Result<Vec<CmtBlock>, String> {
        let mut st = self.conn.prepare(sql_text).map_err(sql)?;
        let ids: Vec<i64> = st
            .query_map(p, |r| r.get(0))
            .map_err(sql)?
            .collect::<Result<_, _>>()
            .map_err(sql)?;
        ids.into_iter().map(|id| self.block(id)).collect()
    }

    /// The comment on `verse_id` (`book*1000000 + chapter*1000 + verse`):
    /// the blocks of every anchor listing it, else the chapter's blocks.
    pub fn for_verse(&self, verse_id: i64) -> Result<(bool, Vec<CmtGroup>), String> {
        let anchors: Vec<i64> = {
            let mut st = self
                .conn
                .prepare(
                    "SELECT DISTINCT anchor FROM cmt_anchor_verse WHERE verse_id = ?1 ORDER BY anchor",
                )
                .map_err(sql)?;
            st.query_map([verse_id], |r| r.get(0))
                .map_err(sql)?
                .collect::<Result<_, _>>()
                .map_err(sql)?
        };
        if anchors.is_empty() {
            let (book, chapter) = (verse_id / 1_000_000, verse_id / 1000 % 1000);
            let blocks = self.blocks(
                "SELECT id FROM cmt_block WHERE book = ?1 AND chapter = ?2 ORDER BY id",
                params![book, chapter],
            )?;
            let groups = if blocks.is_empty() {
                Vec::new()
            } else {
                vec![CmtGroup {
                    first: None,
                    last: None,
                    blocks,
                }]
            };
            return Ok((true, groups));
        }
        let mut groups = Vec::with_capacity(anchors.len());
        for a in anchors {
            let (first, last): (i64, i64) = self
                .conn
                .query_row(
                    "SELECT (SELECT verse_id FROM cmt_anchor_verse WHERE anchor = ?1 ORDER BY k LIMIT 1),
                            (SELECT verse_id FROM cmt_anchor_verse WHERE anchor = ?1 ORDER BY k DESC LIMIT 1)",
                    [a],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(sql)?;
            let blocks = self.blocks(
                "SELECT id FROM cmt_block WHERE anchor = ?1 ORDER BY id",
                [a],
            )?;
            groups.push(CmtGroup {
                first: Some(first),
                last: Some(last),
                blocks,
            });
        }
        Ok((false, groups))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(rows: &[(&str, &str)]) -> BTreeMap<String, String> {
        rows.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn labels_are_gated_on_exact_approval() {
        let base = [
            ("label.display_name", "Matthew Henry's Concise Commentary"),
            ("label.author_label", "Matthew Henry · Presbyterian"),
            ("label.date_label", "c. 1838"),
            (
                "label.citations.4",
                "CCEL, Matthew Henry's Concise Commentary on the Bible, ThML header",
            ),
        ];
        let with = |status: &str| {
            let mut m = meta(&base);
            m.insert("label.status".into(), status.into());
            source_from_meta(&m)
        };
        let pending = with("recommended_pending_owner_approval");
        assert_eq!(
            pending,
            CmtSource {
                title: "Matthew Henry's Concise Commentary".into(),
                approved: false,
                author_line: None,
                date_line: None,
            }
        );
        for near in [
            "approved",
            "Approved_by_owner",
            "approved_by_owner ",
            "approved_by_owner\u{200B}",
        ] {
            assert_eq!(with(near), pending, "{near:?}");
        }
        let ok = with(LABEL_APPROVED);
        assert!(ok.approved);
        assert_eq!(
            ok.author_line.as_deref(),
            Some("Matthew Henry · Presbyterian")
        );
        assert_eq!(ok.date_line.as_deref(), Some("c. 1838"));
        // Not quoted by any citation and not approved: the neutral heading.
        let mut m = meta(&base[..3]);
        m.insert(
            "label.status".into(),
            "recommended_pending_owner_approval".into(),
        );
        assert_eq!(source_from_meta(&m).title, NEUTRAL_TITLE);
        assert_eq!(source_from_meta(&BTreeMap::new()).title, NEUTRAL_TITLE);
    }

    #[test]
    fn malformed_trees_are_refused() {
        let el = |n, parent, name: &str| CmtNode {
            n,
            parent,
            name: Some(name.into()),
            text: None,
            attrs: vec![],
        };
        let tx = |n, parent| CmtNode {
            n,
            parent,
            name: None,
            text: Some("x".into()),
            attrs: vec![],
        };
        assert!(tree_ok(&[el(0, None, "p"), tx(1, Some(0))]));
        assert!(!tree_ok(&[]));
        assert!(!tree_ok(&[tx(0, None), tx(1, Some(0))]));
        assert!(!tree_ok(&[el(0, None, "p"), tx(1, Some(1))]));
        assert!(!tree_ok(&[el(0, None, "p"), tx(2, Some(0))]));
        assert!(!tree_ok(&[el(0, None, "p"), el(1, None, "p")]));
    }
}
