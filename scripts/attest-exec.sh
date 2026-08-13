#!/usr/bin/env bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# EV-3.5 — AIEONYX AArch64 Execution Proof
#
# scripts/attest-build.sh proves the emitted object has the correct shape:
# AArch64, freestanding, global entry symbol. It does not prove the object
# computes anything. This script closes that gap by executing the compiled
# code and asserting the sovereign proof invariant.
#
# The attested object is never modified. A copy is linked against a minimal
# C harness and run under qemu-aarch64 user-mode emulation. The attested
# object's hash is re-checked afterwards to prove it was untouched.
#
# Run after scripts/attest-build.sh:
#   bash scripts/attest-exec.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTDIR="$REPO_ROOT/attest/out"
OBJ="$OUTDIR/sovereign_attest.o"
MANIFEST="$OUTDIR/MANIFEST.txt"
WORK="$OUTDIR/exec"

CC="${CC_AARCH64:-aarch64-linux-gnu-gcc}"
QEMU="${QEMU_AARCH64:-qemu-aarch64}"

EXPECT=16723          # 0x4153 — AIEONYX sovereign proof invariant

PASS=0; FAIL=0
ok()    { printf '  PASS  %s\n' "$1"; PASS=$((PASS+1)); }
bad()   { printf '  FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
head2() { printf '\n--- %s\n' "$1"; }

printf '\nAIEONYX AArch64 Execution Proof\n'
printf 'Object:    attest/out/sovereign_attest.o (copy)\n'
printf 'Emulator:  qemu-aarch64 (user mode)\n'
printf 'Expected:  axon_main() = 16723 (0x4153)\n'

# ------------------------------------------------------------ 0. preflight
head2 "0. preflight"
[ -f "$OBJ" ] || { echo "FATAL: no attested object. Run scripts/attest-build.sh first."; exit 1; }

for t in "$CC" "$QEMU"; do
    if command -v "$t" >/dev/null 2>&1; then
        printf '  %-28s %s\n' "$t" "$(command -v "$t")"
    else
        echo "FATAL: missing $t"
        echo "  sudo apt-get install -y gcc-aarch64-linux-gnu qemu-user"
        exit 1
    fi
done

SHA_BEFORE="$(sha256sum "$OBJ" | awk '{print $1}')"
echo "  attested object sha256: $SHA_BEFORE"

# ------------------------------------------------------------ 1. harness
head2 "1. build harness against a copy"
rm -rf "$WORK"; mkdir -p "$WORK"
cp "$OBJ" "$WORK/subject.o"          # the attested object is never touched

cat > "$WORK/harness.c" <<'CEOF'
/* Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 *
 * Minimal harness. Calls the AXONYX-compiled entry point and reports its
 * return value. Contributes no arithmetic of its own -- the value printed
 * is produced entirely by compiled .ax code.
 */
#include <stdio.h>

extern int axon_main(void);

int main(void) {
    int r = axon_main();
    printf("axon_main() = %d (0x%X)\n", r, r);
    return (r == 16723) ? 0 : 1;
}
CEOF

"$CC" -static -o "$WORK/harness" "$WORK/harness.c" "$WORK/subject.o" 2>&1 | tail -10
if [ ! -x "$WORK/harness" ]; then
    echo "FATAL: link failed"
    exit 1
fi
ok "linked against aarch64 harness"
file "$WORK/harness" | sed 's/^/  /'

# ------------------------------------------------------------ 2. execute
head2 "2. execute under qemu-aarch64"
OUTPUT="$("$QEMU" "$WORK/harness" 2>&1)"
RC=$?
echo "  output: $OUTPUT"
echo "  exit:   $RC"

case "$OUTPUT" in
    *"= $EXPECT "*) ok "axon_main() returned $EXPECT" ;;
    *)              bad "expected $EXPECT, got: $OUTPUT" ;;
esac

case "$OUTPUT" in
    *"0x4153"*) ok "sovereign proof invariant 0x4153 confirmed" ;;
    *)          bad "0x4153 not present in output" ;;
esac

if [ "$RC" -eq 0 ]; then
    ok "harness exit code 0"
else
    bad "harness exit code $RC"
fi

# ------------------------------------------------------------ 3. integrity
head2 "3. attested object integrity"
SHA_AFTER="$(sha256sum "$OBJ" | awk '{print $1}')"
if [ "$SHA_BEFORE" = "$SHA_AFTER" ]; then
    ok "attested object unmodified ($SHA_AFTER)"
else
    bad "attested object CHANGED — execution proof is invalid"
fi

# ------------------------------------------------------------ 4. record
head2 "4. append to manifest"
if [ -f "$MANIFEST" ]; then
    {
      echo ""
      echo "-- execution proof (EV-3.5) --"
      echo "emulator:         qemu-aarch64 user-mode"
      echo "harness_output:   $OUTPUT"
      echo "harness_exit:     $RC"
      echo "object_unchanged: $SHA_AFTER"
      echo "exec_passed:      $PASS"
      echo "exec_failed:      $FAIL"
    } >> "$MANIFEST"
    echo "  appended to attest/out/MANIFEST.txt"
fi

printf '\n===============================\n'
if [ "$FAIL" -eq 0 ]; then
    printf 'EXECUTION PROOF PASSED  (%d assertions)\n' "$PASS"
    printf '===============================\n\n'; exit 0
else
    printf 'EXECUTION PROOF FAILED  (%d passed, %d failed)\n' "$PASS" "$FAIL"
    printf '===============================\n\n'; exit 1
fi
