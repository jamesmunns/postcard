#!/bin/bash
# Fail unless every requirement has an implementation and a verification
# reference, and no reference points at an outdated requirement version.
set -euo pipefail

status="$(tracey query --json status)"
echo "$status"

# An empty impl list (e.g. a missing config) must fail
if ! jq -e '(.impls | length > 0) and all(.impls[];
      .coveredRules == .totalRules
      and .verifiedRules == .totalRules
      and .staleRules == 0)' <<< "$status" > /dev/null; then
  tracey query uncovered
  tracey query untested
  tracey query stale
  exit 1
fi
