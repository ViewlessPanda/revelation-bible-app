#!/usr/bin/env bash
# Review item M0-F2: Git must never change the bytes of a content file.
#
# .gitattributes unsets text/ident/filter/working-tree-encoding for every path,
# but $GIT_DIR/info/attributes (or a .gitattributes in a subdirectory) outranks
# it. So, after checkout, every tracked content file's working-tree bytes must
# hash to exactly the blob recorded in the index:
#
#   git hash-object --no-filters <path>  ==  blob id from `git ls-files -s`
#
# Content paths: sources/**, **/golden/**, pipeline/tests/fixtures/**,
# pipeline/tests/expected/**, docs/pipeline/survey-scripts/fixtures/**, and
# every *.sha256. Needs only bash and git. Run from the repository root.
set -euo pipefail
export LC_ALL=C

is_content_path() {
  # In a bash `case` pattern, * also matches "/".
  case "$1" in
    sources/* | pipeline/tests/fixtures/* | pipeline/tests/expected/* | \
    docs/pipeline/survey-scripts/fixtures/* | golden/* | */golden/* | *.sha256)
      return 0 ;;
  esac
  return 1
}

tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
git ls-files -s -z > "$tmp"

checked=0
bad=0
while IFS= read -r -d '' rec; do
  meta=${rec%%$'\t'*}   # "<mode> <blob> <stage>"
  path=${rec#*$'\t'}
  is_content_path "$path" || continue
  read -r mode blob stage <<< "$meta"
  if [ "$stage" != "0" ]; then
    echo "::error::unmerged index entry for content path: $path"
    bad=1
    continue
  fi
  if [ "$mode" != "100644" ] && [ "$mode" != "100755" ]; then
    echo "::error::content path is not a regular file in the index (mode $mode): $path"
    bad=1
    continue
  fi
  if [ -L "$path" ] || [ ! -f "$path" ]; then
    echo "::error::content path missing or not a regular file in the working tree: $path"
    bad=1
    continue
  fi
  actual=$(git hash-object --no-filters -- "$path")
  if [ "$actual" != "$blob" ]; then
    echo "::error::working-tree bytes differ from the index blob: $path (index $blob, working tree $actual)"
    bad=1
  fi
  checked=$((checked + 1))
done < "$tmp"

if [ "$checked" -eq 0 ] && [ "$bad" -eq 0 ]; then
  # Guard against a broken pattern or listing making this check vacuous.
  echo "::error::no tracked content files found; the byte check checked nothing"
  exit 1
fi
if [ "$bad" -ne 0 ]; then
  echo "content byte check FAILED"
  exit 1
fi
echo "content byte check passed: $checked files byte-identical to their index blobs"
