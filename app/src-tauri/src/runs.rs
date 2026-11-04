//! Turns a stored verse and its code-point ranges into ready-made runs for the
//! WebView (pipeline/DESIGN.md 5.1): the UI never does offset arithmetic.
//!
//! Invariants (tested):
//! - concatenating `run.text` gives the stored verse text exactly;
//! - every run lies wholly inside one red piece or wholly outside all of them,
//!   and the runs carrying piece `n` cover exactly that piece's stored range.
//!
//! Nothing here changes a character. Ranges that break the schema rules fail
//! closed with an error instead of being clamped or repaired.

use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt;

/// One presentation run: a substring of the stored text plus the presentation
/// that applies to all of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Run {
    /// Exact stored characters.
    pub text: String,
    /// Inside a words-of-Jesus piece (stored `red_piece` range).
    pub red: bool,
    /// Which red piece (`red_piece.n`), so the UI makes one `.wj` per piece.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rp: Option<u32>,
    /// Supplied words (`mark.kind = 'added'`): italics.
    pub add: bool,
    /// Divine name (`mark.kind = 'divine'`): small capitals.
    pub nd: bool,
    /// Strong's keys of the word this run belongs to (canonical, in order).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<Vec<String>>,
    /// Which word (`word.n`), so the UI makes one `.w` per word.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wn: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    Added,
    Divine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedPiece {
    pub n: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub kind: MarkKind,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub n: u32,
    pub start: u32,
    pub end: u32,
    pub keys: Vec<String>,
}

/// A range that the schema rules don't allow. Carries numbers only, never text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeError(pub String);

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid stored range: {}", self.0)
    }
}

/// Build the runs for one verse.
pub fn build_runs(
    text: &str,
    red: &[RedPiece],
    marks: &[Mark],
    words: &[Word],
) -> Result<Vec<Run>, RangeError> {
    // Byte offset of every code-point boundary 0..=len.
    let mut byte_at: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    byte_at.push(text.len());
    let len = (byte_at.len() - 1) as u32;
    if len == 0 {
        return Err(RangeError("empty verse text".into()));
    }

    let mut cuts: BTreeSet<u32> = BTreeSet::new();
    cuts.insert(0);
    cuts.insert(len);

    // Red pieces: non-empty, inside the verse, ordered by n and position,
    // non-overlapping (DESIGN.md 5.3 R1, R3).
    let mut sorted_red = red.to_vec();
    sorted_red.sort_by_key(|p| p.n);
    for (i, p) in sorted_red.iter().enumerate() {
        if !(p.start < p.end && p.end <= len) {
            return Err(RangeError(format!(
                "red piece {} [{}, {}) in verse of length {len}",
                p.n, p.start, p.end
            )));
        }
        if p.n as usize != i {
            return Err(RangeError(format!(
                "red pieces not numbered 0..: found n = {}",
                p.n
            )));
        }
        if i > 0 && sorted_red[i - 1].end >= p.start {
            return Err(RangeError(format!(
                "red pieces {} and {} overlap or touch",
                i - 1,
                i
            )));
        }
        cuts.insert(p.start);
        cuts.insert(p.end);
    }
    for m in marks {
        if !(m.start < m.end && m.end <= len) {
            return Err(RangeError(format!(
                "mark [{}, {}) in verse of length {len}",
                m.start, m.end
            )));
        }
        // P4: a mark is inside one red piece or disjoint from all of them.
        for p in &sorted_red {
            let disjoint = m.end <= p.start || m.start >= p.end;
            let inside = m.start >= p.start && m.end <= p.end;
            if !(disjoint || inside) {
                return Err(RangeError(format!(
                    "mark [{}, {}) crosses red piece {}",
                    m.start, m.end, p.n
                )));
            }
        }
        cuts.insert(m.start);
        cuts.insert(m.end);
    }
    let mut sorted_words: Vec<&Word> = words.iter().filter(|w| w.start < w.end).collect();
    sorted_words.sort_by_key(|w| w.start);
    for (i, w) in sorted_words.iter().enumerate() {
        if w.end > len {
            return Err(RangeError(format!(
                "word {} [{}, {}) in verse of length {len}",
                w.n, w.start, w.end
            )));
        }
        if i > 0 && sorted_words[i - 1].end > w.start {
            return Err(RangeError(format!(
                "words {} and {} overlap",
                sorted_words[i - 1].n,
                w.n
            )));
        }
        cuts.insert(w.start);
        cuts.insert(w.end);
    }
    for w in words {
        if w.start > w.end || w.end > len {
            return Err(RangeError(format!(
                "word {} [{}, {}) in verse of length {len}",
                w.n, w.start, w.end
            )));
        }
    }

    let cuts: Vec<u32> = cuts.into_iter().collect();
    let mut runs = Vec::with_capacity(cuts.len());
    for pair in cuts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let piece = sorted_red.iter().find(|p| p.start <= a && b <= p.end);
        let add = marks
            .iter()
            .any(|m| m.kind == MarkKind::Added && m.start <= a && b <= m.end);
        let nd = marks
            .iter()
            .any(|m| m.kind == MarkKind::Divine && m.start <= a && b <= m.end);
        let word = sorted_words.iter().find(|w| w.start <= a && b <= w.end);
        runs.push(Run {
            text: text[byte_at[a as usize]..byte_at[b as usize]].to_string(),
            red: piece.is_some(),
            rp: piece.map(|p| p.n),
            add,
            nd,
            w: word.map(|w| w.keys.clone()),
            wn: word.map(|w| w.n),
        });
    }
    Ok(runs)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn cp_len(s: &str) -> u32 {
        s.chars().count() as u32
    }

    /// The red pieces as reassembled from runs: (n, start, end) in code points.
    pub(crate) fn red_from_runs(runs: &[Run]) -> Vec<(u32, u32, u32)> {
        let mut out: Vec<(u32, u32, u32)> = Vec::new();
        let mut pos = 0u32;
        for r in runs {
            let l = cp_len(&r.text);
            if let Some(n) = r.rp {
                assert!(r.red);
                match out.last_mut() {
                    Some(last) if last.0 == n && last.2 == pos => last.2 = pos + l,
                    _ => out.push((n, pos, pos + l)),
                }
            } else {
                assert!(!r.red);
            }
            pos += l;
        }
        out
    }

    #[test]
    fn plain_verse_is_one_run() {
        let runs = build_runs("alpha beta", &[], &[], &[]).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "alpha beta");
        assert!(!runs[0].red && !runs[0].add && !runs[0].nd && runs[0].w.is_none());
    }

    #[test]
    fn astral_and_combining_marks_use_code_points() {
        // U+1D400 is one code point (two UTF-16 units, four UTF-8 bytes);
        // "e\u{301}" is two code points. Offsets are code points.
        let text = "a \u{1D400}e\u{301}x b";
        assert_eq!(cp_len(text), 8);
        let red = [RedPiece {
            n: 0,
            start: 2,
            end: 6,
        }]; // "\u{1D400}e\u{301}x"
        let words = [Word {
            n: 0,
            start: 2,
            end: 5,
            keys: vec!["G1".into()],
        }];
        let runs = build_runs(text, &red, &[], &words).unwrap();
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, text);
        assert_eq!(red_from_runs(&runs), vec![(0, 2, 6)]);
        let red_text: String = runs
            .iter()
            .filter(|r| r.red)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(red_text, "\u{1D400}e\u{301}x");
        let word_text: String = runs
            .iter()
            .filter(|r| r.wn == Some(0))
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(word_text, "\u{1D400}e\u{301}");
    }

    #[test]
    fn nested_marks_and_words_split_runs_without_changing_text() {
        let text = "one two Lord three";
        let red = [RedPiece {
            n: 0,
            start: 4,
            end: 18,
        }];
        let marks = [
            Mark {
                kind: MarkKind::Added,
                start: 4,
                end: 7,
            },
            Mark {
                kind: MarkKind::Divine,
                start: 8,
                end: 12,
            },
        ];
        let words = [
            Word {
                n: 0,
                start: 0,
                end: 3,
                keys: vec!["H1".into()],
            },
            Word {
                n: 1,
                start: 8,
                end: 12,
                keys: vec!["H2".into(), "H3".into()],
            },
            Word {
                n: 2,
                start: 13,
                end: 13,
                keys: vec!["H4".into()],
            }, // empty word: no run
        ];
        let runs = build_runs(text, &red, &marks, &words).unwrap();
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, text);
        assert_eq!(red_from_runs(&runs), vec![(0, 4, 18)]);
        let lord = runs.iter().find(|r| r.text == "Lord").unwrap();
        assert!(lord.nd && lord.red && !lord.add);
        assert_eq!(
            lord.w.as_deref(),
            Some(&["H2".to_string(), "H3".to_string()][..])
        );
        assert!(runs.iter().all(|r| r.wn != Some(2)));
    }

    #[test]
    fn whole_verse_and_edge_pieces() {
        let runs = build_runs(
            "abc",
            &[RedPiece {
                n: 0,
                start: 0,
                end: 3,
            }],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(red_from_runs(&runs), vec![(0, 0, 3)]);
        let runs = build_runs(
            "ab cd ef",
            &[
                RedPiece {
                    n: 0,
                    start: 0,
                    end: 2,
                },
                RedPiece {
                    n: 1,
                    start: 6,
                    end: 8,
                },
            ],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(red_from_runs(&runs), vec![(0, 0, 2), (1, 6, 8)]);
    }

    #[test]
    fn bad_ranges_fail_closed() {
        let t = "abcd";
        assert!(
            build_runs(
                t,
                &[RedPiece {
                    n: 0,
                    start: 2,
                    end: 2
                }],
                &[],
                &[]
            )
            .is_err()
        ); // empty
        assert!(
            build_runs(
                t,
                &[RedPiece {
                    n: 0,
                    start: 2,
                    end: 5
                }],
                &[],
                &[]
            )
            .is_err()
        ); // past end
        assert!(
            build_runs(
                t,
                &[
                    RedPiece {
                        n: 0,
                        start: 0,
                        end: 2
                    },
                    RedPiece {
                        n: 1,
                        start: 2,
                        end: 4
                    }
                ],
                &[],
                &[]
            )
            .is_err()
        ); // touching
        assert!(
            build_runs(
                t,
                &[RedPiece {
                    n: 1,
                    start: 0,
                    end: 2
                }],
                &[],
                &[]
            )
            .is_err()
        ); // numbering
        assert!(
            build_runs(
                t,
                &[RedPiece {
                    n: 0,
                    start: 0,
                    end: 2
                }],
                &[Mark {
                    kind: MarkKind::Added,
                    start: 1,
                    end: 3
                }],
                &[]
            )
            .is_err()
        ); // P4
        assert!(
            build_runs(
                t,
                &[],
                &[Mark {
                    kind: MarkKind::Divine,
                    start: 3,
                    end: 3
                }],
                &[]
            )
            .is_err()
        );
        assert!(
            build_runs(
                t,
                &[],
                &[],
                &[
                    Word {
                        n: 0,
                        start: 0,
                        end: 3,
                        keys: vec![]
                    },
                    Word {
                        n: 1,
                        start: 2,
                        end: 4,
                        keys: vec![]
                    }
                ]
            )
            .is_err()
        );
        assert!(
            build_runs(
                t,
                &[],
                &[],
                &[Word {
                    n: 0,
                    start: 5,
                    end: 5,
                    keys: vec![]
                }]
            )
            .is_err()
        );
        assert!(build_runs("", &[], &[], &[]).is_err());
    }
}
