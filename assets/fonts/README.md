# assets/fonts

Bundled fonts (PLAN.md Section 6). Only fonts under the SIL Open Font License
are added here, each with its own licence file copied verbatim (F2).

| Folder | Files (unmodified upstream, upstream names) | SOURCES.md row |
|---|---|---|
| `literata/` | `Literata[opsz,wght].ttf`, `Literata-Italic[opsz,wght].ttf`, `OFL.txt` | FONT-LITERATA (3.103) |
| `inter/` | `InterVariable.ttf`, `LICENSE.txt` | FONT-INTER (4.1) |

`fonts.sha256` lists the SHA-256 of every file. `app/scripts/sync-fonts.mjs`
checks each file against it before copying it into the app build, and checks
the built copies again; a changed file stops the build. Never subset, convert,
rename or edit these files (F1). Adding a font means a cleared SOURCES.md row,
an F4 name-table check, and a line in `fonts.sha256`.
