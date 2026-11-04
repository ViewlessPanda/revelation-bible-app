// Required attribution lines, copied exactly from sources/SOURCES.md (static
// text bundled in the app; no network). tests/about.test.ts reads SOURCES.md
// and checks every string below against its row, so a change on either side
// fails the build until both agree. Shown in the About dialog and, for the
// lexicons, under the entries in the study panel (CC BY 4.0 §3(a)).

export interface Credit {
  /** SOURCES.md row ID */
  id: string;
  title: string;
  /** paragraphs, each exactly as in the row */
  paragraphs: string[];
}

/** SOURCES.md row BIB-KJV-CW31, field "Required attribution" (both paragraphs). */
export const KJV_CREDIT: Credit = {
  id: "BIB-KJV-CW31",
  title: "King James Version",
  paragraphs: [
    "King James Version (1769) text with Strong's numbers and words-of-Christ markup from the CrossWire Bible Society KJV module, version 3.1 (https://gitlab.com/crosswire-bible-society/kjv). Any copyright that might be obtained for this effort is held by CrossWire Bible Society © 2003-2023, which grants a general public license to use this text for any purpose (distributed under the GNU General Public License). OT Strong's numbers from The Bible Foundation; NT Strong's data from CrossWire's KJV2003 Project, with Greek data from Dr. Maurice A. Robinson; formatting markup by Michael Paul Johnson (eBible.org). Red-letter markup follows Louis Klopsch's 1901 red-letter edition, per CrossWire. Converted into Revelation's database format by the Revelation pipeline; the text and markup boundaries are unchanged.",
    "Rights in the Authorized (King James) Version in the United Kingdom are vested in the Crown.",
  ],
};

/** SOURCES.md row LEX-HEB-OSHB, field "Required attribution" (CC BY 4.0). */
export const OSHB_CREDIT: Credit = {
  id: "LEX-HEB-OSHB",
  title: "Strong's Hebrew Dictionary",
  paragraphs: ["Strong's Hebrew Dictionary (James Strong, 1890; public domain), XML edition by the Open Scriptures Hebrew Bible Project (https://github.com/openscriptures/HebrewLexicon), licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). Provided \"as is\" without warranties. Converted to Revelation's database format; entry text unchanged."],
};

/** SOURCES.md row LEX-GRC-MGNT19, field "Required attribution" (courtesy credit; CC0). */
export const MGNT_CREDIT: Credit = {
  id: "LEX-GRC-MGNT19",
  title: "Strong's Greek Dictionary",
  paragraphs: ["Strong's Greek Dictionary (James Strong, 1890; public domain), XML edition \"Strong's Greek Dictionary in XML with real Greek\" v1.9 by Ulrik Sandborg-Petersen (CC0), from an e-text by Michael Grier distributed by The Bible Foundation."],
};

/** SOURCES.md row CMT-MHCC-CW20, field "Required attribution" (condition 3:
 *  kept with the commentary wherever its source is shown). */
export const MHCC_CREDIT: Credit = {
  id: "CMT-MHCC-CW20",
  title: "Commentary",
  paragraphs: ["Matthew Henry's Concise Commentary on the Whole Bible (public domain), from the CrossWire Bible Society MHCC module, version 2.0 (https://gitlab.com/crosswire-bible-society/mhcc, commit b15ff40), which was prepared from the Christian Classics Ethereal Library's text. Converted into Revelation's database format by the Revelation pipeline; text unchanged."],
};

/** Credit for a lexicon ID stored in `lex_entry.lexicon`. */
export const LEXICON_CREDITS: Record<string, Credit> = {
  "LEX-HEB-OSHB": OSHB_CREDIT,
  "LEX-GRC-MGNT19": MGNT_CREDIT,
};

/**
 * Fonts: SOURCES.md "Standing font conditions" F3 (in-app credit: copyright
 * line, "SIL Open Font License 1.1", upstream link). Each copyright line is the
 * one quoted in the row's "License" field, and the link is the row's
 * "Publisher / maintainer" URL.
 */
export interface FontCredit {
  /** SOURCES.md row ID */
  id: string;
  name: string;
  copyright: string;
  licence: "SIL Open Font License 1.1";
  url: string;
}

export const FONT_CREDITS: FontCredit[] = [
  {
    // SOURCES.md row FONT-LITERATA ("License" and "Publisher / maintainer" fields)
    id: "FONT-LITERATA",
    name: "Literata",
    copyright: "Copyright 2017 The Literata Project Authors (https://github.com/googlefonts/literata)",
    licence: "SIL Open Font License 1.1",
    url: "https://github.com/googlefonts/literata",
  },
  {
    // SOURCES.md row FONT-INTER ("License" and "Publisher / maintainer" fields)
    id: "FONT-INTER",
    name: "Inter",
    copyright: "Copyright (c) 2016 The Inter Project Authors (https://github.com/rsms/inter)",
    licence: "SIL Open Font License 1.1",
    url: "https://github.com/rsms/inter",
  },
];

/** The app's own licence (LICENSE, first lines). */
export const APP_NAME = "Revelation";
export const APP_COPYRIGHT = "Copyright (c) 2026 C. Ethan Key";
export const APP_LICENCE = "MIT License";

/** About, under the version (wave-2-layout.md 14). */
export const APP_DESCRIPTION = "An open-source Bible app that works offline.";
export const APP_CREDIT = "Made by C. Ethan Key";
/** The icon credit shown on the About screen. */
export const ICON_CREDIT = "Icons: Lucide (ISC); includes icons from Feather (MIT)";

/**
 * About, directly under the KJV credit: the required wording, exactly
 * (licenses/KJV-NOTICE.md; tests/licence.test.ts compares it).
 * `label` is set in bold.
 */
export const KJV_LICENCE_NOTICE = {
  label: "License.",
  text: "The King James Version text (CrossWire KJV 3.1, with Strong's numbers and words-of-Jesus markup) is distributed under the GNU General Public License, version 3. It is kept in its own data file, kjv.sqlite; the rest of Revelation is under the MIT License. The license text is included with the app (GPL-3.0.txt), and the source (CrossWire's kjv.osis.xml and the pipeline that converts it) is published with each release at https://github.com/ViewlessPanda/revelation-bible-app/releases.",
};
