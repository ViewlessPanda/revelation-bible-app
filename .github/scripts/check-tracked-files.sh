#!/usr/bin/env bash
# Review item M0-F3: no raw source file and no large file gets committed.
#
# Fails if
#  (a) any tracked file's SHA-256 equals a pinned source SHA-256 from
#      sources/checksums/*.sha256 (a raw source committed under any name), or
#  (b) any tracked file is larger than 5 MiB (5,242,880 bytes) and lies
#      outside pipeline/tests/fixtures/.
#
# Sizes are the index blob sizes; hashes are of the checked-out files (which
# check-content-bytes.sh and .gitattributes keep equal to the blobs).
# Needs only bash, git, sha256sum and xargs. Run from the repository root.
set -euo pipefail
export LC_ALL=C

MAX_BYTES=5242880

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
bad=0

# ---- pinned source hashes ----------------------------------------------
shopt -s nullglob
sums=(sources/checksums/*.sha256)
shopt -u nullglob
if [ "${#sums[@]}" -eq 0 ]; then
  echo "::error::no sources/checksums/*.sha256 files found"
  exit 1
fi
: > "$tmp/pinned"
for f in "${sums[@]}"; do
  while IFS= read -r line || [ -n "$line" ]; do
    if [[ "$line" =~ ^([0-9a-f]{64})\ \ [A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
      echo "${BASH_REMATCH[1]}" >> "$tmp/pinned"
    else
      echo "::error::malformed line in $f (expected '<64 lowercase hex>  <name>', LF only)"
      bad=1
    fi
  done < "$f"
done
if [ ! -s "$tmp/pinned" ]; then
  echo "::error::no pinned SHA-256 values found in sources/checksums/"
  exit 1
fi

# ---- tracked regular files: paths, blob ids --------------------------------
git ls-files -s -z > "$tmp/index"
paths=()
: > "$tmp/blobs"
: > "$tmp/paths0"
while IFS= read -r -d '' rec; do
  meta=${rec%%$'\t'*}
  path=${rec#*$'\t'}
  read -r mode blob stage <<< "$meta"
  case "$mode" in
    100644 | 100755) ;;
    *) continue ;;    # symlinks and submodules carry no file content here
  esac
  paths+=("$path")
  echo "$blob" >> "$tmp/blobs"
  printf '%s\0' "$path" >> "$tmp/paths0"
done < "$tmp/index"
n=${#paths[@]}
if [ "$n" -eq 0 ]; then
  echo "::error::no tracked files found"
  exit 1
fi

# ---- (b) size --------------------------------------------------------------
git cat-file --batch-check='%(objectsize)' < "$tmp/blobs" > "$tmp/sizes"
mapfile -t sizes < "$tmp/sizes"
if [ "${#sizes[@]}" -ne "$n" ]; then
  echo "::error::size listing has ${#sizes[@]} lines for $n files"
  exit 1
fi
for ((i = 0; i < n; i++)); do
  s=${sizes[$i]}
  if ! [[ "$s" =~ ^[0-9]+$ ]]; then
    echo "::error::cannot read the blob size of ${paths[$i]}: $s"
    bad=1
    continue
  fi
  if [ "$s" -gt "$MAX_BYTES" ]; then
    case "${paths[$i]}" in
      pipeline/tests/fixtures/*) ;;
      *)
        echo "::error::tracked file larger than $MAX_BYTES bytes outside pipeline/tests/fixtures/: ${paths[$i]} ($s bytes)"
        bad=1 ;;
    esac
  fi
done

# ---- (a) SHA-256 equal to a pinned source ----------------------------------
declare -A pinned_set=()
while IFS= read -r h; do pinned_set[$h]=1; done < "$tmp/pinned"
if ! xargs -0 -r sha256sum -- < "$tmp/paths0" > "$tmp/hashes"; then
  echo "::error::sha256sum failed on a tracked file"
  exit 1
fi
mapfile -t hashes < "$tmp/hashes"
if [ "${#hashes[@]}" -ne "$n" ]; then
  echo "::error::sha256sum printed ${#hashes[@]} lines for $n files"
  exit 1
fi
for ((i = 0; i < n; i++)); do
  h=${hashes[$i]#\\}   # sha256sum prefixes "\" when it escapes a file name
  h=${h:0:64}
  if ! [[ "$h" =~ ^[0-9a-f]{64}$ ]]; then
    echo "::error::unexpected sha256sum output for ${paths[$i]}"
    bad=1
    continue
  fi
  if [ -n "${pinned_set[$h]+x}" ]; then
    echo "::error::tracked file has the SHA-256 of a pinned source (a raw source must not be committed): ${paths[$i]} ($h)"
    bad=1
  fi
done

if [ "$bad" -ne 0 ]; then
  echo "tracked-file check FAILED"
  exit 1
fi
echo "tracked-file check passed: $n files; none matches one of $(wc -l < "$tmp/pinned") pinned source hashes; none over $MAX_BYTES bytes outside pipeline/tests/fixtures/"
