#!/usr/bin/env bash

# Sort order is affected by locale. See `man sort`.
# > Set LC_ALL=C to get the traditional sort order that uses native byte values.
export LC_ALL=C

REPO_DIR=$(git rev-parse --show-toplevel)

listTargetFiles() {
  pushd "$REPO_DIR/fuzz" > /dev/null || exit 1
  find fuzz_targets/ -type f -name "*.rs" | sort
  popd > /dev/null || exit 1
}

targetFileToName() {
  echo "$1" \
    | sed 's/^fuzz_targets\///' \
    | sed 's/\.rs$//' \
    | sed 's/\//_/g' \
    | sed 's/^_//g'
}

listTargetNames() {
  for target in $(listTargetFiles); do
    targetFileToName "$target"
  done
}

# Utility function to avoid CI failures on Windows
checkWindowsFiles() {
  incorrectFilenames=$(find . -type f -name "*,*" -o -name "*:*" -o -name "*<*" -o -name "*>*" -o -name "*|*" -o -name "*\?*" -o -name "*\**" -o -name "*\"*" | wc -l)
  if [ "$incorrectFilenames" -gt 0 ]; then
    echo "Bailing early because there is a Windows-incompatible filename in the tree."
    exit 2
  fi
}

# Checks whether a fuzz case outputs some report, and dumps it in hex
checkReport() {
  reportFile="hfuzz_workspace/$1/HONGGFUZZ.REPORT.TXT"
  if [ -f "$reportFile" ]; then
    cat "$reportFile"
    for CASE in "hfuzz_workspace/$1/SIG"*; do
      xxd -p -c10000 < "$CASE"
    done
    exit 1
  fi
}
