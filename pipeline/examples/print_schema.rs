//! Prints `SELECT type, name, sql FROM sqlite_schema` of a built kjv.sqlite,
//! read through `open_verified` with the committed expected hash.
//!
//! Usage: cargo run -p revelation-pipeline --example print_schema -- <kjv.sqlite> <output-sha256.txt>

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exp = std::fs::read_to_string(&args[1]).expect("expected-hash file");
    let want = exp
        .lines()
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| l.strip_suffix("  kjv.sqlite"))
        .expect("kjv.sqlite line");
    let (c, hash) = revelation_pipeline::open::open_verified(Path::new(&args[0]), Some(want))
        .unwrap_or_else(|e| panic!("{e}"));
    println!("-- kjv.sqlite sha256 {hash}");
    for p in [
        "page_size",
        "encoding",
        "auto_vacuum",
        "application_id",
        "user_version",
        "journal_mode",
    ] {
        let v: String = c
            .query_row(&format!("PRAGMA {p}"), [], |r| {
                r.get_ref(0).map(|v| format!("{v:?}"))
            })
            .unwrap_or_else(|e| format!("? ({e})"));
        println!("-- PRAGMA {p} = {v}");
    }
    let mut st = c
        .prepare("SELECT type, name, sql FROM sqlite_schema ORDER BY type DESC, name")
        .unwrap();
    let rows = st
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .unwrap();
    for r in rows {
        let (t, n, sql) = r.unwrap();
        match sql {
            Some(s) => println!("{s};\n"),
            None => println!("-- {t} {n} (automatic, no SQL)\n"),
        }
    }
}
