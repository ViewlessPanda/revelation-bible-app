# sources/checksums

SHA-256 checksum files for pinned source files. `sources/SOURCES.md` is the
authority for each source's URL, version, license and checksum. The pipeline
verifies every downloaded file before parsing it, and refuses to run on a
mismatch (PLAN.md Section 2, rule 3). The file format is set in Milestone 1.

Files here are byte-exact: `.gitattributes` disables all line-ending
conversion under `sources/`.
