//! Strong's number keys: one canonical spelling for the three source spellings.
//!
//! The sources spell the same number differently (`docs/pipeline/kjv-osis-survey.md`
//! sections 6 and 9, `pipeline/DESIGN.md` section 6.2):
//!
//! | Source | Spelling | Example for Hebrew 430 / Greek 1 |
//! |---|---|---|
//! | KJV OSIS `lemma` token | `strong:` + letter + digits (Hebrew always has one leading `0`) | `strong:H0430`, `strong:G1` |
//! | `HebrewStrong.xml` `entry/@id`, `w/@src` | letter + digits, no padding | `H430` |
//! | `strongsgreek.xml` `entry/@strongs` | five digits, zero-padded, no letter | `00001` |
//! | `strongsgreek.xml` `strongsref/@strongs` | digits, language in `@language`; Greek unpadded, Hebrew zero-padded | `1`, `0175` |
//!
//! The canonical key is the letter plus the number without leading zeros
//! (`H430`, `G1`). Only keys are normalised; no displayed text passes through
//! this module.
//!
//! The grammar is strict and fails closed: ASCII digits only, one to five of
//! them, value at least 1, nothing before or after. Suffixes such as the
//! extended-Strong's `H1234a` are rejected, not truncated. Whether a number
//! exists in a lexicon is checked later, against the lexicon table.

use std::fmt;

/// The language half of a Strong's key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {
    Hebrew,
    Greek,
}

impl Lang {
    fn letter(self) -> char {
        match self {
            Lang::Hebrew => 'H',
            Lang::Greek => 'G',
        }
    }

    fn from_letter(byte: u8) -> Option<Lang> {
        match byte {
            b'H' => Some(Lang::Hebrew),
            b'G' => Some(Lang::Greek),
            _ => None,
        }
    }
}

/// A validated Strong's key. `Display` gives the canonical spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrongsKey {
    lang: Lang,
    number: u32,
}

/// Why a key was rejected. Carries no input text, so error messages can't
/// echo untrusted bytes; callers add the location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// The OSIS token doesn't start with `strong:`.
    MissingPrefix,
    /// The first character isn't `H` or `G`.
    BadLanguage,
    /// `strongsref/@language` isn't `HEBREW` or `GREEK`.
    BadLanguageName,
    /// No digits, more than five digits, or a non-digit character.
    BadDigits,
    /// The number is zero.
    Zero,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            KeyError::MissingPrefix => "Strong's token does not start with 'strong:'",
            KeyError::BadLanguage => "Strong's key does not start with 'H' or 'G'",
            KeyError::BadLanguageName => "Strong's language is not 'HEBREW' or 'GREEK'",
            KeyError::BadDigits => "Strong's number must be 1 to 5 ASCII digits",
            KeyError::Zero => "Strong's number must not be zero",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for KeyError {}

impl StrongsKey {
    /// The language of the key.
    pub fn lang(self) -> Lang {
        self.lang
    }

    /// The number, without padding.
    pub fn number(self) -> u32 {
        self.number
    }

    /// A key from a language and a digit string such as `"00001"` or `"427"`.
    pub fn from_digits(lang: Lang, digits: &str) -> Result<StrongsKey, KeyError> {
        let bytes = digits.as_bytes();
        if bytes.is_empty() || bytes.len() > 5 || !bytes.iter().all(u8::is_ascii_digit) {
            return Err(KeyError::BadDigits);
        }
        let number = bytes.iter().fold(0u32, |n, b| n * 10 + u32::from(b - b'0'));
        if number == 0 {
            return Err(KeyError::Zero);
        }
        Ok(StrongsKey { lang, number })
    }

    /// A letter-prefixed key such as `"H1"` (HebrewStrong.xml) or `"H0430"`.
    pub fn from_prefixed(key: &str) -> Result<StrongsKey, KeyError> {
        let lang = key
            .as_bytes()
            .first()
            .copied()
            .and_then(Lang::from_letter)
            .ok_or(KeyError::BadLanguage)?;
        // The letter is one ASCII byte, so slicing at 1 is on a char boundary.
        StrongsKey::from_digits(lang, &key[1..])
    }

    /// One whitespace-separated token of a KJV OSIS `lemma` attribute, for
    /// example `"strong:H07225"`. Other token kinds (`lemma.TR:…`) are the
    /// caller's business and must not be passed here.
    pub fn from_osis_lemma(token: &str) -> Result<StrongsKey, KeyError> {
        let rest = token
            .strip_prefix("strong:")
            .ok_or(KeyError::MissingPrefix)?;
        StrongsKey::from_prefixed(rest)
    }

    /// `strongsgreek.xml` `<strongsref language="…" strongs="…"/>`.
    pub fn from_strongsref(language: &str, digits: &str) -> Result<StrongsKey, KeyError> {
        let lang = match language {
            "HEBREW" => Lang::Hebrew,
            "GREEK" => Lang::Greek,
            _ => return Err(KeyError::BadLanguageName),
        };
        StrongsKey::from_digits(lang, digits)
    }
}

impl fmt::Display for StrongsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.lang.letter(), self.number)
    }
}

// These tests have not been run yet: `cargo test` needs the MSVC linker
// (M0-3). They do pass `cargo clippy --all-targets`.
#[cfg(test)]
mod tests {
    use super::*;

    fn key(lang: Lang, number: u32) -> StrongsKey {
        StrongsKey { lang, number }
    }

    #[test]
    fn kjv_osis_spellings_from_the_survey() {
        // Byte-exact tokens seen in kjv.osis.xml (Gen 1:1, John 3:2).
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:H07225"),
            Ok(key(Lang::Hebrew, 7225))
        );
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:H0430"),
            Ok(key(Lang::Hebrew, 430))
        );
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:H01"),
            Ok(key(Lang::Hebrew, 1))
        );
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:G3778"),
            Ok(key(Lang::Greek, 3778))
        );
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:G1"),
            Ok(key(Lang::Greek, 1))
        );
    }

    #[test]
    fn lexicon_spellings_meet_the_kjv_spelling() {
        let kjv = StrongsKey::from_osis_lemma("strong:H01").unwrap();
        let hebrew_lexicon = StrongsKey::from_prefixed("H1").unwrap();
        assert_eq!(kjv, hebrew_lexicon);

        let kjv = StrongsKey::from_osis_lemma("strong:G1").unwrap();
        let greek_entry = StrongsKey::from_digits(Lang::Greek, "00001").unwrap();
        let greek_ref = StrongsKey::from_strongsref("GREEK", "1").unwrap();
        assert_eq!(kjv, greek_entry);
        assert_eq!(kjv, greek_ref);
        assert_eq!(
            StrongsKey::from_strongsref("HEBREW", "8674"),
            Ok(key(Lang::Hebrew, 8674))
        );
        // strongsgreek.xml entry 00002 cites Hebrew as strongs="0175" (zero-padded).
        assert_eq!(
            StrongsKey::from_strongsref("HEBREW", "0175"),
            Ok(key(Lang::Hebrew, 175))
        );
    }

    #[test]
    fn canonical_spelling_has_no_padding() {
        assert_eq!(
            StrongsKey::from_osis_lemma("strong:H07225")
                .unwrap()
                .to_string(),
            "H7225"
        );
        assert_eq!(
            StrongsKey::from_digits(Lang::Greek, "05624")
                .unwrap()
                .to_string(),
            "G5624"
        );
    }

    #[test]
    fn rejects_everything_else() {
        use KeyError::*;
        let cases: &[(&str, KeyError)] = &[
            ("G5129", MissingPrefix), // the bare token in 2 Cor 8:10's lemma
            ("lemma.TR:ουτος", MissingPrefix),
            ("Strong:H1", MissingPrefix),
            ("strong:", BadLanguage),
            ("strong:h1", BadLanguage),
            ("strong:A1", BadLanguage),
            ("strong:H", BadDigits),
            ("strong:H1234a", BadDigits),
            ("strong:H123456", BadDigits),
            ("strong:H 1", BadDigits),
            ("strong:H+1", BadDigits),
            ("strong:H١", BadDigits), // Arabic-Indic digit one
            ("strong:H0", Zero),
            ("strong:G00000", Zero),
        ];
        for (input, want) in cases {
            assert_eq!(
                StrongsKey::from_osis_lemma(input),
                Err(*want),
                "input {input:?}"
            );
        }
        assert_eq!(
            StrongsKey::from_strongsref("Greek", "1"),
            Err(BadLanguageName)
        );
        assert_eq!(StrongsKey::from_prefixed(""), Err(BadLanguage));
        assert_eq!(StrongsKey::from_prefixed("ה1"), Err(BadLanguage));
    }
}
