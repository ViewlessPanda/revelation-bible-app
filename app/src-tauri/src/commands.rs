//! The small typed IPC surface. Every argument is validated here or in the
//! module it reaches; errors cross the boundary as short strings that carry no
//! content.

use crate::commentary::{self, Commentary, CommentaryResult};
use crate::content::{self, Bible, BookInfo, ChapterOut, LexEntry, Lexicon, QueryError};
use crate::expected;
use crate::marks::{MarkIn, MarkOut, Piece};
use crate::search::{self, SearchIndex, SearchResult, SearchScope};
use crate::user::{self, Bookmark, Highlight, SavedState, UserDb, UserError};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct Content {
    pub bibles: Vec<Bible>,
    /// The lexicons (`content.sqlite`), opened like the Bible through
    /// `open_verified` with their own compiled-in hash. Reading Scripture never
    /// depends on them: if the file is missing or fails its check, this holds
    /// the reason, the Bible still opens, and Strong's lookups answer
    /// "unavailable" (the panel asks for a repair).
    pub lexicon: Result<Lexicon, String>,
    /// The phrase-search index (`kjv-search.sqlite`), opened the same way
    /// against its own compiled-in hash and required to be built from the
    /// expected `kjv.sqlite`. Like the lexicons, reading never depends on it:
    /// if it can't be used this holds the reason and search answers
    /// "unavailable" (wave-2-layout 6.6).
    pub search: Result<SearchIndex, String>,
    /// The commentary (`mhcc.sqlite`), opened the same way against its own
    /// compiled-in hash and identity. Reading never depends on it: if it
    /// can't be used this holds the reason and the Commentary panel says
    /// "unavailable" (wave-2-layout 7).
    pub commentary: Result<Commentary, String>,
}

pub struct AppState {
    pub content: Mutex<Result<Content, String>>,
    pub user: Mutex<Result<UserDb, Refusal>>,
}

/// Why the user database couldn't be opened at startup. `too_new` (W2A-U1):
/// the file was made by a newer build, so it must not be set aside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub msg: String,
    pub too_new: bool,
}
impl From<String> for Refusal {
    fn from(msg: String) -> Self {
        Refusal {
            msg,
            too_new: false,
        }
    }
}
impl From<&str> for Refusal {
    fn from(msg: &str) -> Self {
        msg.to_string().into()
    }
}
impl From<UserError> for Refusal {
    fn from(e: UserError) -> Self {
        let too_new = matches!(e, UserError::TooNew(_));
        Refusal {
            msg: e.to_string(),
            too_new,
        }
    }
}
impl From<Refusal> for String {
    fn from(r: Refusal) -> Self {
        r.msg
    }
}

/// The KJV file name inside the content folder.
pub const KJV_FILE: &str = "kjv.sqlite";
/// The lexicon file name inside the content folder.
pub const LEX_FILE: &str = "content.sqlite";

/// Where content files are looked for (KISS, task D4a):
/// - debug builds (`cargo test`, `tauri dev`): the pipeline's output folder in
///   this repository, `pipeline/out/`;
/// - release builds: a `content` folder next to the exe
///   (`scripts/copy-content.mjs` puts a hash-checked copy there).
///
/// Either way the file is opened only through `open_verified` with the hash
/// compiled in from `pipeline/tests/expected/output-sha256.txt`.
pub fn content_dir() -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pipeline/out"))
    }
    #[cfg(not(debug_assertions))]
    {
        let exe = std::env::current_exe()
            .map_err(|e| format!("cannot find the program folder: {}", e.kind()))?;
        let dir = exe
            .parent()
            .ok_or("cannot find the program folder")?
            .join("content");
        Ok(dir)
    }
}

/// Open the installed content: the KJV and the lexicons, each verified
/// against its compiled-in hash.
pub fn load_content() -> Result<Content, String> {
    load_content_from(
        &content_dir()?,
        expected::KJV_SQLITE,
        expected::CONTENT_SQLITE,
    )
}

/// Open `<dir>/kjv.sqlite` against `kjv_sha256` and `<dir>/content.sqlite`
/// against `lex_sha256` (and the search index against its compiled-in hash). A KJV failure (missing, damaged, replaced, or rebuilt
/// without rebuilding the app) is an error: the UI then shows the
/// repair/reinstall message and no text. A lexicon failure only disables
/// Strong's lookups (`Content::lexicon`).
pub fn load_content_from(
    dir: &Path,
    kjv_sha256: &str,
    lex_sha256: &str,
) -> Result<Content, String> {
    load_content_with(dir, kjv_sha256, lex_sha256, expected::KJV_SEARCH_SQLITE)
}

/// [`load_content_from`] with the search index's expected hash too.
pub fn load_content_with(
    dir: &Path,
    kjv_sha256: &str,
    lex_sha256: &str,
    search_sha256: &str,
) -> Result<Content, String> {
    let kjv = Bible::open(&dir.join(KJV_FILE), kjv_sha256).map_err(|e| e.to_string())?;
    let lexicon = Lexicon::open(&dir.join(LEX_FILE), lex_sha256).map_err(|e| e.to_string());
    let search = SearchIndex::open(&dir.join(search::SEARCH_FILE), search_sha256, kjv_sha256);
    let commentary = Commentary::open(&dir.join(commentary::MHCC_FILE), expected::MHCC_SQLITE);
    Ok(Content {
        bibles: vec![kjv],
        lexicon,
        search,
        commentary,
    })
}

/// The user database's file name inside the app data folder.
pub const USER_FILE: &str = "user.sqlite";

pub fn open_user(dir: &Path) -> Result<UserDb, Refusal> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("could not create the app data folder: {}", e.kind()))?;
    Ok(UserDb::open(&dir.join(USER_FILE))?)
}

/// D5-F5: after the user database was refused, keep that file (renamed, never
/// deleted) and start a new one. Refuses while the current file works.
pub fn start_new_user(state: &AppState, dir: &Path) -> Result<String, String> {
    let mut guard = lock(&state.user);
    match &*guard {
        Ok(_) => return Err("your data file is working; nothing was changed".into()),
        // W2A-U1: a newer build's file stays where that build finds it
        Err(r) if r.too_new => {
            return Err("your data file was made by a newer version of Revelation; open it with that version. Nothing was changed".into())
        }
        Err(_) => {}
    }
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("could not create the app data folder: {}", e.kind()))?;
    let (db, kept) =
        UserDb::set_aside_and_start_new(&dir.join(USER_FILE)).map_err(|e| e.to_string())?;
    *guard = Ok(db);
    Ok(kept)
}

/// D5-F6: a highlight is stored only for a verse that exists in a loaded
/// translation (OSIS book ID, case-sensitive).
pub fn check_reference(state: &AppState, reference: &str) -> Result<(), String> {
    let (book, chapter, verse) = user::parse_reference(reference).ok_or("invalid reference")?;
    let guard = lock(&state.content);
    let content = guard.as_ref().map_err(|e| e.clone())?;
    for b in &content.bibles {
        if b.has(book, chapter, Some(verse))
            .map_err(|e| e.to_string())?
        {
            return Ok(());
        }
    }
    Err("no such verse".into())
}

/// D5-F6: a saved position must name a loaded translation and a chapter (and
/// verses) that exist in it.
pub fn check_position(state: &AppState, saved: &SavedState) -> Result<(), String> {
    let Some(p) = &saved.position else {
        return Ok(());
    };
    let exists =
        |v: Option<u32>| with_bible(state, &p.translation, |b| b.has(&p.book, p.chapter, v));
    if !exists(None)? {
        return Err("invalid value: position".into());
    }
    for v in [p.anchor_verse, p.current_verse].into_iter().flatten() {
        if !exists(Some(v))? {
            return Err("invalid value: position".into());
        }
    }
    Ok(())
}

/// `set_highlight` without Tauri's State wrapper (tested directly).
pub fn highlight(state: &AppState, reference: &str, color: Option<&str>) -> Result<(), String> {
    check_reference(state, reference)?;
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.set_highlight(reference, color)
        .map_err(|e| e.to_string())
}

/// `save_state` without Tauri's State wrapper (tested directly).
pub fn save(state: &AppState, saved: &SavedState) -> Result<(), String> {
    check_position(state, saved)?;
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.save_state(saved).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct TranslationInfo {
    pub abbrev: String,
    pub name: String,
    pub has_red_letters: bool,
}

#[derive(Serialize)]
pub struct AppInfo {
    /// Set when content can't be shown: the UI shows the repair/reinstall message.
    pub content_error: Option<String>,
    pub user_error: Option<String>,
    /// W2A-U1: the refused file was made by a newer build (no "start a new data file").
    pub user_too_new: bool,
    pub translations: Vec<TranslationInfo>,
    pub lexicon_available: bool,
    /// Why the lexicons can't be used (missing or failed their check), if so.
    pub lexicon_error: Option<String>,
    pub search_available: bool,
    /// Why search can't be used (index missing, damaged or stale), if so.
    pub search_error: Option<String>,
    pub commentary_available: bool,
    /// Why the commentary can't be used (missing or failed its check), if so.
    pub commentary_error: Option<String>,
}

fn lock<'a, T>(m: &'a Mutex<T>) -> std::sync::MutexGuard<'a, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

fn valid_abbrev(s: &str) -> bool {
    !s.is_empty() && s.len() <= 16 && s.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub fn with_bible<R>(
    state: &AppState,
    translation: &str,
    f: impl FnOnce(&Bible) -> Result<R, QueryError>,
) -> Result<R, String> {
    if !valid_abbrev(translation) {
        return Err("invalid translation".into());
    }
    let guard = lock(&state.content);
    let content = guard.as_ref().map_err(|e| e.clone())?;
    let bible = content
        .bibles
        .iter()
        .find(|b| b.abbrev == translation)
        .ok_or("unknown translation")?;
    f(bible).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    info(&state)
}

pub fn info(state: &AppState) -> AppInfo {
    let content = lock(&state.content);
    let user = lock(&state.user);
    match content.as_ref() {
        Ok(c) => AppInfo {
            content_error: None,
            user_error: user.as_ref().err().map(|r| r.msg.clone()),
            user_too_new: user.as_ref().err().is_some_and(|r| r.too_new),
            translations: c
                .bibles
                .iter()
                .map(|b| TranslationInfo {
                    abbrev: b.abbrev.clone(),
                    name: b.name.clone(),
                    has_red_letters: b.has_red_letters,
                })
                .collect(),
            lexicon_available: c.lexicon.is_ok(),
            lexicon_error: c.lexicon.as_ref().err().cloned(),
            search_available: c.search.is_ok(),
            search_error: c.search.as_ref().err().cloned(),
            commentary_available: c.commentary.is_ok(),
            commentary_error: c.commentary.as_ref().err().cloned(),
        },
        Err(e) => AppInfo {
            content_error: Some(e.clone()),
            user_error: user.as_ref().err().map(|r| r.msg.clone()),
            user_too_new: user.as_ref().err().is_some_and(|r| r.too_new),
            translations: Vec::new(),
            lexicon_available: false,
            lexicon_error: None,
            search_available: false,
            search_error: None,
            commentary_available: false,
            commentary_error: None,
        },
    }
}

#[tauri::command]
pub fn list_books(
    state: State<'_, AppState>,
    translation: String,
) -> Result<Vec<BookInfo>, String> {
    with_bible(&state, &translation, |b| b.books())
}

#[tauri::command]
pub fn get_chapter(
    state: State<'_, AppState>,
    translation: String,
    book: u32,
    chapter: u32,
) -> Result<ChapterOut, String> {
    if !(1..=66).contains(&book) || !(1..=999).contains(&chapter) {
        return Err("invalid book or chapter".into());
    }
    with_bible(&state, &translation, |b| b.chapter(book, chapter))
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StrongsResult {
    Ok {
        entry: LexEntry,
    },
    NotFound {
        key: String,
    },
    /// The lexicon file is missing or failed its check; `reason` says which.
    NotAvailable {
        reason: String,
    },
}

#[tauri::command]
pub fn get_strongs(state: State<'_, AppState>, key: String) -> Result<StrongsResult, String> {
    strongs(&state, &key)
}

/// `get_strongs` without Tauri's State wrapper (tested directly).
pub fn strongs(state: &AppState, key: &str) -> Result<StrongsResult, String> {
    if !content::is_canonical_key(key) {
        return Err("invalid Strong's key".into());
    }
    let guard = lock(&state.content);
    let content = match guard.as_ref() {
        Ok(c) => c,
        Err(e) => return Ok(StrongsResult::NotAvailable { reason: e.clone() }),
    };
    let lex = match content.lexicon.as_ref() {
        Ok(l) => l,
        Err(e) => return Ok(StrongsResult::NotAvailable { reason: e.clone() }),
    };
    match lex.entry(key).map_err(|e| e.to_string())? {
        Some(entry) => Ok(StrongsResult::Ok { entry }),
        None => Ok(StrongsResult::NotFound {
            key: key.to_string(),
        }),
    }
}

#[tauri::command]
pub fn get_highlights(
    state: State<'_, AppState>,
    book: String,
    chapter: u32,
) -> Result<Vec<Highlight>, String> {
    let guard = lock(&state.user);
    let db = guard.as_ref().map_err(|e| e.clone())?;
    db.highlights(&book, chapter).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_highlight(
    state: State<'_, AppState>,
    reference: String,
    color: Option<String>,
) -> Result<(), String> {
    highlight(&state, &reference, color.as_deref())
}

#[tauri::command]
pub fn load_state(state: State<'_, AppState>) -> Result<SavedState, String> {
    let guard = lock(&state.user);
    let db = guard.as_ref().map_err(|e| e.clone())?;
    db.load_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_state(state: State<'_, AppState>, saved: SavedState) -> Result<(), String> {
    save(&state, &saved)
}

/// The reader chose "Start a new data file" on the refused-data banner.
/// Returns the kept (renamed) file's name, never a path.
#[tauri::command]
pub fn start_new_user_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "no app data folder is available".to_string())?;
    start_new_user(&state, &dir)
}

/// Phrase and word search (wave-2-layout 6). `query` at most 200 code
/// points; `scope` a closed enum; `offset` at most 40,000 and `limit` 1 to
/// 100. Results carry verse ids, references and the stored verse text from
/// `kjv.sqlite` with emphasis runs; never anything from the index.
#[tauri::command]
pub fn search(
    state: State<'_, AppState>,
    translation: String,
    query: String,
    scope: SearchScope,
    offset: u32,
    limit: u32,
) -> Result<SearchResult, String> {
    run_search(&state, &translation, &query, scope, offset, limit)
}

/// `search` without Tauri's State wrapper (tested directly).
pub fn run_search(
    state: &AppState,
    translation: &str,
    query: &str,
    scope: SearchScope,
    offset: u32,
    limit: u32,
) -> Result<SearchResult, String> {
    if !valid_abbrev(translation) {
        return Err("invalid translation".into());
    }
    search::validate(query, offset, limit)?;
    let guard = lock(&state.content);
    let content = match guard.as_ref() {
        Ok(c) => c,
        Err(e) => return Ok(SearchResult::Unavailable { reason: e.clone() }),
    };
    let index = match content.search.as_ref() {
        Ok(i) => i,
        Err(e) => return Ok(SearchResult::Unavailable { reason: e.clone() }),
    };
    if translation != search::SEARCH_TRANSLATION {
        return Ok(SearchResult::Unavailable {
            reason: format!("no search index for {translation}"),
        });
    }
    let bible = content
        .bibles
        .iter()
        .find(|b| b.abbrev == translation)
        .ok_or("unknown translation")?;
    let page = index
        .page(query, scope.into(), offset as usize, limit as usize)
        .map_err(|e| e.to_string())?;
    let phrase = matches!(
        page.query,
        Some(revelation_content::search::Query::Phrase(_))
    );
    Ok(SearchResult::Ok {
        total: page.total,
        exact_total: page.exact_total,
        offset: offset as usize,
        phrase,
        hits: search::hits(bible, &page)?,
    })
}

/// A bookmark names a book and chapter that exist in a loaded translation
/// (OSIS ID, case-sensitive), like highlights (D5-F6).
pub fn check_chapter(state: &AppState, book: &str, chapter: u32) -> Result<(), String> {
    if !user::valid_book_id(book) || !(1..=999).contains(&chapter) {
        return Err("invalid book or chapter".into());
    }
    let guard = lock(&state.content);
    let content = guard.as_ref().map_err(|e| e.clone())?;
    for b in &content.bibles {
        if b.has(book, chapter, None).map_err(|e| e.to_string())? {
            return Ok(());
        }
    }
    Err("no such chapter".into())
}

/// `bookmark_set` without Tauri's State wrapper (tested directly).
pub fn set_bookmark(state: &AppState, book: &str, chapter: u32) -> Result<(), String> {
    check_chapter(state, book, chapter)?;
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.set_bookmark(book, chapter).map_err(|e| e.to_string())
}

/// `bookmark_restore` without Tauri's State wrapper (tested directly).
pub fn restore_bookmark(state: &AppState, row: &Bookmark) -> Result<(), String> {
    check_chapter(state, &row.book, row.chapter)?;
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.restore_bookmark(row).map_err(|e| e.to_string())
}

/// `bookmark_clear` without Tauri's State wrapper (tested directly).
pub fn clear_bookmark(state: &AppState, book: &str) -> Result<Option<Bookmark>, String> {
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.clear_bookmark(book).map_err(|e| e.to_string())
}

/// Every automatic bookmark (bookmarks.md 8).
#[tauri::command]
pub fn bookmarks_list(state: State<'_, AppState>) -> Result<Vec<Bookmark>, String> {
    let guard = lock(&state.user);
    let db = guard.as_ref().map_err(|e| e.clone())?;
    db.bookmarks().map_err(|e| e.to_string())
}

/// A chapter counted as read: `book`'s bookmark is now `chapter`, read now.
#[tauri::command]
pub fn bookmark_set(state: State<'_, AppState>, book: String, chapter: u32) -> Result<(), String> {
    set_bookmark(&state, &book, chapter)
}

/// "Clear bookmark": returns the old row for Undo, or null if there was none.
#[tauri::command]
pub fn bookmark_clear(
    state: State<'_, AppState>,
    book: String,
) -> Result<Option<Bookmark>, String> {
    clear_bookmark(&state, &book)
}

/// Undo of a clear: that exact row back.
#[tauri::command]
pub fn bookmark_restore(state: State<'_, AppState>, row: Bookmark) -> Result<(), String> {
    restore_bookmark(&state, &row)
}

// ---- pen-mode range highlights (highlights-pen.md 5.3) ----
// The backend is the gate: every call names a loaded translation (exact id),
// a chapter that exists in it, and pieces that are checked against that
// chapter's stored text here (bounds, B1, contiguity). Lengths and
// fingerprints are computed here from the read-only content, never sent by
// the UI. Only user.sqlite is written.

/// The stored verse texts of a chapter of a loaded translation, or an error.
pub fn chapter_texts(
    state: &AppState,
    translation: &str,
    book: &str,
    chapter: u32,
) -> Result<Vec<(u32, String)>, String> {
    if !user::valid_book_id(book) || !(1..=999).contains(&chapter) {
        return Err("invalid book or chapter".into());
    }
    let texts = with_bible(state, translation, |b| b.chapter_texts(book, chapter))?;
    if texts.is_empty() {
        return Err("no such chapter".into());
    }
    Ok(texts)
}

/// Run `f` on the user database with the chapter's key and texts (tested directly).
pub fn with_marks<R>(
    state: &AppState,
    translation: &str,
    book: &str,
    chapter: u32,
    f: impl FnOnce(&mut UserDb, &user::ChapterKey, &[(u32, String)]) -> Result<R, user::UserError>,
) -> Result<R, String> {
    let texts = chapter_texts(state, translation, book, chapter)?;
    let key = user::ChapterKey {
        translation,
        book,
        chapter,
    };
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    f(db, &key, &texts).map_err(|e| e.to_string())
}

/// The chapter's range marks in this translation, each piece flagged stale
/// if its verse text changed.
#[tauri::command]
pub fn marks_for_chapter(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.marks(k, t)
    })
}

/// One pen stroke: returns the chapter's new marks.
#[tauri::command]
pub fn pen_apply(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
    color: String,
    pieces: Vec<Piece>,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.pen_apply(k, t, &color, &pieces)
    })
}

/// One eraser stroke: returns the chapter's new marks.
#[tauri::command]
pub fn pen_erase(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
    pieces: Vec<Piece>,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.pen_erase(k, t, &pieces)
    })
}

/// Remove one whole mark of this chapter.
#[tauri::command]
pub fn mark_remove(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
    id: i64,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.mark_remove(k, t, id)
    })
}

/// Recolour one whole mark of this chapter.
#[tauri::command]
pub fn mark_recolor(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
    id: i64,
    color: String,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.mark_recolor(k, t, id, &color)
    })
}

/// Undo / redo: restore a snapshot of the chapter's drawn marks.
#[tauri::command]
pub fn marks_replace_chapter(
    state: State<'_, AppState>,
    translation: String,
    book: String,
    chapter: u32,
    marks: Vec<MarkIn>,
) -> Result<Vec<MarkOut>, String> {
    with_marks(&state, &translation, &book, chapter, |db, k, t| {
        db.marks_replace_chapter(k, t, &marks)
    })
}

/// The commentary on one KJV verse (wave-2-layout 7; DESIGN.md 17.5): the
/// blocks of every anchor that lists it, or the whole chapter's blocks
/// (`whole_chapter`) when no anchor does. The verse must exist in the KJV.
#[tauri::command]
pub fn get_commentary(
    state: State<'_, AppState>,
    book: u32,
    chapter: u32,
    verse: u32,
) -> Result<CommentaryResult, String> {
    commentary_for(&state, book, chapter, verse)
}

/// `get_commentary` without Tauri's State wrapper (tested directly).
pub fn commentary_for(
    state: &AppState,
    book: u32,
    chapter: u32,
    verse: u32,
) -> Result<CommentaryResult, String> {
    if !(1..=66).contains(&book) || !(1..=999).contains(&chapter) || !(1..=999).contains(&verse) {
        return Err("invalid verse".into());
    }
    let id = i64::from(book) * 1_000_000 + i64::from(chapter) * 1000 + i64::from(verse);
    let guard = lock(&state.content);
    let content = match guard.as_ref() {
        Ok(c) => c,
        Err(e) => return Ok(CommentaryResult::Unavailable { reason: e.clone() }),
    };
    let cmt = match content.commentary.as_ref() {
        Ok(c) => c,
        Err(e) => return Ok(CommentaryResult::Unavailable { reason: e.clone() }),
    };
    let kjv = content
        .bibles
        .iter()
        .find(|b| b.abbrev == search::SEARCH_TRANSLATION)
        .ok_or("unknown translation")?;
    if kjv.verse_by_id(id).map_err(|e| e.to_string())?.is_none() {
        return Err("no such verse".into());
    }
    let (whole_chapter, groups) = cmt.for_verse(id)?;
    Ok(CommentaryResult::Ok {
        source: cmt.source.clone(),
        verse_id: id,
        whole_chapter,
        groups,
    })
}

// ---- daily notes (W2-5, wave-2-layout 8) ----
// The UI sends the whole day's document; user.rs checks the day and the
// schema (notes.rs) and writes it in one transaction. Only user.sqlite is
// written.

pub fn notes_page(
    state: &AppState,
    before: Option<&str>,
    limit: u32,
) -> Result<user::NotesPage, String> {
    let guard = lock(&state.user);
    let db = guard.as_ref().map_err(|e| e.clone())?;
    db.notes_list(before, limit).map_err(|e| e.to_string())
}

pub fn save_note(state: &AppState, day: &str, doc: &str) -> Result<(), String> {
    let mut guard = lock(&state.user);
    let db = guard.as_mut().map_err(|e| e.clone())?;
    db.note_save(day, doc).map_err(|e| e.to_string())
}

/// Days with notes, newest first, `limit` at a time, before `before`.
#[tauri::command]
pub fn notes_list(
    state: State<'_, AppState>,
    before: Option<String>,
    limit: u32,
) -> Result<user::NotesPage, String> {
    notes_page(&state, before.as_deref(), limit)
}

/// Save one day's note (an empty document removes the day).
#[tauri::command]
pub fn note_save(state: State<'_, AppState>, day: String, doc: String) -> Result<(), String> {
    save_note(&state, &day, &doc)
}

// ---- closing the window (W2-5: notes are flushed before the app closes) ----
// The close button is held once: the page is asked to save (lib.rs evals
// `window.__revelationClose()`), and it answers with `app_close`. `hold`
// means the page is asking the reader (a save failed); otherwise the window
// closes. If the page doesn't answer in `CLOSE_WAIT`, it closes anyway, so a
// broken page can never keep the window open.

/// How long a close waits for the page's answer.
pub const CLOSE_WAIT: std::time::Duration = std::time::Duration::from_secs(3);

#[derive(Default)]
pub struct CloseGate {
    inner: Mutex<CloseInner>,
}

#[derive(Default)]
struct CloseInner {
    /// The next close request goes through.
    allowed: bool,
    /// Which close request is waiting (each request bumps it).
    asked: u64,
    /// The page answered "hold" for request `asked`.
    held: bool,
}

impl CloseGate {
    /// A close was requested: None to let it go through, else hold it and ask
    /// the page (the number identifies this request for [`Self::timed_out`]).
    pub fn request(&self) -> Option<u64> {
        let mut g = lock(&self.inner);
        if g.allowed {
            return None;
        }
        g.asked += 1;
        g.held = false;
        Some(g.asked)
    }
    /// The page answered: hold (asking the reader) or close now.
    pub fn answer(&self, hold: bool) -> bool {
        let mut g = lock(&self.inner);
        if hold {
            g.held = true;
            false
        } else {
            g.allowed = true;
            true
        }
    }
    /// `CLOSE_WAIT` after request `n`: true if the window must close now
    /// (the page never answered that request, and it isn't holding).
    pub fn timed_out(&self, n: u64) -> bool {
        let mut g = lock(&self.inner);
        if g.allowed || g.asked != n || g.held {
            return false;
        }
        g.allowed = true;
        true
    }
}

/// The page's answer to a close request (see [`CloseGate`]).
#[tauri::command]
pub fn app_close(window: tauri::Window, gate: State<'_, CloseGate>, hold: bool) {
    if gate.answer(hold) {
        let _ = window.close();
    }
}

#[cfg(test)]
mod close_tests {
    use super::*;

    #[test]
    fn a_close_waits_for_the_page_or_the_timeout_once() {
        let g = CloseGate::default();
        // the page saves and answers: the second request goes through
        let n = g.request().unwrap();
        assert!(g.answer(false));
        assert_eq!(g.request(), None);
        assert!(!g.timed_out(n), "already closing");

        // no answer: the timeout closes
        let g = CloseGate::default();
        let n = g.request().unwrap();
        assert!(g.timed_out(n));
        assert_eq!(g.request(), None);

        // the page holds (a failed save, asking the reader): no timeout
        let g = CloseGate::default();
        let n = g.request().unwrap();
        assert!(!g.answer(true));
        assert!(!g.timed_out(n));
        // the reader keeps it open and later closes again: asked afresh
        let m = g.request().unwrap();
        assert!(m > n);
        assert!(!g.timed_out(n), "a stale timer never closes");
        assert!(g.timed_out(m));
    }
}
