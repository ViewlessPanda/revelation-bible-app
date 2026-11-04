//! Parser for `pipeline/tests/expected/output-sha256.txt` (pipeline/DESIGN.md
//! 10.2), the committed expected SHA-256 of each bundled content file.
//!
//! Used by the app's `build.rs` (which compiles the values into the app, with
//! this crate's default features off) and by the pipeline's T16 tests. Plain
//! std only.
//!
//! Format, tolerant of what the pipeline writes and nothing more:
//! - blank lines and lines starting with `#` are ignored (header comments);
//! - every other line is `<64 lower-case hex digits><space><space or '*'><file name>`
//!   (the `sha256sum` text and binary forms);
//! - a trailing `\r` is ignored (CRLF checkouts);
//! - file names are plain names (`[A-Za-z0-9._-]`, no path), each at most once.
//!
//! Anything else is an error, so a damaged file fails the build instead of
//! compiling in a wrong value.

/// `(file name, sha256)` pairs in file order.
pub fn parse(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (i, raw) in text.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let n = i + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let bytes = line.as_bytes();
        if bytes.len() < 67 {
            return Err(format!("line {n}: too short for '<sha256>  <file>'"));
        }
        let (Some(hash), Some(rest)) = (line.get(..64), line.get(64..)) else {
            return Err(format!(
                "line {n}: the first 64 characters are not lower-case hex"
            ));
        };
        if !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(format!(
                "line {n}: the first 64 characters are not lower-case hex"
            ));
        }
        let name = rest
            .strip_prefix("  ")
            .or_else(|| rest.strip_prefix(" *"))
            .ok_or_else(|| format!("line {n}: expected two spaces (or ' *') after the hash"))?;
        if name.is_empty()
            || name == "."
            || name == ".."
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
        {
            return Err(format!("line {n}: file name is not a plain file name"));
        }
        if out.iter().any(|(f, _)| f == name) {
            return Err(format!("line {n}: {name} is listed twice"));
        }
        out.push((name.to_string(), hash.to_string()));
    }
    if out.is_empty() {
        return Err("no '<sha256>  <file>' lines".to_string());
    }
    Ok(out)
}

/// The expected hash of `file`, or an error naming what is missing.
pub fn lookup<'a>(entries: &'a [(String, String)], file: &str) -> Result<&'a str, String> {
    entries
        .iter()
        .find(|(f, _)| f == file)
        .map(|(_, h)| h.as_str())
        .ok_or_else(|| format!("no expected SHA-256 for {file}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: &str = "92cae5e0a198cfea5952249b30488c25a0006c9faa7c59726242d93006264e27";

    #[test]
    fn accepts_comments_crlf_and_both_sha256sum_forms() {
        let text = format!(
            "# header\r\n# more: with  two spaces\r\n\r\n{H}  kjv.sqlite\r\n{} *content.sqlite\n",
            "0".repeat(64)
        );
        let e = parse(&text).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(lookup(&e, "kjv.sqlite").unwrap(), H);
        assert_eq!(lookup(&e, "content.sqlite").unwrap(), "0".repeat(64));
        assert!(lookup(&e, "other.sqlite").is_err());
    }

    #[test]
    fn rejects_malformed_lines() {
        let bad = [
            String::new(),
            "# only comments\n".to_string(),
            format!("{}  kjv.sqlite", H.to_uppercase()),
            format!("{}  kjv.sqlite", &H[..63]),
            format!("{H}g  kjv.sqlite"),
            format!("{H} kjv.sqlite"),
            format!("{H}\tkjv.sqlite"),
            format!("{H}  "),
            format!("{H}  ../kjv.sqlite"),
            format!("{H}  out/kjv.sqlite"),
            format!("{H}  kjv.sqlite "),
            format!("{H}  kjv.sqlite\n{H}  kjv.sqlite"),
            format!(" {H}  kjv.sqlite"),
            format!("\u{e9}{}  kjv.sqlite", &H[..62]),
            format!("{}\u{e9}  kjv.sqlite", &H[..63]), // byte 64 inside a character
            "kjv.sqlite".to_string(),
        ];
        for b in &bad {
            assert!(parse(b).is_err(), "accepted: {b:?}");
        }
    }

    /// The committed file itself parses and names both content files (the
    /// app build already depends on this; the test makes a failure readable).
    #[test]
    fn committed_expected_file_parses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../pipeline/tests/expected/output-sha256.txt");
        let e = parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        for f in ["kjv.sqlite", "content.sqlite"] {
            assert_eq!(lookup(&e, f).unwrap().len(), 64, "{f}");
        }
    }
}
