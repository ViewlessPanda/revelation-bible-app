//! Dump, for the scholar, the red pieces of every golden-sample reference as
//! read back from a BUILT kjv.sqlite through `open_verified` (task D1/D6).
//!
//! Usage:
//!   cargo run -p revelation-pipeline --example golden_red_dump -- \
//!     <kjv.sqlite> <output-sha256.txt> <red-letter-golden-samples.md> <out.tsv>
//!
//! The expected SHA-256 comes from the committed output-sha256.txt, never
//! from beside the database. Words are cut from the stored text by the stored
//! offsets; nothing is typed.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use revelation_pipeline::golden;
use revelation_pipeline::open::open_verified;
use revelation_pipeline::sources::sha256_hex;

const EDGE_WORDS: usize = 4;

fn run(args: &[String]) -> Result<String, String> {
    let [db, expected, samples, out] = args else {
        return Err(
            "usage: golden_red_dump <kjv.sqlite> <output-sha256.txt> <golden-samples.md> <out.tsv>"
                .into(),
        );
    };
    let exp = fs::read_to_string(expected).map_err(|e| e.to_string())?;
    let want = exp
        .lines()
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| l.strip_suffix("  kjv.sqlite"))
        .ok_or("no kjv.sqlite line in the expected-hash file")?;
    let (conn, hash) = open_verified(Path::new(db), Some(want)).map_err(|e| e.to_string())?;
    let md = fs::read(samples).map_err(|e| e.to_string())?;
    let rows = golden::scholar_rows(&String::from_utf8(md.clone()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut o = Vec::new();
    o.push("# golden-db-red-dump.tsv: red pieces of every golden-sample reference, read back from the built".to_string());
    o.push("# kjv.sqlite through open_verified (sqlite3_deserialize of the hashed bytes). Machine-extracted:".into());
    o.push("# the words are cut from the stored verse text by the stored code-point offsets, never typed.".into());
    o.push(format!(
        "# database: kjv.sqlite sha256 {hash} (= pipeline/tests/expected/output-sha256.txt)"
    ));
    o.push(format!(
        "# references: docs/scholar/red-letter-golden-samples.md section 10, sha256 {}",
        sha256_hex(&md)
    ));
    o.push("# generator: pipeline/examples/golden_red_dump.rs. One line per piece; a verse without red has one".into());
    o.push(format!("# line with '-'. first_words/last_words: the first/last {EDGE_WORDS} space-separated words of the piece."));
    o.push("ref\tosis\tn\tstart_cp\tend_cp\tq_ord\tverse_len\tfirst_words\tlast_words".into());
    let mut pieces_total = 0;
    for r in &rows {
        let (id, osis) = golden::usfm_ref(&r.usfm).ok_or("bad ref")?;
        let text: String = conn
            .query_row("SELECT text FROM verse WHERE id = ?1", [id], |x| x.get(0))
            .map_err(|e| format!("{osis}: {e}"))?;
        let chars: Vec<char> = text.chars().collect();
        let ps = golden::pieces(&conn, id).map_err(|e| e.to_string())?;
        if ps.is_empty() {
            o.push(format!(
                "{}\t{osis}\t-\t-\t-\t-\t{}\t-\t-",
                r.usfm,
                chars.len()
            ));
        }
        for (n, s, e, q) in ps {
            let piece: String = chars[s..e].iter().collect();
            let words: Vec<&str> = piece.split(' ').filter(|w| !w.is_empty()).collect();
            let first = words[..words.len().min(EDGE_WORDS)].join(" ");
            let last = words[words.len().saturating_sub(EDGE_WORDS)..].join(" ");
            o.push(format!(
                "{}\t{osis}\t{n}\t{s}\t{e}\t{q}\t{}\t{first}\t{last}",
                r.usfm,
                chars.len()
            ));
            pieces_total += 1;
        }
    }
    fs::write(out, o.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(format!(
        "wrote {} references, {pieces_total} pieces; database sha256 {hash}",
        rows.len()
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
