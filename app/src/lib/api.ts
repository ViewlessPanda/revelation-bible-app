// Typed wrappers for the core's commands. The only IPC the UI uses.
import { invoke } from "@tauri-apps/api/core";
import type { Bookmark } from "./bookmarks";
import type { CommentaryResult } from "./commentary";
import type { MarkIn, MarkOut, Piece } from "./pen";
import type {
  AppInfo, Book, Chapter, Highlight, HighlightColor, SavedState, SearchResult, SearchScope, StrongsResult,
} from "./types";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  listBooks: (translation: string) => invoke<Book[]>("list_books", { translation }),
  getChapter: (translation: string, book: number, chapter: number) =>
    invoke<Chapter>("get_chapter", { translation, book, chapter }),
  getStrongs: (key: string) => invoke<StrongsResult>("get_strongs", { key }),
  getHighlights: (book: string, chapter: number) => invoke<Highlight[]>("get_highlights", { book, chapter }),
  setHighlight: (reference: string, color: HighlightColor | null) =>
    invoke<void>("set_highlight", { reference, color }),
  loadState: () => invoke<SavedState>("load_state"),
  saveState: (saved: SavedState) => invoke<void>("save_state", { saved }),
  /** After user.sqlite was refused: keep it (renamed) and start a new one. Returns the kept file name. */
  startNewUserData: () => invoke<string>("start_new_user_data"),
  /** Phrase and word search (wave-2-layout 6): verse ids, references and stored text with emphasis runs. */
  search: (translation: string, query: string, scope: SearchScope, offset: number, limit: number) =>
    invoke<SearchResult>("search", { translation, query, scope, offset, limit }),
  /** Automatic bookmarks (bookmarks.md 8). */
  bookmarksList: () => invoke<Bookmark[]>("bookmarks_list"),
  bookmarkSet: (book: string, chapter: number) => invoke<void>("bookmark_set", { book, chapter }),
  /** Returns the old row, for Undo. */
  bookmarkClear: (book: string) => invoke<Bookmark | null>("bookmark_clear", { book }),
  bookmarkRestore: (row: Bookmark) => invoke<void>("bookmark_restore", { row }),
  /** The commentary on one KJV verse (wave-2-layout 7): its anchors' blocks, or the whole chapter's. */
  getCommentary: (book: number, chapter: number, verse: number) =>
    invoke<CommentaryResult>("get_commentary", { book, chapter, verse }),
  /** Pen-mode range highlights (highlights-pen.md 5.3): every call returns the chapter's marks. */
  marksForChapter: (translation: string, book: string, chapter: number) =>
    invoke<MarkOut[]>("marks_for_chapter", { translation, book, chapter }),
  penApply: (translation: string, book: string, chapter: number, color: HighlightColor, pieces: Piece[]) =>
    invoke<MarkOut[]>("pen_apply", { translation, book, chapter, color, pieces }),
  penErase: (translation: string, book: string, chapter: number, pieces: Piece[]) =>
    invoke<MarkOut[]>("pen_erase", { translation, book, chapter, pieces }),
  markRemove: (translation: string, book: string, chapter: number, id: number) =>
    invoke<MarkOut[]>("mark_remove", { translation, book, chapter, id }),
  markRecolor: (translation: string, book: string, chapter: number, id: number, color: HighlightColor) =>
    invoke<MarkOut[]>("mark_recolor", { translation, book, chapter, id, color }),
  /** Daily notes (wave-2-layout 8): days newest first, `limit` at a time; a doc is null when the row can't be read. */
  notesList: (before: string | null, limit: number) =>
    invoke<{ days: { day: string; doc: string | null; updated_at: number }[]; more: boolean }>("notes_list", { before, limit }),
  /** One day's whole document (JSON); an empty document removes the day. */
  noteSave: (day: string, doc: string) => invoke<void>("note_save", { day, doc }),
  /** The answer to a close request: hold (asking the reader) or close now. */
  appClose: (hold: boolean) => invoke<void>("app_close", { hold }),
  marksReplaceChapter: (translation: string, book: string, chapter: number, marks: MarkIn[]) =>
    invoke<MarkOut[]>("marks_replace_chapter", { translation, book, chapter, marks }),
};