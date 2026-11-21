#!/bin/bash
# Fetch the tip of $BRANCH from $REPO_URL and, if it isn't deployed yet,
# build the mdbook spec into $OUT_DIR/1.0 and the tracey spec into
# $OUT_DIR/1.1. Exits 75 (EX_TEMPFAIL) if the fetch fails.
#
# Each build is written to $OUT_DIR/_builds/<sha>, and 1.0 and 1.1 are
# relative symlinks into it, replaced by rename so readers never see a
# partial build.
set -euo pipefail

: "${REPO_URL:?}" "${BRANCH:?}" "${OUT_DIR:?}" "${WORK_DIR:?}"

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

if [ "$(readlink "$OUT_DIR/1.0" || true)" = "_builds/$sha/1.0" ] &&
  [ "$(readlink "$OUT_DIR/1.1" || true)" = "_builds/$sha/1.1" ]; then
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
  MDBOOK_OUTPUT__HTML__SITE_URL=/1.0/ \
    mdbook build "$repo/spec/mdbook" -d "$stage/1.0"

  # Export starts a tracey daemon for the checkout; stop it afterwards.
  trap 'tracey kill "$repo" > /dev/null 2>&1 || true' EXIT
  (cd "$repo" && tracey export "$stage/1.1")

  # tracey export writes root-absolute links (/assets/..., /postcard/...),
  # so prefix them with the path the site is served under.
  #
  # TODO: suggest `--base-dir` to upstream tracey?
  find "$stage/1.1" -name '*.html' -exec sed -i -E \
    -e 's#(href|src)="/([^/])#\1="/1.1/\2#g' \
    -e 's#url=/([^/])#url=/1.1/\1#g' \
    {} +

  mv -T "$stage" "$builds/$sha"
fi

for ver in 1.0 1.1; do
  ln -sfn "_builds/$sha/$ver" "$OUT_DIR/.$ver.new"
  mv -T "$OUT_DIR/.$ver.new" "$OUT_DIR/$ver"
done

for old in "$builds"/*; do
  if [ "$old" != "$builds/$sha" ]; then
    rm -rf "$old"
  fi
done

echo "deploy: deployed $sha"
