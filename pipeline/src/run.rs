//! The whole build and the published-file check, as library functions the
//! CLI and the tests share.

use std::path::{Path, PathBuf};

use crate::check;
use crate::commentary::{self, CmtModel};
use crate::db::{self, LEX_DEST, Meta, SourceRow, Step};
use crate::error::{Error, Result};
use crate::labels;
use crate::lexicon::{self, LexModel};
use crate::open::open_verified;
use crate::osis::{self, Model};
use crate::profile::{self, Profile};
use crate::search::{self, SEARCH_DEST, SearchInput};
use crate::sources::{
    self, CAP_COMMENTARY, CAP_KJV, CAP_LEXICON, CAP_OVERRIDES, CAP_VRS, Verified,
};
use crate::validate;
use crate::vrs::{self, Table};

/// Output file name of the KJV pack.
pub const KJV_DEST: &str = "kjv.sqlite";

/// Verified, parsed and validated KJV inputs.
pub struct Prepared {
    pub kjv: Verified,
    pub vrs: Verified,
    pub overrides_sha256: String,
    pub table: Table,
    pub model: Model,
    pub profile: Profile,
}

/// Stages 1–3 for the real KJV: verify (incl. the SOURCES.md rows), parse,
/// T2 table totals, validate.
pub fn prepare_kjv(raw_dir: &Path) -> Result<Prepared> {
    let sums = crate::checksums_dir();
    let md = std::fs::read_to_string(crate::sources_md_path())
        .map_err(|e| Error::new("io", format!("SOURCES.md: {e}")))?;
    let kjv = sources::verify(&sums, raw_dir, "BIB-KJV-CW31", CAP_KJV)?;
    sources::check_sources_md(&md, &kjv.id, &kjv.sha256)?;
    let vrs_src = sources::verify(&sums, raw_dir, "VRS-ENG-PALASO", CAP_VRS)?;
    sources::check_sources_md(&md, &vrs_src.id, &vrs_src.sha256)?;
    let ov_bytes = sources::read_capped(&crate::overrides_path(), CAP_OVERRIDES)?;
    let overrides = vrs::parse_overrides(&ov_bytes)?;
    if overrides != profile::kjv_overrides() {
        return Err(Error::new("T2", "overrides file differs from the reviewed list").into());
    }
    let (eng, _) = vrs::parse_vrs(&vrs_src.bytes)?;
    let table = vrs::apply(&eng, &overrides)?;
    let p = profile::kjv();
    if let Some(f) = &p.facts
        && (table.chapters() != f.chapters || table.verses() != f.verses as u64)
    {
        return Err(Error::new("T2", "versification table does not total 1,189 / 31,102").into());
    }
    let model = osis::parse(&kjv.bytes, &p)?;
    validate::validate(&model, &p, Some(&table))?;
    Ok(Prepared {
        overrides_sha256: sources::sha256_hex(&ov_bytes),
        kjv,
        vrs: vrs_src,
        table,
        model,
        profile: p,
    })
}

impl Prepared {
    pub fn meta(&self) -> Meta {
        let row = |v: &Verified| SourceRow {
            id: v.id.clone(),
            file: v.file.clone(),
            size: v.bytes.len() as i64,
            sha256: v.sha256.clone(),
        };
        Meta::kjv(
            &self.model,
            &self.kjv.id,
            vec![row(&self.kjv), row(&self.vrs)],
            Some(&self.overrides_sha256),
        )
    }
}

/// Stages 4–6 for a parsed, validated model.
pub fn publish_model(
    out_dir: &Path,
    dest: &str,
    model: &Model,
    p: &Profile,
    table: Option<&Table>,
    meta: &Meta,
    hook: &mut dyn FnMut(Step) -> Result<()>,
) -> Result<String> {
    db::publish(
        out_dir,
        dest,
        |conn| db::write(conn, model, meta),
        |conn, _hash| check::check(conn, model, p, table, meta),
        hook,
    )
}

/// Stages 2–6 for any OSIS buffer under `p` (fixtures; the real build adds
/// stage 1 first). Writes nothing unless every check passes.
pub fn build_bytes(
    bytes: &[u8],
    p: &Profile,
    table: Option<&Table>,
    out_dir: &Path,
    dest: &str,
) -> Result<String> {
    let model = osis::parse(bytes, p)?;
    validate::validate(&model, p, table)?;
    let meta = Meta::kjv(&model, p.source_id, Vec::new(), None);
    publish_model(out_dir, dest, &model, p, table, &meta, &mut |_| Ok(()))
}

/// The `check` subcommand on a published file: no sidecar of `<dest>`, the
/// `.sha256` beside it matches, and every stage-5 check passes against a
/// fresh parse of the sources. Returns the file's SHA-256.
pub fn check_published(prep: &Prepared, out_dir: &Path) -> Result<String> {
    let (conn, hash) = open_published(out_dir, KJV_DEST)?;
    let meta = prep.meta();
    check::check(&conn, &prep.model, &prep.profile, Some(&prep.table), &meta)?;
    Ok(hash)
}

/// A published `<out-dir>/<dest>`: no sidecar beside it, opened through
/// `open_verified`, and its `.sha256` file matching.
fn open_published(out_dir: &Path, dest: &str) -> Result<(rusqlite::Connection, String)> {
    let path: PathBuf = out_dir.join(dest);
    let side = db::existing_sidecars(&path);
    if !side.is_empty() {
        return Err(Error::new("sidecar", format!("sidecar beside {dest}: {side:?}")).into());
    }
    let (conn, hash) = open_verified(&path, None)?;
    let sum = sources::read_capped(&out_dir.join(format!("{dest}.sha256")), 4096)?;
    let (want, name) = sources::parse_checksum_file(&sum)?;
    if name != dest || want != hash {
        return Err(Error::new(
            "sha256",
            format!("{dest}.sha256 does not match {dest} (file {hash}, sidecar {want})"),
        )
        .into());
    }
    Ok((conn, hash))
}

// ------------------------------------------------------------------ lexicons

/// Verified, parsed and validated lexicons (stages 1–3, DESIGN.md 7).
pub struct PreparedLex {
    pub oshb: Verified,
    pub mgnt: Verified,
    pub heb: LexModel,
    pub grc: LexModel,
}

impl PreparedLex {
    pub fn models(&self) -> [&LexModel; 2] {
        [&self.heb, &self.grc]
    }

    pub fn meta(&self) -> Meta {
        let row = |v: &Verified| SourceRow {
            id: v.id.clone(),
            file: v.file.clone(),
            size: v.bytes.len() as i64,
            sha256: v.sha256.clone(),
        };
        Meta::lexicons(vec![row(&self.oshb), row(&self.mgnt)])
    }
}

/// Stages 1–3 for both lexicons: verify (with the SOURCES.md rows), parse
/// with every pinned fact, and resolve every cross-reference.
pub fn prepare_lexicons(raw_dir: &Path) -> Result<PreparedLex> {
    let sums = crate::checksums_dir();
    let md = std::fs::read_to_string(crate::sources_md_path())
        .map_err(|e| Error::new("io", format!("SOURCES.md: {e}")))?;
    let oshb = sources::verify(&sums, raw_dir, lexicon::OSHB_ID, CAP_LEXICON)?;
    sources::check_sources_md(&md, &oshb.id, &oshb.sha256)?;
    let mgnt = sources::verify(&sums, raw_dir, lexicon::MGNT_ID, CAP_LEXICON)?;
    sources::check_sources_md(&md, &mgnt.id, &mgnt.sha256)?;
    let heb = lexicon::parse(&oshb.bytes, &lexicon::oshb())?;
    let grc = lexicon::parse(&mgnt.bytes, &lexicon::mgnt())?;
    lexicon::check_cross_refs(&[&heb, &grc])?;
    Ok(PreparedLex {
        oshb,
        mgnt,
        heb,
        grc,
    })
}

/// T13: every Strong's key of the KJV resolves to a used lexicon entry of
/// its language. On failure the error lists **every** unresolved key with
/// its number of word links; nothing is invented or fixed.
pub fn resolve_kjv(model: &Model, lex: &[&LexModel]) -> Result<()> {
    let keys = model
        .verses
        .iter()
        .flat_map(|v| &v.words)
        .flat_map(|w| &w.keys);
    let bad = lexicon::unresolved_keys(keys, lex);
    if bad.is_empty() {
        return Ok(());
    }
    let list: Vec<String> = bad.iter().map(|(k, n)| format!("{k} ({n})")).collect();
    Err(Error::new(
        "T13",
        format!(
            "{} KJV Strong's keys do not resolve to a used lexicon entry: {}",
            bad.len(),
            list.join(", ")
        ),
    )
    .into())
}

/// Stages 4–6 for content.sqlite.
pub fn publish_lexicons(
    out_dir: &Path,
    lex: &PreparedLex,
    hook: &mut dyn FnMut(Step) -> Result<()>,
) -> Result<String> {
    let meta = lex.meta();
    let models = lex.models();
    db::publish(
        out_dir,
        LEX_DEST,
        |conn| db::write_lex(conn, &models, &meta),
        |conn, _hash| check::check_lex(conn, &models, &meta),
        hook,
    )
}

/// The `check` subcommand for a published content.sqlite.
pub fn check_published_lex(lex: &PreparedLex, out_dir: &Path) -> Result<String> {
    let (conn, hash) = open_published(out_dir, LEX_DEST)?;
    check::check_lex(&conn, &lex.models(), &lex.meta())?;
    Ok(hash)
}

/// Stages 2–6 for two lexicon buffers under fixture profiles (synthetic
/// fixtures; the real build adds stage 1 first). Writes nothing unless every
/// check passes.
pub fn build_lex_bytes(oshb: &[u8], mgnt: &[u8], out_dir: &Path) -> Result<String> {
    let heb = lexicon::parse(oshb, &lexicon::fixture(lexicon::oshb()))?;
    let grc = lexicon::parse(mgnt, &lexicon::fixture(lexicon::mgnt()))?;
    lexicon::check_cross_refs(&[&heb, &grc])?;
    let meta = Meta::lexicons(Vec::new());
    let models = [&heb, &grc];
    db::publish(
        out_dir,
        LEX_DEST,
        |conn| db::write_lex(conn, &models, &meta),
        |conn, _hash| check::check_lex(conn, &models, &meta),
        &mut |_| Ok(()),
    )
}

// ------------------------------------------------------------------ commentaries

/// A verified, parsed and resolved commentary (stages 1–3, DESIGN.md 17).
pub struct PreparedCmt {
    pub src: Verified,
    pub vrs: SourceRow,
    pub overrides_sha256: String,
    pub table: Table,
    pub model: CmtModel,
    /// The scholar's label rows plus `label_file`, `label_file_sha256` and
    /// `label_cited_from` (DESIGN.md 17.7).
    pub labels: Vec<(String, String)>,
}

impl PreparedCmt {
    pub fn meta(&self) -> Meta {
        let row = SourceRow {
            id: self.src.id.clone(),
            file: self.src.file.clone(),
            size: self.src.bytes.len() as i64,
            sha256: self.src.sha256.clone(),
        };
        Meta::commentary(
            self.model.source_id,
            vec![row, self.vrs.clone()],
            Some(&self.overrides_sha256),
            self.labels.clone(),
        )
    }
}

/// The label rows for `source_id` from the committed label file: every row
/// of [`labels::parse_labels`], whose `label.id` must be `source_id`, plus
/// the file's repo path, SHA-256 and citation.
pub fn load_labels(source_id: &str) -> Result<Vec<(String, String)>> {
    let bytes = sources::read_capped(&labels::mhcc_labels_path(), labels::CAP_LABELS)?;
    let mut rows = labels::parse_labels(&bytes)?;
    let id = rows
        .iter()
        .find(|r| r.0 == "label.id")
        .map(|r| r.1.as_str());
    if id != Some(source_id) {
        return Err(Error::new("labels", format!("label.id is not {source_id}"))
            .file(labels::MHCC_LABELS_FILE)
            .into());
    }
    rows.push((
        "label_file".into(),
        format!("pipeline/data/{}", labels::MHCC_LABELS_FILE),
    ));
    rows.push(("label_file_sha256".into(), sources::sha256_hex(&bytes)));
    rows.push((
        "label_cited_from".into(),
        labels::MHCC_LABELS_CITED_FROM.into(),
    ));
    Ok(rows)
}

/// Stages 1–3 for MHCC: verify (with the SOURCES.md row), parse with every
/// pinned fact, and resolve every anchor and reference against the KJV
/// versification table of `kjv` (the same table kjv.sqlite is checked
/// against).
pub fn prepare_mhcc(raw_dir: &Path, kjv: &Prepared) -> Result<PreparedCmt> {
    let md = std::fs::read_to_string(crate::sources_md_path())
        .map_err(|e| Error::new("io", format!("SOURCES.md: {e}")))?;
    let src = sources::verify(
        &crate::checksums_dir(),
        raw_dir,
        commentary::MHCC_ID,
        CAP_COMMENTARY,
    )?;
    sources::check_sources_md(&md, &src.id, &src.sha256)?;
    let model = commentary::parse(&src.bytes, &commentary::mhcc())?;
    commentary::resolve(&model, &kjv.table)?;
    Ok(PreparedCmt {
        src,
        vrs: SourceRow {
            id: kjv.vrs.id.clone(),
            file: kjv.vrs.file.clone(),
            size: kjv.vrs.bytes.len() as i64,
            sha256: kjv.vrs.sha256.clone(),
        },
        overrides_sha256: kjv.overrides_sha256.clone(),
        table: kjv.table.clone(),
        model,
        labels: load_labels(commentary::MHCC_ID)?,
    })
}

/// Stages 4–6 for mhcc.sqlite.
pub fn publish_mhcc(
    out_dir: &Path,
    c: &PreparedCmt,
    hook: &mut dyn FnMut(Step) -> Result<()>,
) -> Result<String> {
    let meta = c.meta();
    db::publish(
        out_dir,
        commentary::MHCC_DEST,
        |conn| db::write_cmt(conn, &c.model, &meta),
        |conn, _hash| check::check_cmt(conn, &c.model, &meta, &c.src.bytes, Some(&c.table)),
        hook,
    )
}

/// The `check` subcommand for a published mhcc.sqlite.
pub fn check_published_mhcc(c: &PreparedCmt, out_dir: &Path) -> Result<String> {
    let (conn, hash) = open_published(out_dir, commentary::MHCC_DEST)?;
    check::check_cmt(&conn, &c.model, &c.meta(), &c.src.bytes, Some(&c.table))?;
    Ok(hash)
}

/// Stages 2–6 for a commentary buffer under the fixture profile, resolved
/// against `table` (synthetic fixtures; the real build adds stage 1 first).
/// Writes nothing unless every check passes.
pub fn build_cmt_bytes(bytes: &[u8], table: &Table, out_dir: &Path) -> Result<String> {
    let model = commentary::parse(bytes, &commentary::fixture())?;
    commentary::resolve(&model, table)?;
    let meta = Meta::commentary(model.source_id, Vec::new(), None, Vec::new());
    db::publish(
        out_dir,
        commentary::MHCC_DEST,
        |conn| db::write_cmt(conn, &model, &meta),
        |conn, _hash| check::check_cmt(conn, &model, &meta, bytes, Some(table)),
        &mut |_| Ok(()),
    )
}

// ------------------------------------------------------------------ search index

/// The verse rows of the published `<out-dir>/kjv.sqlite`, which must have
/// SHA-256 `kjv_sha256` (the value its build or check just returned).
pub fn load_search_input(out_dir: &Path, kjv_sha256: &str) -> Result<SearchInput> {
    search::load(&out_dir.join(KJV_DEST), kjv_sha256)
}

/// Stages 4–6 for kjv-search.sqlite (DESIGN.md 18).
pub fn publish_search(
    out_dir: &Path,
    input: &SearchInput,
    hook: &mut dyn FnMut(Step) -> Result<()>,
) -> Result<String> {
    let meta = input.meta();
    db::publish(
        out_dir,
        SEARCH_DEST,
        |conn| search::write(conn, input, &meta),
        |conn, _hash| check::check_search(conn, input, &meta),
        hook,
    )
}

/// The `check` subcommand for a published kjv-search.sqlite, against the
/// published kjv.sqlite with SHA-256 `kjv_sha256`.
pub fn check_published_search(out_dir: &Path, kjv_sha256: &str) -> Result<String> {
    let input = load_search_input(out_dir, kjv_sha256)?;
    let (conn, hash) = open_published(out_dir, SEARCH_DEST)?;
    check::check_search(&conn, &input, &input.meta())?;
    Ok(hash)
}
