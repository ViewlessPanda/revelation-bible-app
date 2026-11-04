//! Stage 5, check (DESIGN.md 2 step 4, 8, 11): run on the connection that
//! `open_verified` made from the hashed bytes. Requires the pinned schema,
//! `integrity_check` = ok, no foreign-key violations, the section 8 queries
//! empty (range ends, R1, R3, R6, q_ord), and the round trip against the
//! parsed model: T3 (books, chapters, verse numbering), T4 (text, marks,
//! words, keys), T7 (red pieces per verse), T8 (per-book red totals, also
//! against the pinned facts). Every failure names its check; all are
//! collected so a test can see which fired.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::commentary::{self as cmt, CmtModel, CmtNodeKind, is_kjv_verse, serialise_block};
use crate::db::{CMT_SCHEMA, LEX_SCHEMA, Meta, SCHEMA, query_strings};
use crate::error::{Collector, Error, Result};
use crate::lexicon::{LexModel, NodeKind};
use crate::osis::{Model, RedPiece};
use crate::profile::Profile;
use crate::validate::red_totals;
use crate::vrs::Table;

/// Section 8: range ends inside the verse.
pub const Q_RANGE_ENDS: &str = "\
SELECT 'mark' AS t, verse_id, n FROM mark JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
UNION ALL SELECT 'red_piece', verse_id, n FROM red_piece JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
UNION ALL SELECT 'word', verse_id, n FROM word JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)";

/// R1: every piece is non-empty and inside its verse.
pub const Q_R1: &str = "\
SELECT 'R1', r.verse_id, r.n FROM red_piece r JOIN verse v ON v.id = r.verse_id
 WHERE NOT (r.start_cp < r.end_cp AND r.end_cp <= length(v.text))";

/// R3: in each verse n runs 0, 1, 2, … in position order, at least one
/// character between pieces.
pub const Q_R3: &str = "\
SELECT 'R3', a.verse_id, a.n FROM red_piece a
  JOIN red_piece b ON b.verse_id = a.verse_id AND b.n = a.n + 1
 WHERE b.start_cp <= a.end_cp
UNION ALL
SELECT 'R3', verse_id, NULL FROM red_piece GROUP BY verse_id
HAVING min(n) <> 0 OR max(n) <> count(*) - 1";

/// R6: the pieces of one q_ord sit in consecutive verses, one per verse;
/// inner edges are 0 and length.
pub const Q_R6: &str = "\
WITH p AS (
  SELECT r.q_ord, r.verse_id, r.start_cp, r.end_cp, length(v.text) AS len,
         lag(r.verse_id) OVER w AS prev_v, row_number() OVER w AS i,
         count(*) OVER (PARTITION BY r.q_ord) AS k
    FROM red_piece r JOIN verse v ON v.id = r.verse_id
  WINDOW w AS (PARTITION BY r.q_ord ORDER BY r.verse_id))
SELECT 'R6', verse_id, q_ord FROM p
 WHERE (i > 1 AND start_cp <> 0) OR (i < k AND end_cp <> len)
    OR (i > 1 AND (prev_v = verse_id
                   OR EXISTS (SELECT 1 FROM verse WHERE id > p.prev_v AND id < p.verse_id)))";

/// q_ord runs 1..N with no gaps. (DESIGN.md 8 selects NULL, NULL here;
/// SQLite 3.54 rejects HAVING on a query with no aggregate in its select
/// list, so the aggregates are selected instead. Same rows, same meaning.)
pub const Q_QORD: &str = "SELECT 'q_ord', count(DISTINCT q_ord), max(q_ord) FROM red_piece HAVING count(DISTINCT q_ord) <> max(q_ord)";

fn sqlerr(check: &'static str) -> impl Fn(rusqlite::Error) -> Error {
    move |e| Error::new(check, format!("SQLite: {e}"))
}

type SchemaRow = (String, String, String, Option<String>);

fn schema_rows(conn: &Connection) -> rusqlite::Result<Vec<SchemaRow>> {
    let mut st =
        conn.prepare("SELECT type, name, tbl_name, sql FROM sqlite_schema ORDER BY rowid")?;
    let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
    // VACUUM rewrites sqlite_schema in its own order, so compare as a set.
    let mut v: Vec<SchemaRow> = rows.collect::<rusqlite::Result<_>>()?;
    v.sort();
    Ok(v)
}

/// The `sqlite_schema` rows that `schema` ([`SCHEMA`] or [`LEX_SCHEMA`])
/// creates.
pub fn expected_schema(schema: &str) -> Vec<SchemaRow> {
    let c = Connection::open_in_memory().expect("in-memory database");
    c.execute_batch(schema).expect("schema is valid SQL");
    schema_rows(&c).expect("read schema")
}

fn count_rows(conn: &Connection, q: &str) -> rusqlite::Result<usize> {
    let mut st = conn.prepare(q)?;
    let mut rows = st.query([])?;
    let mut n = 0;
    while rows.next()?.is_some() {
        n += 1;
    }
    Ok(n)
}

/// Structural checks that need no model: schema, integrity, foreign keys,
/// the section 8 queries.
fn structure(
    conn: &Connection,
    schema: &str,
    queries: &[(&'static str, &str)],
    c: &mut Collector,
) -> std::result::Result<(), Error> {
    let s = sqlerr("check");
    if schema_rows(conn).map_err(&s)? != expected_schema(schema) {
        c.push(Error::new(
            "schema",
            "sqlite_schema differs from the pinned schema",
        ));
        return Ok(());
    }
    let app: i64 = conn
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(&s)?;
    let uv: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(&s)?;
    if app != crate::db::APPLICATION_ID || uv != crate::db::SCHEMA_VERSION {
        c.push(Error::new(
            "schema",
            "application_id or user_version differs",
        ));
    }
    let ic = query_strings(conn, "PRAGMA integrity_check").map_err(&s)?;
    if ic != ["ok"] {
        c.push(Error::new(
            "integrity",
            format!("integrity_check: {:?}", &ic[..ic.len().min(5)]),
        ));
    }
    let fk = count_rows(conn, "PRAGMA foreign_key_check").map_err(&s)?;
    if fk != 0 {
        c.push(Error::new("fk", format!("foreign_key_check: {fk} rows")));
    }
    for &(check, q) in queries {
        let n = count_rows(conn, q).map_err(&s)?;
        if n != 0 {
            c.push(Error::new(
                check,
                format!("{n} rows from the {check} query on the built file"),
            ));
        }
    }
    Ok(())
}

fn grouped<T>(
    conn: &Connection,
    q: &str,
    f: impl Fn(&rusqlite::Row<'_>) -> rusqlite::Result<(i64, T)>,
) -> rusqlite::Result<HashMap<i64, Vec<T>>> {
    let mut st = conn.prepare(q)?;
    let mut out: HashMap<i64, Vec<T>> = HashMap::new();
    let mut rows = st.query([])?;
    while let Some(r) = rows.next()? {
        let (k, v) = f(r)?;
        out.entry(k).or_default().push(v);
    }
    Ok(out)
}

/// `meta` and `source` rows equal the build's.
fn meta_rows(conn: &Connection, meta: &Meta, c: &mut Collector) -> std::result::Result<(), Error> {
    let s = sqlerr("check");
    let mut st = conn
        .prepare("SELECT key, value FROM meta ORDER BY key")
        .map_err(&s)?;
    let got: Vec<(String, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    if got != meta.entries {
        c.push(Error::new("meta", "meta rows differ from the build's"));
    }
    let mut st = conn
        .prepare("SELECT id, file, size, sha256 FROM source ORDER BY id")
        .map_err(&s)?;
    let got: Vec<crate::db::SourceRow> = st
        .query_map([], |r| {
            Ok(crate::db::SourceRow {
                id: r.get(0)?,
                file: r.get(1)?,
                size: r.get(2)?,
                sha256: r.get(3)?,
            })
        })
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let mut want = meta.sources.clone();
    want.sort_by(|a, b| a.id.cmp(&b.id));
    if got != want {
        c.push(Error::new(
            "meta",
            "source rows differ from the verified sources",
        ));
    }

    Ok(())
}

/// Round trip and totals against the model (T3, T4, T7, T8).
fn round_trip(
    conn: &Connection,
    m: &Model,
    p: &Profile,
    table: Option<&Table>,
    meta: &Meta,
    c: &mut Collector,
) -> std::result::Result<(), Error> {
    let s = sqlerr("check");

    meta_rows(conn, meta, c)?;

    // T3: books in canonical order; chapters; every chapter numbered 1..n.
    let mut st = conn
        .prepare("SELECT ord, osis, usfm, chapters, main_title FROM book ORDER BY ord")
        .map_err(&s)?;
    type BookRow = (i64, String, String, i64, Option<String>);
    let books: Vec<BookRow> = st
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let want_books: Vec<BookRow> = m
        .books
        .iter()
        .map(|b| {
            (
                i64::from(b.ord),
                b.osis.clone(),
                b.usfm.clone(),
                i64::from(b.chapters),
                b.main_title.clone(),
            )
        })
        .collect();
    if books.len() != want_books.len() {
        c.push(Error::new("T3", "book table differs from the source"));
    }
    for (got, want) in books.iter().zip(&want_books) {
        if (got.0, &got.1, &got.2, got.3) != (want.0, &want.1, &want.2, want.3) {
            c.push(Error::new(
                "T3",
                format!("{}: book row differs from the source", want.1),
            ));
        }
        if got.4 != want.4 {
            c.push(Error::new(
                "title",
                format!("{}: main_title differs from the source", want.1),
            ));
        }
    }
    let mut st = conn
        .prepare("SELECT book, chapter, count(*), min(verse), max(verse) FROM verse GROUP BY book, chapter ORDER BY book, chapter")
        .map_err(&s)?;
    let chapters: Vec<(i64, i64, i64, i64, i64)> = st
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    for &(b, ch, n, lo, hi) in &chapters {
        if lo != 1 || hi != n {
            c.push(Error::new(
                "T3",
                format!("book {b} chapter {ch}: verses not numbered 1..{n}"),
            ));
        }
    }
    if let Some(t) = table {
        let mut want = Vec::new();
        for (bi, (_, chs)) in t.books.iter().enumerate() {
            for (ci, &n) in chs.iter().enumerate() {
                want.push(((bi + 1) as i64, (ci + 1) as i64, i64::from(n)));
            }
        }
        let got: Vec<(i64, i64, i64)> = chapters
            .iter()
            .map(|&(b, ch, n, _, _)| (b, ch, n))
            .collect();
        if got != want {
            let diff = got.iter().zip(&want).filter(|(a, b)| a != b).count()
                + got.len().abs_diff(want.len());
            c.push(Error::new(
                "T3",
                format!("{diff} chapters differ from the versification table"),
            ));
        }
        if let Some(f) = &p.facts
            && (got.len() != f.chapters
                || chapters.iter().map(|x| x.2).sum::<i64>() as usize != f.verses)
        {
            c.push(Error::new(
                "T3",
                "chapter or verse total differs from the pinned facts",
            ));
        }
    }

    // T4: verse text, byte for byte, every verse.
    let mut st = conn
        .prepare("SELECT id, text FROM verse ORDER BY id")
        .map_err(&s)?;
    let mut rows = st.query([]).map_err(&s)?;
    let mut i = 0;
    let mut bad = 0usize;
    while let Some(r) = rows.next().map_err(&s)? {
        let id: i64 = r.get(0).map_err(&s)?;
        let text: String = r.get(1).map_err(&s)?;
        match m.verses.get(i) {
            Some(v) if v.id() == id && v.text == text => {}
            Some(v) => {
                bad += 1;
                if bad <= 5 {
                    c.push(
                        Error::new(
                            "T4",
                            format!("verse {id}: text or id differs from the source"),
                        )
                        .verse(v.osis.clone()),
                    );
                }
            }
            None => bad += 1,
        }
        i += 1;
    }
    if i != m.verses.len() {
        c.push(Error::new(
            "T4",
            format!(
                "{i} verses in the database, {} in the source",
                m.verses.len()
            ),
        ));
    } else if bad > 5 {
        c.push(Error::new("T4", format!("{bad} verses differ in all")));
    }

    // T4: marks, words and keys; T7: red pieces.
    let marks = grouped(
        conn,
        "SELECT verse_id, n, kind, start_cp, end_cp FROM mark ORDER BY verse_id, n",
        |r| {
            Ok((
                r.get(0)?,
                (
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, u32>(3)?,
                    r.get::<_, u32>(4)?,
                ),
            ))
        },
    )
    .map_err(&s)?;
    let red = grouped(
        conn,
        "SELECT verse_id, n, start_cp, end_cp, q_ord FROM red_piece ORDER BY verse_id, n",
        |r| {
            Ok((
                r.get(0)?,
                (
                    r.get::<_, i64>(1)?,
                    RedPiece {
                        start: r.get(2)?,
                        end: r.get(3)?,
                        q_ord: r.get(4)?,
                    },
                ),
            ))
        },
    )
    .map_err(&s)?;
    let words = grouped(
        conn,
        "SELECT verse_id, n, start_cp, end_cp FROM word ORDER BY verse_id, n",
        |r| {
            Ok((
                r.get(0)?,
                (
                    r.get::<_, i64>(1)?,
                    r.get::<_, u32>(2)?,
                    r.get::<_, u32>(3)?,
                ),
            ))
        },
    )
    .map_err(&s)?;
    let keys = grouped(
        conn,
        "SELECT verse_id * 10000 + n, k, key FROM word_key ORDER BY verse_id, n, k",
        |r| Ok((r.get(0)?, (r.get::<_, i64>(1)?, r.get::<_, String>(2)?))),
    )
    .map_err(&s)?;
    let (mut bad_marks, mut bad_red, mut bad_words) = (0usize, 0usize, 0usize);
    let (mut n_marks, mut n_red, mut n_words, mut n_keys) = (0usize, 0usize, 0usize, 0usize);
    for v in &m.verses {
        let id = v.id();
        let got_m: &[(i64, String, u32, u32)] = marks.get(&id).map(|x| &x[..]).unwrap_or(&[]);
        n_marks += got_m.len();
        let want_m: Vec<(i64, String, u32, u32)> = v
            .marks
            .iter()
            .enumerate()
            .map(|(n, mk)| (n as i64, mk.kind.as_str().to_string(), mk.start, mk.end))
            .collect();
        if got_m != want_m.as_slice() {
            bad_marks += 1;
            if bad_marks <= 5 {
                c.push(Error::new("T4", "marks differ from the source").verse(v.osis.clone()));
            }
        }
        let got_r: &[(i64, RedPiece)] = red.get(&id).map(|x| &x[..]).unwrap_or(&[]);
        n_red += got_r.len();
        let want_r: Vec<(i64, RedPiece)> = v
            .red
            .iter()
            .enumerate()
            .map(|(n, r)| (n as i64, *r))
            .collect();
        if got_r != want_r.as_slice() {
            bad_red += 1;
            if bad_red <= 5 {
                c.push(
                    Error::new(
                        "T7",
                        format!(
                            "red pieces {:?} differ from the source {:?}",
                            got_r.iter().map(|x| x.1).collect::<Vec<_>>(),
                            v.red
                        ),
                    )
                    .verse(v.osis.clone()),
                );
            }
        }
        let got_w: &[(i64, u32, u32)] = words.get(&id).map(|x| &x[..]).unwrap_or(&[]);
        n_words += got_w.len();
        let mut same = got_w.len() == v.words.len();
        for (n, w) in v.words.iter().enumerate() {
            if !same {
                break;
            }
            let g = got_w[n];
            let ks: &[(i64, String)] = keys
                .get(&(id * 10000 + n as i64))
                .map(|x| &x[..])
                .unwrap_or(&[]);
            n_keys += ks.len();
            let want_k: Vec<(i64, String)> = w
                .keys
                .iter()
                .enumerate()
                .map(|(k, key)| (k as i64, key.to_string()))
                .collect();
            same = g == (n as i64, w.start, w.end) && ks == want_k.as_slice();
        }
        if !same {
            bad_words += 1;
            if bad_words <= 5 {
                c.push(
                    Error::new("T4", "words or Strong's keys differ from the source")
                        .verse(v.osis.clone()),
                );
            }
        }
    }
    fn total<T>(x: &HashMap<i64, Vec<T>>) -> usize {
        x.values().map(Vec::len).sum()
    }
    if n_marks != total(&marks) {
        c.push(Error::new("T4", "mark rows for verses not in the source"));
    }
    if n_red != total(&red) {
        c.push(Error::new(
            "T7",
            "red_piece rows for verses not in the source",
        ));
    }
    if n_words != total(&words) || n_keys != total(&keys) {
        c.push(Error::new("T4", "word or word_key rows not in the source"));
    }
    for (check, n) in [
        ("T4 marks", bad_marks),
        ("T7", bad_red),
        ("T4 words", bad_words),
    ] {
        if n > 5 {
            c.push(Error::new(
                if check == "T7" { "T7" } else { "T4" },
                format!("{n} verses differ in all ({check})"),
            ));
        }
    }

    // T8: per-book red totals from the database equal the source and the facts.
    let mut st = conn
        .prepare(
            "SELECT v.book, count(*), sum(r.end_cp - r.start_cp) FROM red_piece r JOIN verse v ON v.id = r.verse_id GROUP BY v.book",
        )
        .map_err(&s)?;
    let pieces: HashMap<i64, (i64, i64)> = st
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let mut st = conn
        .prepare(
            "SELECT v.book, count(*) FROM red_piece r JOIN verse v ON v.id = r.verse_id
              WHERE r.verse_id = (SELECT min(verse_id) FROM red_piece x WHERE x.q_ord = r.q_ord) GROUP BY v.book",
        )
        .map_err(&s)?;
    let qs: HashMap<i64, i64> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let model_totals = red_totals(m);
    for b in &m.books {
        let ord = i64::from(b.ord);
        let (pc, cp) = pieces.get(&ord).copied().unwrap_or((0, 0));
        let q = qs.get(&ord).copied().unwrap_or(0);
        let got = (q as u32, pc as u32, cp as u64);
        let src = model_totals.get(&b.ord).copied().unwrap_or_default();
        if got != src {
            c.push(Error::new(
                "T8",
                format!(
                    "{}: database q/pieces/red cp {got:?}, source {src:?}",
                    b.osis
                ),
            ));
        }
        if let Some(f) = &p.facts {
            let fact = f
                .red
                .iter()
                .find(|r| r.book == b.osis)
                .map(|r| (r.q, r.pieces, r.cp))
                .unwrap_or_default();
            if got != fact {
                c.push(Error::new(
                    "T8",
                    format!(
                        "{}: database q/pieces/red cp {got:?}, pinned fact {fact:?}",
                        b.osis
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Every stage-5 check on a connection from `open_verified`.
pub fn check(
    conn: &Connection,
    m: &Model,
    p: &Profile,
    table: Option<&Table>,
    meta: &Meta,
) -> Result<()> {
    let mut c = Collector::default();
    let queries = [
        ("range", Q_RANGE_ENDS),
        ("R1", Q_R1),
        ("R3", Q_R3),
        ("R6", Q_R6),
        ("R6", Q_QORD),
    ];
    structure(conn, SCHEMA, &queries, &mut c)?;
    if !c.has_check("schema") {
        round_trip(conn, m, p, table, meta, &mut c)?;
    }
    c.finish()
}

// ------------------------------------------------------------------ content.sqlite

/// Lexicon invariants run on the built file, each required to return zero
/// rows (DESIGN.md 7, 8).
pub const LEX_QUERIES: [(&str, &str); 6] = [
    // A node's parent is an element node of the same entry.
    (
        "T14",
        "SELECT c.key, c.n FROM lex_node c JOIN lex_node p ON p.key = c.key AND p.n = c.parent WHERE p.name IS NULL",
    ),
    // Attributes sit on element nodes only.
    (
        "T14",
        "SELECT a.key, a.n FROM lex_attr a JOIN lex_node x ON x.key = a.key AND x.n = a.n WHERE x.name IS NULL",
    ),
    // Nodes of an entry are numbered 0..k-1; every entry has nodes.
    (
        "T14",
        "SELECT key FROM lex_node GROUP BY key HAVING min(n) <> 0 OR max(n) <> count(*) - 1",
    ),
    (
        "T14",
        "SELECT key FROM lex_entry WHERE key NOT IN (SELECT key FROM lex_node)",
    ),
    // ord runs 1..N in each lexicon.
    (
        "T14",
        "SELECT lexicon FROM lex_entry GROUP BY lexicon HAVING min(ord) <> 1 OR max(ord) <> count(*) OR count(DISTINCT ord) <> count(*)",
    ),
    // Every cross-reference resolves.
    (
        "lex-ref",
        "SELECT key, n FROM lex_attr WHERE ref_key IS NOT NULL AND ref_key NOT IN (SELECT key FROM lex_entry)",
    ),
];

type NodeRow = (Option<i64>, Option<String>, Option<String>);
type AttrRow = (i64, i64, String, String, Option<String>);

/// Stage 5 for content.sqlite: schema, integrity, foreign keys, the lexicon
/// queries, meta and source rows, and T14: every entry's tree (node order,
/// parents, names, text bytes, attributes in order with their reference
/// keys) equals the parse, for every entry of every lexicon.
pub fn check_lex(conn: &Connection, lexicons: &[&LexModel], meta: &Meta) -> Result<()> {
    let mut c = Collector::default();
    structure(conn, LEX_SCHEMA, &LEX_QUERIES, &mut c)?;
    if c.has_check("schema") {
        return c.finish();
    }
    let s = sqlerr("check");
    meta_rows(conn, meta, &mut c)?;

    let mut st = conn
        .prepare("SELECT key, lexicon, ord FROM lex_entry")
        .map_err(&s)?;
    let entries: HashMap<String, (String, i64)> = st
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let mut nodes: HashMap<String, Vec<NodeRow>> = HashMap::new();
    let mut st = conn
        .prepare("SELECT key, parent, name, text FROM lex_node ORDER BY key, n")
        .map_err(&s)?;
    let mut rows = st.query([]).map_err(&s)?;
    while let Some(r) = rows.next().map_err(&s)? {
        nodes.entry(r.get(0).map_err(&s)?).or_default().push((
            r.get(1).map_err(&s)?,
            r.get(2).map_err(&s)?,
            r.get(3).map_err(&s)?,
        ));
    }
    let mut attrs: HashMap<String, Vec<AttrRow>> = HashMap::new();
    let mut st = conn
        .prepare("SELECT key, n, i, name, value, ref_key FROM lex_attr ORDER BY key, n, i")
        .map_err(&s)?;
    let mut rows = st.query([]).map_err(&s)?;
    while let Some(r) = rows.next().map_err(&s)? {
        attrs.entry(r.get(0).map_err(&s)?).or_default().push((
            r.get(1).map_err(&s)?,
            r.get(2).map_err(&s)?,
            r.get(3).map_err(&s)?,
            r.get(4).map_err(&s)?,
            r.get(5).map_err(&s)?,
        ));
    }

    let mut bad = 0usize;
    let mut seen = 0usize;
    for m in lexicons {
        for e in &m.entries {
            seen += 1;
            let key = e.key.to_string();
            let want_nodes: Vec<NodeRow> = e
                .nodes
                .iter()
                .map(|n| match &n.kind {
                    NodeKind::Element { name, .. } => {
                        (n.parent.map(i64::from), Some(name.clone()), None)
                    }
                    NodeKind::Text(t) => (n.parent.map(i64::from), None, Some(t.clone())),
                })
                .collect();
            let want_attrs: Vec<AttrRow> = e
                .nodes
                .iter()
                .enumerate()
                .flat_map(|(n, node)| {
                    let list = match &node.kind {
                        NodeKind::Element { attrs, .. } => &attrs[..],
                        NodeKind::Text(_) => &[][..],
                    };
                    list.iter().enumerate().map(move |(i, a)| {
                        (
                            n as i64,
                            i as i64,
                            a.name.clone(),
                            a.value.clone(),
                            a.ref_key.map(|k| k.to_string()),
                        )
                    })
                })
                .collect();
            let ok = entries.get(&key) == Some(&(m.source_id.to_string(), i64::from(e.ord)))
                && nodes.get(&key).map(Vec::as_slice).unwrap_or(&[]) == want_nodes.as_slice()
                && attrs.get(&key).map(Vec::as_slice).unwrap_or(&[]) == want_attrs.as_slice();
            if !ok {
                bad += 1;
                if bad <= 5 {
                    c.push(Error::new(
                        "T14",
                        format!("entry {key}: tree differs from the source"),
                    ));
                }
            }
        }
    }
    if bad > 5 {
        c.push(Error::new("T14", format!("{bad} entries differ in all")));
    }
    if entries.len() != seen || nodes.len() > seen || attrs.len() > seen {
        c.push(Error::new(
            "T14",
            format!(
                "{} entries in the database, {seen} in the sources",
                entries.len()
            ),
        ));
    }
    c.finish()
}

// ------------------------------------------------------------------ commentary databases

/// Commentary invariants run on the built file, each required to return zero
/// rows (DESIGN.md 17.5).
pub const CMT_QUERIES: [(&str, &str); 14] = [
    // A node's parent is an element node of the same block.
    (
        "T22",
        "SELECT c.block, c.n FROM cmt_node c JOIN cmt_node p ON p.block = c.block AND p.n = c.parent WHERE p.name IS NULL",
    ),
    // Attributes sit on element nodes only.
    (
        "T22",
        "SELECT a.block, a.n FROM cmt_attr a JOIN cmt_node x ON x.block = a.block AND x.n = a.n WHERE x.name IS NULL",
    ),
    // Nodes of a block are numbered 0..k-1; every block has nodes.
    (
        "T22",
        "SELECT block FROM cmt_node GROUP BY block HAVING min(n) <> 0 OR max(n) <> count(*) - 1",
    ),
    (
        "T22",
        "SELECT id FROM cmt_block WHERE id NOT IN (SELECT block FROM cmt_node)",
    ),
    // Blocks and anchors are numbered 1..N.
    (
        "T22",
        "SELECT count(*) FROM cmt_block HAVING count(*) > 0 AND (min(id) <> 1 OR max(id) <> count(*))",
    ),
    (
        "T22",
        "SELECT count(*) FROM cmt_anchor HAVING count(*) > 0 AND (min(id) <> 1 OR max(id) <> count(*))",
    ),
    // Every anchor lists at least one verse, k = 0..k-1.
    (
        "T22",
        "SELECT anchor FROM cmt_anchor_verse GROUP BY anchor HAVING min(k) <> 0 OR max(k) <> count(*) - 1",
    ),
    (
        "T22",
        "SELECT id FROM cmt_anchor WHERE id NOT IN (SELECT anchor FROM cmt_anchor_verse)",
    ),
    // Every anchor is a non-empty run of consecutive blocks ...
    (
        "T22",
        "SELECT id FROM cmt_anchor WHERE id NOT IN (SELECT anchor FROM cmt_block WHERE anchor IS NOT NULL)",
    ),
    (
        "T22",
        "SELECT anchor FROM cmt_block WHERE anchor IS NOT NULL GROUP BY anchor HAVING max(id) - min(id) + 1 <> count(*)",
    ),
    // ... and anchors follow one another in document order.
    (
        "T22",
        "WITH f AS (SELECT anchor, min(id) AS first FROM cmt_block WHERE anchor IS NOT NULL GROUP BY anchor) \
         SELECT a.anchor FROM f a JOIN f b ON b.anchor = a.anchor + 1 WHERE b.first < a.first",
    ),
    // An anchored block sits in its anchor's chapter.
    (
        "T22",
        "SELECT b.id FROM cmt_block b JOIN cmt_anchor_verse v ON v.anchor = b.anchor \
         WHERE b.chapter IS NULL OR v.verse_id / 1000 <> b.book * 1000 + b.chapter",
    ),
    // Books and chapters never go backwards.
    (
        "T22",
        "SELECT a.id FROM cmt_block a JOIN cmt_block b ON b.id = a.id + 1 \
         WHERE b.book < a.book OR (b.book = a.book AND coalesce(b.chapter, 0) < coalesce(a.chapter, 0))",
    ),
    // Reference verse ids sit exactly on reference@osisRef.
    (
        "cmt-ref",
        "SELECT a.block, a.n FROM cmt_attr a JOIN cmt_node x ON x.block = a.block AND x.n = a.n \
         WHERE (a.ref_start IS NOT NULL) <> (x.name = 'reference' AND a.name = 'osisRef')",
    ),
];

type CmtAttrRow = (i64, i64, String, String, Option<i64>, Option<i64>);

/// Stage 5 for a commentary database: schema, integrity, foreign keys, the
/// commentary queries, meta and source rows, and T22: every block (book,
/// chapter, anchor, node tree, attributes in order with their reference ids)
/// and every anchor (sID, verse list in order) equals the parse, and every
/// block rebuilt from its rows alone ([`serialise_block`]) equals its source
/// bytes in `src`. With `table`, every stored verse id and reference end is
/// a KJV verse (cmt-kjv).
pub fn check_cmt(
    conn: &Connection,
    m: &CmtModel,
    meta: &Meta,
    src: &[u8],
    table: Option<&Table>,
) -> Result<()> {
    let mut c = Collector::default();
    structure(conn, CMT_SCHEMA, &CMT_QUERIES, &mut c)?;
    if c.has_check("schema") {
        return c.finish();
    }
    let s = sqlerr("check");
    meta_rows(conn, meta, &mut c)?;

    let mut st = conn
        .prepare("SELECT id, book, chapter, anchor FROM cmt_block ORDER BY id")
        .map_err(&s)?;
    type BlockRow = (i64, i64, Option<i64>, Option<i64>);
    let blocks: Vec<BlockRow> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let nodes = grouped(
        conn,
        "SELECT block, parent, name, text FROM cmt_node ORDER BY block, n",
        |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?, r.get(3)?))),
    )
    .map_err(&s)?;
    let attrs: HashMap<i64, Vec<CmtAttrRow>> = grouped(
        conn,
        "SELECT block, n, i, name, value, ref_start, ref_end FROM cmt_attr ORDER BY block, n, i",
        |r| {
            Ok((
                r.get(0)?,
                (
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ),
            ))
        },
    )
    .map_err(&s)?;
    let mut st = conn
        .prepare("SELECT id, sid FROM cmt_anchor ORDER BY id")
        .map_err(&s)?;
    let anchors: Vec<(i64, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let verses: HashMap<i64, Vec<i64>> = grouped(
        conn,
        "SELECT anchor, verse_id FROM cmt_anchor_verse ORDER BY anchor, k",
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .map_err(&s)?;

    let mut bad = 0usize;
    let mut note = |c: &mut Collector, msg: String| {
        bad += 1;
        if bad <= 5 {
            c.push(Error::new("T22", msg));
        }
    };
    if blocks.len() != m.blocks.len() {
        note(
            &mut c,
            format!(
                "{} blocks in the database, {} in the source",
                blocks.len(),
                m.blocks.len()
            ),
        );
    }
    let empty_nodes: Vec<cmt::NodeRow> = Vec::new();
    let empty_attrs: Vec<CmtAttrRow> = Vec::new();
    for (i, b) in m.blocks.iter().enumerate() {
        let id = i as i64 + 1;
        let row = blocks.get(i);
        let want_row = (
            id,
            i64::from(b.book),
            b.chapter.map(i64::from),
            b.anchor.map(i64::from),
        );
        let (want_nodes, _) = b.rows();
        let want_attrs: Vec<CmtAttrRow> = b
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(n, node)| {
                let list = match &node.kind {
                    CmtNodeKind::Element { attrs, .. } => &attrs[..],
                    CmtNodeKind::Text(_) => &[][..],
                };
                list.iter().enumerate().map(move |(k, a)| {
                    (
                        n as i64,
                        k as i64,
                        a.name.clone(),
                        a.value.clone(),
                        a.refs.map(|r| i64::from(r.0)),
                        a.refs.map(|r| i64::from(r.1)),
                    )
                })
            })
            .collect();
        let got_nodes = nodes.get(&id).unwrap_or(&empty_nodes);
        let got_attrs = attrs.get(&id).unwrap_or(&empty_attrs);
        if row != Some(&want_row) || *got_nodes != want_nodes || *got_attrs != want_attrs {
            note(
                &mut c,
                format!("block {id}: row or tree differs from the source"),
            );
            continue;
        }
        // Round trip from the stored rows alone.
        let plain: Vec<cmt::AttrRow> = got_attrs
            .iter()
            .map(|a| (a.0, a.2.clone(), a.3.clone()))
            .collect();
        let rebuilt = serialise_block(got_nodes, &plain);
        if src.get(b.start as usize..b.end as usize) != Some(rebuilt.as_bytes()) {
            note(
                &mut c,
                format!("block {id}: rebuilt bytes differ from the source bytes"),
            );
        }
    }
    if anchors.len() != m.anchors.len() {
        note(
            &mut c,
            format!(
                "{} anchors in the database, {} in the source",
                anchors.len(),
                m.anchors.len()
            ),
        );
    }
    for (i, a) in m.anchors.iter().enumerate() {
        let id = i as i64 + 1;
        let want: Vec<i64> = a.verses.iter().map(|&v| i64::from(v)).collect();
        if anchors.get(i) != Some(&(id, a.sid.clone())) || verses.get(&id) != Some(&want) {
            note(
                &mut c,
                format!("anchor {id} ({}): differs from the source", a.sid),
            );
        }
    }
    if bad > 5 {
        c.push(Error::new("T22", format!("{bad} differences in all")));
    }
    if let Some(t) = table {
        let mut ids: Vec<i64> = verses.values().flatten().copied().collect();
        ids.extend(attrs.values().flatten().filter_map(|a| a.4));
        ids.extend(attrs.values().flatten().filter_map(|a| a.5));
        let unresolved: std::collections::BTreeSet<i64> = ids
            .into_iter()
            .filter(|&v| !u32::try_from(v).is_ok_and(|v| is_kjv_verse(t, v)))
            .collect();
        if !unresolved.is_empty() {
            c.push(Error::new(
                "cmt-kjv",
                format!(
                    "{} stored verse ids are not KJV verses: {:?}",
                    unresolved.len(),
                    unresolved.iter().take(20).collect::<Vec<_>>()
                ),
            ));
        }
    }
    c.finish()
}

// ------------------------------------------------------------------ kjv-search.sqlite

/// Search-index invariants on the built file, each required to return zero
/// rows (DESIGN.md 18.5): ids are KJV verse ids; `norm` is non-empty
/// `[a-z0-9]` words separated by single spaces.
pub const SEARCH_QUERIES: [(&str, &str); 2] = [
    (
        "search-id",
        "SELECT rowid FROM verse_fts WHERE rowid NOT BETWEEN 1001001 AND 66999999 \
         OR rowid % 1000 = 0 OR rowid / 1000 % 1000 = 0",
    ),
    (
        "search-norm",
        "SELECT rowid FROM verse_fts WHERE typeof(norm) <> 'text' OR norm = '' \
         OR norm GLOB '*[^a-z0-9 ]*' OR norm GLOB ' *' OR norm GLOB '* ' OR norm GLOB '*  *'",
    ),
];

/// The non-ASCII characters the KJV's stored text uses as word separators
/// (DESIGN.md 18.2): ’ – —. Written out here, separately from
/// `revelation_content::search::SEPARATORS`, for [`search_norm_oracle`].
const KJV_NON_ASCII_SEPARATORS: [char; 3] = ['\u{2019}', '\u{2013}', '\u{2014}'];

/// DESIGN.md 18.2 computed a second way, independently of
/// `revelation_content::search::tokens` (which the build used): map every
/// character to its replacement, then collapse runs of spaces. `None` for a
/// character outside the KJV's known set (ASCII, Æ, ’ – —), so new text
/// can't slip through on a shared mistake.
pub fn search_norm_oracle(text: &str) -> Option<String> {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if c == '\u{00C6}' || c == '\u{00E6}' {
            s.push_str("ae");
        } else if c.is_ascii() || KJV_NON_ASCII_SEPARATORS.contains(&c) {
            s.push(' ');
        } else {
            return None;
        }
    }
    Some(
        s.split(' ')
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Stage 5 for kjv-search.sqlite (DESIGN.md 18.5): pinned schema (with the
/// FTS5 shadow tables), `integrity_check` = ok (which runs FTS5's own
/// integrity check), the [`SEARCH_QUERIES`], `meta`/`source` rows, every
/// verse id of `input` exactly once and nothing else, `norm` equal to
/// [`search_norm_oracle`] of its `verse.text` for every verse, row counts
/// of the FTS5 content and docsize tables, and every verse found by an FTS5
/// phrase query of its own full normalised text.
pub fn check_search(
    conn: &Connection,
    input: &crate::search::SearchInput,
    meta: &Meta,
) -> Result<()> {
    let mut c = Collector::default();
    structure(conn, crate::search::SEARCH_SCHEMA, &SEARCH_QUERIES, &mut c)?;
    if c.has_check("schema") {
        return c.finish();
    }
    let s = sqlerr("check");
    meta_rows(conn, meta, &mut c)?;

    let mut st = conn
        .prepare("SELECT rowid, norm FROM verse_fts ORDER BY rowid")
        .map_err(&s)?;
    let rows: Vec<(i64, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<_>>()
        .map_err(&s)?;
    let n = input.verses.len();
    let mut bad = 0usize;
    let mut note = |c: &mut Collector, check: &'static str, msg: String| {
        bad += 1;
        if bad <= 5 {
            c.push(Error::new(check, msg));
        }
    };
    if rows.len() != n {
        note(
            &mut c,
            "search-count",
            format!("{} index rows for {n} verses", rows.len()),
        );
    }
    for (i, (id, text)) in input.verses.iter().enumerate() {
        match rows.get(i) {
            Some((rid, _)) if rid != id => {
                note(
                    &mut c,
                    "search-id",
                    format!("index row {i} is verse {rid}, expected {id}"),
                );
            }
            Some((_, norm)) => {
                if search_norm_oracle(text).as_deref() != Some(norm.as_str()) {
                    note(
                        &mut c,
                        "search-norm",
                        format!("verse {id}: indexed text is not the normalisation of verse.text"),
                    );
                }
            }
            None => note(
                &mut c,
                "search-id",
                format!("verse {id} is not in the index"),
            ),
        }
    }
    for (table, check) in [
        ("verse_fts_content", "search-count"),
        ("verse_fts_docsize", "search-count"),
    ] {
        let k: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .map_err(&s)?;
        if k as usize != n {
            note(
                &mut c,
                check,
                format!("{table} has {k} rows for {n} verses"),
            );
        }
    }
    // Index against content, through MATCH: each verse is found by the phrase
    // of its own whole text (and the phrase query is built by the same
    // function the app uses).
    let mut st = conn
        .prepare("SELECT rowid FROM verse_fts WHERE verse_fts MATCH ?1 AND rowid = ?2")
        .map_err(&s)?;
    for (id, norm) in &rows {
        let q =
            revelation_content::search::Query::Phrase(norm.split(' ').map(String::from).collect());
        let found = match q.fts_phrase() {
            Some(fts) => st
                .query_map(rusqlite::params![fts, id], |r| r.get::<_, i64>(0))
                .map_err(&s)?
                .count(),
            None => 0,
        };
        if found != 1 {
            note(
                &mut c,
                "search-fts",
                format!("verse {id}: not found by the phrase of its own text"),
            );
        }
    }
    if bad > 5 {
        c.push(Error::new("search", format!("{bad} differences in all")));
    }
    c.finish()
}
