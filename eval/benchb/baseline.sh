#!/usr/bin/env bash
# Sanity check: the unmodified app passes the old-behavior baseline. Usage: baseline.sh <app dir> <lang>
. "$(dirname "$0")/lib.sh"; bb_init "$@"
bb_baseline
bb_unit
echo PASS
