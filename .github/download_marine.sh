#!/bin/bash
set -o pipefail -o errexit -o nounset
set -x

MARINE_RELEASE="https://api.github.com/repos/fluencelabs/marine/releases/latest"
OUT_DIR=/usr/local/bin

# get metadata about release
curl -s -H "Accept: application/vnd.github.v3+json" $MARINE_RELEASE |
    # extract url and name for asset with name "marine"
    # also append $OUT_DIR to each name so file is saved to $OUT_DIR
    jq -r ".assets | .[] | select(.name == \"marine\") | \"\(.browser_download_url) $OUT_DIR/\(.name)\"" |
    # download assets
    xargs -n2 bash -c 'curl -L $0 -o $1 && chmod +x $1'
