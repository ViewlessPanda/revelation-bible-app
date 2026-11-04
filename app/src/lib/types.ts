// Shapes of the IPC payloads (src-tauri/src/commands.rs, content.rs, user.rs).

/** One presentation run of a verse: exact stored characters plus flags. */
export interface Run {
  text: string;
  red: boolean;
  /** red piece number (one `.wj` per piece) */
  rp?: number;
  add: boolean;
  nd: boolean;
  /** Strong's keys of the word this run belongs to */
  w?: string[];
  /** word number (one `.w` per word) */
  wn?: number;
}

export interface Verse {
  v: number;
  /** exact stored text; the runs concatenate to it */
  text: string;
  runs: Run[];
}

export interface Chapter {
  book: number;
  osis: string;
  chapter: number;
  verses: Verse[];
}

export interface Book {
  ord: number;
  osis: string;
  /** short name from the scholar's cited table (src-tauri/data/kjv-book-names.tsv) */
  name: string;
  /** name used in chapter headings ("Psalms" for now; an owner decision) */
  heading: string;
  /** `book.main_title`, the source's main title, exactly as stored */
  main_title: string | null;
  abbrevs: string[];
  chapters: number;
  /** highest verse number per chapter, index 0 = chapter 1 */
  verses: number[];
}

export interface TranslationInfo {
  abbrev: string;
  name: string;
  has_red_letters: boolean;
}

export interface AppInfo {
  content_error: string | null;
  user_error: string | null;
  /** W2A-U1: the refused file was made by a newer build; it is never set aside */
  user_too_new?: boolean;
  translations: TranslationInfo[];
  lexicon_available: boolean;
  /** why the lexicons can't be used (missing or failed their check), if so */
  lexicon_error: string | null;
  /** the phrase-search index opened and passed its checks */
  search_available?: boolean;
  /** why search can't be used (index missing, damaged or stale), if so */
  search_error?: string | null;
  /** the commentary (mhcc.sqlite) opened and passed its checks */
  commentary_available?: boolean;
  /** why the commentary can't be used (missing or failed its check), if so */
  commentary_error?: string | null;
}

/** A run of a search result verse (search.rs HitRun): no Strong's words, split at the emphasis. */
export interface HitRun {
  text: string;
  red: boolean;
  rp?: number;
  add: boolean;
  nd: boolean;
  /** inside a match: <mark class="hit">, presentation only */
  hit: boolean;
}

export interface SearchHit {
  /** book*1000000 + chapter*1000 + verse */
  id: number;
  book: number;
  osis: string;
  chapter: number;
  verse: number;
  /** "John 11:35" */
  reference: string;
  /** has the exact phrase (sorted first) */
  exact: boolean;
  /** the stored verse text from kjv.sqlite; the runs concatenate to it */
  text: string;
  runs: HitRun[];
  /** false: the match couldn't be mapped, shown without emphasis */
  emphasised: boolean;
}

export type SearchScope = "bible" | "old_testament" | "new_testament" | { book: number };

export type SearchResult =
  | { status: "ok"; total: number; exact_total: number; offset: number; phrase: boolean; hits: SearchHit[] }
  /** the index is missing, damaged or stale (the Bible still works) */
  | { status: "unavailable"; reason: string };

export interface Highlight {
  verse: number;
  color: HighlightColor;
}

export const COLORS = ["gold", "green", "blue", "rose", "violet"] as const;
export type HighlightColor = (typeof COLORS)[number];
export const COLOR_NAMES: Record<HighlightColor, string> = {
  gold: "Gold", green: "Green", blue: "Blue", rose: "Rose", violet: "Violet",
};

export interface Position {
  translation: string;
  book: string;
  chapter: number;
  anchor_verse: number | null;
  anchor_frac: number | null;
  current_verse: number | null;
}

export interface Settings {
  verse_numbers: boolean;
  red_letters: boolean;
  supplied_italics: boolean;
  text_step: number;
  /** Study panel open */
  panel_open: boolean;
  /** Commentary panel open (wave-2-layout.md 4) */
  commentary_open: boolean;
  /** The right column's width in rem, 18 to 36 (wave-2-layout.md 17.4) */
  right_width_rem: number;
  /** Commentary's share of the right column's height, 0 to 1 (17.4) */
  right_split: number;
  /** The highlighter pen's last colour (highlights-pen.md 3.1) */
  pen_color: HighlightColor;
  /** Notes panel open (wave-2-layout.md 4) */
  notes_open: boolean;
  /** The notes panel's width in rem, 16 to 32 */
  notes_width_rem: number;
}

export interface SavedState {
  position: Position | null;
  settings: Settings;
}

export interface LexAttr { name: string; value: string; ref_key?: string }
export interface LexNode { n: number; parent: number | null; name: string | null; text: string | null; attrs: LexAttr[] }
export interface LexEntry { key: string; lexicon: string; nodes: LexNode[] }

export type StrongsResult =
  | { status: "ok"; entry: LexEntry }
  | { status: "not_found"; key: string }
  /** content.sqlite is missing or failed its check (the Bible still works) */
  | { status: "not_available"; reason: string };

/** What the study panel shows. */
export type Lookup =
  | { kind: "empty" }
  /** `title`: what was clicked ("Lexicon for “love”", "From G26") */
  | { kind: "loading"; title: string; keys: string[] }
  /** results[i] is the lookup of keys[i] */
  | { kind: "done"; title: string; keys: string[]; results: StrongsResult[] }
  | { kind: "error"; message: string };
