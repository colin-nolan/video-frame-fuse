#!/usr/bin/env bash

set -euf -o pipefail

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"
directory_manifest_location="${script_directory}/manifest.csv"

# TODO: if already initialised?

while IFS=, read -r _ location; do
    >&2 echo "Initialising: ${location}..."
    stat "${script_directory}/${location}" > /dev/null
done < <(tail -n +2 "${directory_manifest_location}")

>&2 echo "Complete!"
