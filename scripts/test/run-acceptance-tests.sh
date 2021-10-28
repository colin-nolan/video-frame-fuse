#!/usr/bin/env bash

set -euf -o pipefail

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"
repository_root_directory="$(cd "${script_directory}" && git rev-parse --show-toplevel)"

TOOL="${TOOL:="$(cd "${script_directory}" && git rev-parse --show-toplevel)/target/release/video-frame-fuse"}"
export TOOL

if [[ ! -f "${TOOL}" ]]; then
    >&2 echo "Tool does not exist in location: ${TOOL}. Consider building with: ./scripts/build/run-release-build.sh"
    exit 1
fi

pushd "${repository_root_directory}" > /dev/null

shellspec -j "$(nproc)" --format tap --shell bash "$@"

popd > /dev/null
