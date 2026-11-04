//! One error type for the whole pipeline.
//!
//! Every failure names the rule that fired (`check`, for example `"sha256"`,
//! `"V4"`, `"R3"`, `"T7"`), so tests can assert *which* check caught a fault
//! (DESIGN.md T11). Messages never contain raw source bytes: excerpts go
//! through [`excerpt`], which escapes control and non-ASCII characters and
//! keeps at most 80 bytes (DESIGN.md 9.5).

use std::fmt;

/// One failed check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The rule that fired, e.g. `"sha256"`, `"xml"`, `"V2"`, `"R1"`, `"T7"`.
    pub check: &'static str,
    /// What went wrong. Never raw source text (see [`excerpt`]).
    pub msg: String,
    /// File name (not a full path) the error is about, if any.
    pub file: Option<String>,
    /// Byte offset in that file, if known.
    pub offset: Option<u64>,
    /// OSIS verse reference, if known.
    pub verse: Option<String>,
}

impl Error {
    pub fn new(check: &'static str, msg: impl Into<String>) -> Error {
        Error {
            check,
            msg: msg.into(),
            file: None,
            offset: None,
            verse: None,
        }
    }

    pub fn file(mut self, file: impl Into<String>) -> Error {
        self.file = Some(file.into());
        self
    }

    pub fn at(mut self, offset: u64) -> Error {
        self.offset = Some(offset);
        self
    }

    pub fn verse(mut self, verse: impl Into<String>) -> Error {
        self.verse = Some(verse.into());
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.check)?;
        if let Some(file) = &self.file {
            write!(f, " {file}")?;
        }
        if let Some(offset) = self.offset {
            write!(f, " at byte {offset}")?;
        }
        if let Some(verse) = &self.verse {
            write!(f, " ({verse})")?;
        }
        write!(f, ": {}", self.msg)
    }
}

impl std::error::Error for Error {}

/// The shared opener's errors keep their check names (`path`, `size`, `io`,
/// `sha256`, `header`, `content`, `open`) and file name.
impl From<revelation_content::OpenError> for Error {
    fn from(e: revelation_content::OpenError) -> Error {
        let msg = match &e {
            revelation_content::OpenError::HashMismatch {
                actual, expected, ..
            } => format!("database SHA-256 {actual} does not match the expected {expected}"),
            _ => e.reason(),
        };
        let err = Error::new(e.check(), msg);
        match e.file() {
            Some(f) => err.file(f),
            None => err,
        }
    }
}

/// A failed stage: at least one error, at most [`MAX_ERRORS`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure(pub Vec<Error>);

/// Validation collects errors and reports at most this many (DESIGN.md 2, stage 3).
pub const MAX_ERRORS: usize = 100;

impl Failure {
    /// The first error (there is always one).
    pub fn first(&self) -> &Error {
        &self.0[0]
    }

    /// Whether any error came from `check`.
    pub fn has(&self, check: &str) -> bool {
        self.0.iter().any(|e| e.check == check)
    }

    /// The distinct checks that fired, in first-seen order.
    pub fn checks(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        for e in &self.0 {
            if !out.contains(&e.check) {
                out.push(e.check);
            }
        }
        out
    }
}

impl From<Error> for Failure {
    fn from(e: Error) -> Failure {
        Failure(vec![e])
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Failure {}

pub type Result<T, E = Failure> = std::result::Result<T, E>;

/// Collects errors up to [`MAX_ERRORS`]; `finish` turns them into a result.
#[derive(Debug, Default)]
pub struct Collector {
    errors: Vec<Error>,
    dropped: usize,
}

impl Collector {
    pub fn push(&mut self, e: Error) {
        if self.errors.len() < MAX_ERRORS {
            self.errors.push(e);
        } else {
            self.dropped += 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_check(&self, check: &str) -> bool {
        self.errors.iter().any(|e| e.check == check)
    }

    pub fn finish(mut self) -> Result<()> {
        if self.errors.is_empty() {
            return Ok(());
        }
        if self.dropped > 0 {
            let more = Error::new(
                self.errors[0].check,
                format!("{} more errors not shown", self.dropped),
            );
            self.errors.push(more);
        }
        Err(Failure(self.errors))
    }
}

/// A short, escaped excerpt for error messages: at most 80 bytes of input,
/// printable ASCII kept, everything else shown as `\u{…}`.
pub fn excerpt(bytes: &[u8]) -> String {
    let cut = &bytes[..bytes.len().min(80)];
    let text = String::from_utf8_lossy(cut);
    let mut out = String::new();
    for c in text.chars() {
        if (' '..='~').contains(&c) && c != '\\' {
            out.push(c);
        } else {
            out.push_str(&format!("\\u{{{:x}}}", c as u32));
        }
    }
    if bytes.len() > 80 {
        out.push('…');
    }
    out
}

/// Shows text with non-obvious whitespace escaped (`\x20` for a trailing
/// space, `\t`, `\u{a0}`), for golden-file diffs (DESIGN.md 12.3).
pub fn show_ws(text: &str) -> String {
    let mut out = String::new();
    let n = text.chars().count();
    for (i, c) in text.chars().enumerate() {
        match c {
            ' ' if i + 1 == n || i == 0 => out.push_str("\\x20"),
            ' ' => out.push(' '),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            c if c.is_whitespace() || c.is_control() => {
                out.push_str(&format!("\\u{{{:x}}}", c as u32))
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_escapes_controls_and_caps_length() {
        assert_eq!(excerpt(b"a\x1b[31mb"), "a\\u{1b}[31mb");
        let long = vec![b'x'; 200];
        let e = excerpt(&long);
        assert!(e.starts_with(&"x".repeat(80)) && e.ends_with('…'));
    }

    #[test]
    fn show_ws_marks_trailing_space() {
        assert_eq!(show_ws("it. "), "it.\\x20");
        assert_eq!(show_ws("a\u{a0}b\tc"), "a\\u{a0}b\\tc");
    }
}
