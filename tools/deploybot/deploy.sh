#!/bin/bash
# Fetch the tip of $BRANCH from $REPO_URL and, if it isn't deployed
# yet, build the site into $OUT_DIR, with each spec at /<spec>/<version>/:
#
#   /index.html       an index of the specs
#   /wire/1.0/        the mdbook spec
#   /wire/1.1/        the tracey wire spec
#   /schema/0.1/      the tracey schema spec
#   /assets/          tracey's assets, shared by both tracey specs
#
# Redirects (from the old /1.0/ and /1.1/ URLs, and from /wire/ and
# /schema/ to the newest version) are in the server's nginx config.
#
# Exits 75 (EX_TEMPFAIL) if the fetch fails.
#
# Each build is written to $OUT_DIR/_builds/<sha>, and $OUT_DIR/current
# is a relative symlink to it, replaced by rename so readers never see a
# partial build. The top-level entries are fixed symlinks through current.
set -euo pipefail

: "${REPO_URL:?}" "${BRANCH:?}" "${OUT_DIR:?}" "${WORK_DIR:?}"

# The tracey specs to publish: tracey spec name, URL name, version.
TRACEY_SPECS=(
  "postcard wire 1.1"
  "postcard-schema schema 0.1"
)

repo="$WORK_DIR/repo"
builds="$OUT_DIR/_builds"

if [ ! -d "$repo/.git" ]; then
  git init -q "$repo"
  git -C "$repo" remote add origin "$REPO_URL"
fi
git -C "$repo" remote set-url origin "$REPO_URL"

if ! git -C "$repo" fetch -q --depth 1 origin "$BRANCH" 2> /dev/null; then
  exit 75
fi
sha="$(git -C "$repo" rev-parse FETCH_HEAD)"

if [ "$(readlink "$OUT_DIR/current" || true)" = "_builds/$sha" ]; then
  exit 0
fi

# A build dir only exists after completing a build, reuse it if interrupted.
if [ ! -d "$builds/$sha" ]; then
  echo "deploy: building $sha"
  git -C "$repo" checkout -q --force --detach FETCH_HEAD
  git -C "$repo" clean -qfdx

  stage="$builds/.stage-$sha"
  rm -rf "$stage"
  mkdir -p "$stage"

  # site-url is only used by 404.html, whose links must be absolute.
  MDBOOK_OUTPUT__HTML__SITE_URL=/wire/1.0/ \
    mdbook build "$repo/spec/mdbook" -d "$stage/wire/1.0"

  # Export starts a tracey daemon for the checkout; stop it afterwards.
  export="$stage/.tracey"
  trap 'tracey kill "$repo" > /dev/null 2>&1 || true' EXIT
  (cd "$repo" && tracey export "$export")

  # tracey export writes /<spec>/<impl>/ and /assets/, with root-absolute links.
  # Move each <spec>/<impl>/ to /<url>/<version>/ as it is, and point links at
  # the new path. /assets/ stays where it is.
  mv "$export/assets" "$stage/assets"
  rewrite=()
  dirs=()
  for spec in "${TRACEY_SPECS[@]}"; do
    read -r name url ver <<< "$spec"
    impls=("$export/$name"/*/)
    if [ "${#impls[@]}" -ne 1 ]; then
      echo "deploy: expected one tracey impl for $name, found ${#impls[@]}" >&2
      exit 1
    fi
    impl="$(basename "${impls[0]}")"
    mkdir -p "$stage/$url"
    mv "${impls[0]}" "$stage/$url/$ver"
    rewrite+=(-e "s#(href|src)=\"/$name/$impl/#\\1=\"/$url/$ver/#g")
    dirs+=("$stage/$url/$ver")
  done
  find "${dirs[@]}" -name '*.html' -exec sed -i -E "${rewrite[@]}" {} +
  rm -rf "$export"

  # Fail on any root-absolute link the rewrite didn't account for, such as a new
  # kind of page or asset after a tracey update.
  if unknown="$(find "${dirs[@]}" -name '*.html' -exec grep -ohE \
    '(href|src)="/[^"]*' {} + | grep -vE '="/(assets|wire|schema)/')"; then
    echo "deploy: unexpected root-absolute links in the tracey export:" >&2
    echo "$unknown" | sort -u >&2
    exit 1
  fi

  for spec in "${TRACEY_SPECS[@]}"; do
    read -r name url ver <<< "$spec"
    cat > "$stage/$url/$ver/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0; url=spec.html">
  <script>location.replace("spec.html" + location.hash)</script>
</head>
<body><p><a href="spec.html">spec.html</a></p></body>
</html>
EOF
  done

  cat > "$stage/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>postcard specifications</title>
</head>
<body>
  <h1>postcard specifications</h1>
  <ul>
    <li><a href="/wire/1.1/">The Postcard Wire Specification</a>, version 1.1 (pre-release)
      (<a href="/wire/1.0/">1.0</a>)</li>
    <li><a href="/schema/0.1/">The Postcard Schema Specification</a>, version 0.1 (draft)</li>
  </ul>
  <p><a href="https://github.com/jamesmunns/postcard">github.com/jamesmunns/postcard</a></p>
</body>
</html>
EOF

  mv -T "$stage" "$builds/$sha"
fi

ln -sfn "_builds/$sha" "$OUT_DIR/.current.new"
mv -T "$OUT_DIR/.current.new" "$OUT_DIR/current"

# These only change if the layout does. Replacing them each time is cheap, and
# the first deploy of this layout removes the old 1.0 and 1.1.
for link in index.html assets wire schema; do
  ln -sfn "current/$link" "$OUT_DIR/.$link.new"
  mv -T "$OUT_DIR/.$link.new" "$OUT_DIR/$link"
done
rm -f "$OUT_DIR/1.0" "$OUT_DIR/1.1"

for old in "$builds"/*; do
  if [ "$old" != "$builds/$sha" ]; then
    rm -rf "$old"
  fi
done

echo "deploy: deployed $sha"
