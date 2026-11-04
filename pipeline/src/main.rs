//! Revelation content pipeline CLI (DESIGN.md 2). std only: no argument
//! parser crate, no network, never executes anything.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use revelation_pipeline::commentary::MHCC_DEST;
use revelation_pipeline::db::{self, LEX_DEST, Step};
use revelation_pipeline::error::Result;
use revelation_pipeline::run::{self, KJV_DEST};
use revelation_pipeline::search::SEARCH_DEST;
use revelation_pipeline::validate::red_totals;

const USAGE: &str = "\
revelation-pipeline: builds kjv.sqlite, kjv-search.sqlite (its search
index), content.sqlite (lexicons) and mhcc.sqlite (Matthew Henry's Concise
Commentary) from pinned, checksummed sources.

Usage:
    revelation-pipeline verify <raw-dir>
    revelation-pipeline build  <raw-dir> <out-dir>
    revelation-pipeline check  <raw-dir> <out-dir>
    revelation-pipeline [help | -h | --help]

<raw-dir> is sources/raw (one directory per source ID). Checksums, SOURCES.md
and the versification overrides are read from this repository.

verify  stages 1-3 for every source: SHA-256, parse, validate, every KJV
        Strong's key resolves to a lexicon entry, and every commentary anchor
        and cross-reference is a KJV verse (writes nothing).
build   stages 1-6: also writes <out-dir>/kjv.sqlite, kjv-search.sqlite
        (built from the published kjv.sqlite), content.sqlite and
        mhcc.sqlite, each with its .sha256, each published only after its
        temp file passed every check. Prints the SHA-256s and the build
        provenance.
check   re-checks the published kjv.sqlite, kjv-search.sqlite,
        content.sqlite and mhcc.sqlite against the sources.

Environment: REVELATION_PIPELINE_COMMIT (40 hex digits, optional -dirty;
otherwise 'unknown-dirty'), printed as build provenance and never stored in
a database; REVELATION_RELEASE=1 makes build refuse a -dirty commit.
";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Verify(PathBuf),
    Build(PathBuf, PathBuf),
    Check(PathBuf, PathBuf),
    Bad(String),
}

fn parse(mut args: impl Iterator<Item = OsString>) -> Command {
    let cmd = args.next();
    let rest: Vec<OsString> = args.collect();
    let Some(cmd) = cmd else {
        return Command::Help;
    };
    let path = |i: usize| PathBuf::from(&rest[i]);
    match (cmd.to_str(), rest.len()) {
        (Some("help" | "-h" | "--help"), 0) => Command::Help,
        (Some("verify"), 1) => Command::Verify(path(0)),
        (Some("build"), 2) => Command::Build(path(0), path(1)),
        (Some("check"), 2) => Command::Check(path(0), path(1)),
        _ => Command::Bad(format!(
            "unknown command or wrong arguments: '{}'",
            cmd.to_string_lossy()
        )),
    }
}

/// Build provenance (DESIGN.md 10.1): reported, never stored in a database.
fn commit() -> Result<String> {
    let release = std::env::var_os("REVELATION_RELEASE").is_some_and(|v| v == "1");
    Ok(db::build_commit(
        std::env::var("REVELATION_PIPELINE_COMMIT").ok().as_deref(),
        release,
    )?)
}

/// The provenance lines printed by `build`: what produced the files, which
/// the files themselves deliberately don't record.
fn provenance(commit: &str) {
    let lock = std::fs::read(revelation_pipeline::repo_dir().join("Cargo.lock"))
        .map(|b| revelation_pipeline::sources::sha256_hex(&b))
        .unwrap_or_else(|_| "unreadable".into());
    println!("provenance: pipeline_commit {commit}");
    println!("provenance: Cargo.lock sha256 {lock}");
    println!(
        "provenance: revelation-pipeline {} ({}), not stored in any database",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
}

fn summary(prep: &run::Prepared) {
    let m = &prep.model;
    let t = red_totals(m);
    let (q, p, cp) = t
        .values()
        .fold((0, 0, 0), |a, x| (a.0 + x.0, a.1 + x.1, a.2 + x.2));
    println!(
        "{} books, {} chapters, {} verses; {q} q, {p} red pieces, {cp} red code points",
        m.books.len(),
        prep.table.chapters(),
        m.verses.len()
    );
    for a in &prep.profile.anomalies {
        println!(
            "anomaly list: {} skip token '{}' ({})",
            a.verse, a.skip_token, a.why
        );
    }
}

fn lex_summary(lex: &run::PreparedLex) {
    for m in lex.models() {
        let ph = m.entries.iter().filter(|e| e.placeholder).count();
        println!(
            "{}: {} entries ({ph} \"Not Used\" placeholders)",
            m.source_id,
            m.entries.len()
        );
    }
}

fn cmt_summary(c: &run::PreparedCmt) {
    let m = &c.model;
    let in_anchor = m.blocks.iter().filter(|b| b.anchor.is_some()).count();
    let refs: usize = m.anchors.iter().map(|a| a.verses.len()).sum();
    println!(
        "{}: {} blocks ({in_anchor} in verse anchors), {} anchors listing {refs} KJV verses; every anchor and reference resolves",
        m.source_id,
        m.blocks.len(),
        m.anchors.len()
    );
}

/// Stages 1-3 for every source, including T13 (every KJV key resolves) and
/// the commentary's KJV resolution, before anything is written.
fn prepare_all(
    raw: &std::path::Path,
) -> Result<(run::Prepared, run::PreparedLex, run::PreparedCmt)> {
    let prep = run::prepare_kjv(raw)?;
    summary(&prep);
    let lex = run::prepare_lexicons(raw)?;
    lex_summary(&lex);
    run::resolve_kjv(&prep.model, &lex.models())?;
    println!("T13: every KJV Strong's key resolves to a used lexicon entry");
    let cmt = run::prepare_mhcc(raw, &prep)?;
    cmt_summary(&cmt);
    Ok((prep, lex, cmt))
}

fn go(cmd: Command) -> Result<()> {
    match cmd {
        Command::Verify(raw) => {
            prepare_all(&raw)?;
            println!("verify: ok");
        }
        Command::Build(raw, out) => {
            let c = commit()?;
            let (prep, lex, cmt) = prepare_all(&raw)?;
            let meta = prep.meta();
            let hash = run::publish_model(
                &out,
                KJV_DEST,
                &prep.model,
                &prep.profile,
                Some(&prep.table),
                &meta,
                &mut |_: Step| Ok(()),
            )?;
            println!("published {KJV_DEST} sha256 {hash}");
            let input = run::load_search_input(&out, &hash)?;
            let hash = run::publish_search(&out, &input, &mut |_: Step| Ok(()))?;
            println!("published {SEARCH_DEST} sha256 {hash}");
            let hash = run::publish_lexicons(&out, &lex, &mut |_: Step| Ok(()))?;
            println!("published {LEX_DEST} sha256 {hash}");
            let hash = run::publish_mhcc(&out, &cmt, &mut |_: Step| Ok(()))?;
            println!("published {MHCC_DEST} sha256 {hash}");
            provenance(&c);
        }
        Command::Check(raw, out) => {
            let (prep, lex, cmt) = prepare_all(&raw)?;
            let hash = run::check_published(&prep, &out)?;
            println!("check: ok, {KJV_DEST} sha256 {hash}");
            let hash = run::check_published_search(&out, &hash)?;
            println!("check: ok, {SEARCH_DEST} sha256 {hash}");
            let hash = run::check_published_lex(&lex, &out)?;
            println!("check: ok, {LEX_DEST} sha256 {hash}");
            let hash = run::check_published_mhcc(&cmt, &out)?;
            println!("check: ok, {MHCC_DEST} sha256 {hash}");
        }
        Command::Help | Command::Bad(_) => unreachable!(),
    }
    Ok(())
}

fn main() -> ExitCode {
    match parse(std::env::args_os().skip(1)) {
        Command::Help => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::Bad(msg) => {
            eprintln!("error: {msg}\n");
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
        cmd => match go(cmd) {
            Ok(()) => ExitCode::SUCCESS,
            Err(f) => {
                eprintln!("FAILED (nothing published):\n{f}");
                ExitCode::FAILURE
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = OsString> {
        list.iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn no_arguments_prints_help() {
        assert_eq!(parse(args(&[])), Command::Help);
    }

    #[test]
    fn help_flags_print_help() {
        for flag in ["help", "-h", "--help"] {
            assert_eq!(parse(args(&[flag])), Command::Help);
        }
    }

    #[test]
    fn commands_take_exact_arguments() {
        assert_eq!(parse(args(&["verify", "r"])), Command::Verify("r".into()));
        assert_eq!(
            parse(args(&["build", "r", "o"])),
            Command::Build("r".into(), "o".into())
        );
        assert_eq!(
            parse(args(&["check", "r", "o"])),
            Command::Check("r".into(), "o".into())
        );
        for bad in [
            &["build", "r"][..],
            &["verify"],
            &["frobnicate"],
            &["check", "a", "b", "c"],
        ] {
            assert!(matches!(parse(args(bad)), Command::Bad(_)), "{bad:?}");
        }
    }
}
