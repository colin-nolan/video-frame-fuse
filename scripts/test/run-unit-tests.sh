#!/usr/bin/env bash

set -euf -o pipefail

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"
repository_root_directory="$(cd "${script_directory}" && git rev-parse --show-toplevel)"

pushd "${repository_root_directory}" > /dev/null

cargo test --jobs "$(nproc)"

popd > /dev/null
