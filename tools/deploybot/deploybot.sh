#!/bin/bash
# Run deploy.sh every $INTERVAL seconds (default 300), forever. A failed
# fetch from GitHub is logged, and the loop carries on.
set -uo pipefail

interval="${INTERVAL:-300}"
here="$(dirname "$0")"

# As PID 1, bash gets no default SIGTERM handling, so exit explicitly.
# Sleeping in the background keeps the trap from waiting out the sleep.
trap 'exit 0' TERM INT

while true; do
  status=0
  "$here/deploy.sh" || status=$?
  case "$status" in
    0) ;;
    75) echo "$(date -u +%FT%TZ) deploybot: couldn't fetch from GitHub, retrying in ${interval}s" ;;
    *) echo "$(date -u +%FT%TZ) deploybot: deploy failed (exit $status), retrying in ${interval}s" ;;
  esac
  sleep "$interval" &
  wait $!
done
