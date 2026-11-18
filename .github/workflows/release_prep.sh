#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

# Called by the reusable release workflow with the tag being released.
TAG=$1
# Match the prefix GitHub gives source archives, so a release artifact and a
# source archive share one strip_prefix.
PREFIX="rules_buffa-${TAG:1}"
ARCHIVE="rules_buffa-${TAG}.tar.gz"

git archive --format=tar --prefix="${PREFIX}/" "${TAG}" | gzip >"${ARCHIVE}"

cat <<NOTES
## Add to your \`MODULE.bazel\` file:

\`\`\`starlark
bazel_dep(name = "rules_buffa", version = "${TAG:1}")
\`\`\`
NOTES
