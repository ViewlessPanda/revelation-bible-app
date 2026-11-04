#!/usr/bin/env bash
# Downloads the pinned source files into <raw-dir>/<ID>/<name> (normally
# sources/raw, which is git-ignored) and verifies each one with `sha256sum -c`
# against sources/checksums/<ID>.sha256 BEFORE it lands there: each file is
# fetched into a fresh staging directory, checked, and only then moved into
# place. A mismatch stops the job and leaves nothing in <raw-dir> for that ID.
#
# The URLs below must appear verbatim as the "Exact file URL" of a row in
# sources/SOURCES.md (checked here), and each URL's last path segment must be
# the file name in the ID's checksum file. The files are untrusted data: they
# are only hashed here and never executed. The pipeline re-verifies them.
#
# Usage (from the repository root): bash .github/scripts/fetch-sources.sh <raw-dir>
# Needs bash, curl, sha256sum, grep and mktemp.
set -euo pipefail
export LC_ALL=C

if [ "$#" -ne 1 ]; then
  echo "usage: fetch-sources.sh <raw-dir>" >&2
  exit 2
fi
raw=$1
repo=$PWD

# ID|URL, one per pinned source used by the pipeline.
pins=(
  "BIB-KJV-CW31|https://gitlab.com/crosswire-bible-society/kjv/-/raw/d490be7e34762deb2c76cb2c1306d4808e27890d/kjv.osis.xml"
  "LEX-HEB-OSHB|https://raw.githubusercontent.com/openscriptures/HebrewLexicon/21c9add13bc727d3a951361778e97e3ff7afd1ce/HebrewStrong.xml"
  "LEX-GRC-MGNT19|https://raw.githubusercontent.com/morphgnt/strongs-dictionary-xml/dd6758b82f46b620804a8ea677d715844679eea1/strongsgreek.xml"
  "VRS-ENG-PALASO|https://raw.githubusercontent.com/sillsdev/libpalaso/f1f70e93a79d55b1769a4e20fb36047f2fa08d99/SIL.Scripture/Resources/eng.vrs.txt"
  "CMT-MHCC-CW20|https://gitlab.com/crosswire-bible-society/mhcc/-/raw/b15ff40aab9556005fd3d38d5388040a90b99145/mhcc.osis.xml"
)

stage_root=$(mktemp -d)
trap 'rm -rf "$stage_root"' EXIT

for pin in "${pins[@]}"; do
  id=${pin%%|*}
  url=${pin#*|}
  sums="$repo/sources/checksums/$id.sha256"

  # The checksum file must be exactly one '<64 hex>  <name>' line.
  [ -f "$sums" ] || { echo "::error::missing $sums"; exit 1; }
  [ "$(wc -l < "$sums")" -eq 1 ] || { echo "::error::$sums must have exactly one line"; exit 1; }
  line=$(cat "$sums")
  if [[ ! "$line" =~ ^[0-9a-f]{64}\ \ ([A-Za-z0-9][A-Za-z0-9._-]*)$ ]]; then
    echo "::error::malformed $sums"
    exit 1
  fi
  name=${BASH_REMATCH[1]}

  # The URL is the one SOURCES.md pins, and it names the checksummed file.
  if ! grep -qxF -- "- **Exact file URL:** $url" sources/SOURCES.md; then
    echo "::error::$id: URL is not an 'Exact file URL' line in sources/SOURCES.md: $url"
    exit 1
  fi
  if [ "${url##*/}" != "$name" ]; then
    echo "::error::$id: URL file name ${url##*/} differs from $name in $sums"
    exit 1
  fi
  if [ -e "$raw/$id" ]; then
    echo "::error::$raw/$id already exists; refusing to mix files"
    exit 1
  fi

  stage="$stage_root/$id"
  mkdir "$stage"
  echo "fetching $id: $url"
  curl --fail --silent --show-error --location \
    --proto '=https' --proto-redir '=https' --tlsv1.2 \
    --retry 3 --retry-delay 5 --retry-all-errors \
    --connect-timeout 30 --max-time 600 --max-filesize 104857600 \
    --output "$stage/$name" -- "$url"

  # Verify before anything else reads the file.
  if ! (cd "$stage" && sha256sum --strict -c "$sums"); then
    echo "::error::$id: downloaded $name does not match $sums"
    exit 1
  fi

  mkdir -p "$raw/$id"
  mv -- "$stage/$name" "$raw/$id/$name"
done

echo "all ${#pins[@]} pinned sources downloaded and verified into $raw"
