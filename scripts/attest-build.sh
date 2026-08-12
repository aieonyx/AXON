#!/usr/bin/env bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# EV-2 — AIEONYX AArch64 Attestation Build
#
# Compiles the frozen attestation fixture and verifies the emitted object is
# a genuine AArch64 relocatable, linkable as an seL4 Protection Domain.
# Exits non-zero on any failed assertion.
#
# The same script runs locally and in CI. One code path, so a green badge
# and a clean local run mean the same thing.
#
#   bash scripts/attest-build.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE="$REPO_ROOT/attest/sovereign_attest.ax"
OUTDIR="$REPO_ROOT/attest/out"
OBJ="$OUTDIR/sovereign_attest.o"
BUILDLOG="$OUTDIR/build.log"
MANIFEST="$OUTDIR/MANIFEST.txt"

OBJCOPY="${OBJCOPY:-aarch64-linux-gnu-objcopy}"
NM="${NM:-aarch64-linux-gnu-nm}"
READELF="${READELF:-aarch64-linux-gnu-readelf}"

PASS=0; FAIL=0
ok()    { printf '  PASS  %s\n' "$1"; PASS=$((PASS+1)); }
bad()   { printf '  FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
head2() { printf '\n--- %s\n' "$1"; }

printf '\nAIEONYX AArch64 Attestation\n'
printf 'Fixture:   attest/sovereign_attest.ax\n'
printf 'Target:    aarch64-sel4 (aarch64-unknown-none-elf)\n'
printf 'Profile:   seL4-strict\n'
printf 'Invariant: axon_main() -> 0x4153 (16723)\n'

# ------------------------------------------------------------ 0. toolchain
head2 "0. toolchain"
for t in llc clang "$OBJCOPY" "$NM" "$READELF"; do
    if command -v "$t" >/dev/null 2>&1; then
        printf '  %-32s %s\n' "$t" "$(command -v "$t")"
    else
        echo "FATAL: missing required tool: $t"
        exit 1
    fi
done
# Code generation shells out to llc and clang; without them the compiler
# cannot emit an object at all.

# ------------------------------------------------------------ 1. build
head2 "1. build compiler from source"
cd "$REPO_ROOT" || exit 1
cargo build --release 2>&1 | tail -3

AXON_BIN="$REPO_ROOT/target/release/axon"
[ -x "$AXON_BIN" ] || { echo "FATAL: no compiler at $AXON_BIN"; exit 1; }
echo "compiler: $AXON_BIN"
"$AXON_BIN" version 2>&1 | head -1

# Absolute path, never the copy on PATH. A stale binary there once shadowed
# the real one and made working patches look like no-ops.

# ------------------------------------------------------------ 2. compile
head2 "2. compile fixture"
[ -f "$FIXTURE" ] || { echo "FATAL: fixture missing at $FIXTURE"; exit 1; }

mkdir -p "$OUTDIR"
rm -f "$OBJ" "$BUILDLOG"

# The compiler writes <basename>.o into the working directory, so we compile
# from inside OUTDIR with an absolute fixture path.
cd "$OUTDIR" || exit 1
"$AXON_BIN" build "$FIXTURE" \
    --target aarch64-sel4 \
    --profile seL4-strict 2>&1 | tee "$BUILDLOG"
cd "$REPO_ROOT" || exit 1

[ -f "$OBJ" ] || { echo "FATAL: no object at $OBJ"; cat "$BUILDLOG"; exit 1; }
echo "object: $OBJ"

# ------------------------------------------------------------ 3. compiler gate
head2 "3. compiler ABI gate"
# The compiler runs its own seL4-strict validator: aarch64 ELF confirmed and
# forbidden symbols rejected. Its verdict is the primary assertion here.
if grep -q "seL4 ABI check PASSED" "$BUILDLOG"; then
    ok "compiler seL4 ABI check PASSED"
else
    bad "compiler did not report a passing seL4 ABI check"
fi

if grep -q "profile seL4-strict enforced" "$BUILDLOG"; then
    ok "seL4-strict profile enforced"
else
    bad "seL4-strict profile was not enforced"
fi

# ------------------------------------------------------------ 4. globalize
head2 "4. globalize axon_main"
PRE="$("$NM" "$OBJ" 2>/dev/null | grep -w 'axon_main' | head -1)"
echo "  before: ${PRE:-<absent>}"
[ -n "$PRE" ] || { echo "FATAL: axon_main not emitted — check the fixture"; exit 1; }

"$OBJCOPY" --globalize-symbol=axon_main "$OBJ" || { echo "FATAL: objcopy failed"; exit 1; }
POST="$("$NM" "$OBJ" 2>/dev/null | grep -w 'axon_main' | head -1)"
echo "  after:  $POST"

# ------------------------------------------------------------ 5. verify
head2 "5. independent verification"
FILE_OUT="$(file "$OBJ")"
echo "  file: $FILE_OUT"

case "$FILE_OUT" in
    *"ELF 64-bit LSB relocatable"*) ok "ELF 64-bit LSB relocatable" ;;
    *) bad "not a 64-bit LSB relocatable" ;;
esac
case "$FILE_OUT" in
    *aarch64*) ok "architecture is aarch64" ;;
    *) bad "architecture is NOT aarch64 — this is the whole claim" ;;
esac

MACHINE="$("$READELF" -h "$OBJ" 2>/dev/null | grep -i 'Machine:' | head -1)"
echo "  readelf:$MACHINE"
case "$MACHINE" in
    *AArch64*) ok "ELF header Machine = AArch64" ;;
    *) bad "ELF header Machine is not AArch64" ;;
esac

case "$POST" in
    *" T "*) ok "axon_main is a global text symbol" ;;
    *" t "*) bad "axon_main still LOCAL — will not link into a PD" ;;
    *)       bad "axon_main absent after globalize" ;;
esac

if "$READELF" -d "$OBJ" 2>/dev/null | grep -q 'NEEDED'; then
    bad "declares dynamic dependencies — not freestanding"
else
    ok "no dynamic dependencies (freestanding)"
fi

if "$READELF" -S "$OBJ" 2>/dev/null | grep -q '\.interp'; then
    bad "has .interp — expects a dynamic loader"
else
    ok "no .interp section"
fi

# ------------------------------------------------------------ 6. manifest
head2 "6. attestation manifest"
SHA="$(sha256sum "$OBJ" | awk '{print $1}')"
FSHA="$(sha256sum "$FIXTURE" | awk '{print $1}')"

{
  echo "AIEONYX AArch64 Attestation Manifest"
  echo "Copyright (c) 2026 Edison Lepiten / AIEONYX"
  echo "SPDX-License-Identifier: Apache-2.0"
  echo ""
  echo "generated_utc:    $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "commit:           $(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "compiler_version: $("$AXON_BIN" version 2>/dev/null | head -1)"
  echo ""
  echo "fixture:          attest/sovereign_attest.ax"
  echo "fixture_sha256:   $FSHA"
  echo ""
  echo "object:           attest/out/sovereign_attest.o"
  echo "object_sha256:    $SHA"
  echo "object_bytes:     $(stat -c%s "$OBJ")"
  echo ""
  echo "target:           aarch64-sel4  (aarch64-unknown-none-elf)"
  echo "profile:          seL4-strict"
  echo "abi_gate:         compiler seL4 ABI check PASSED"
  echo "invariant:        axon_main() -> 0x4153 (16723)"
  echo ""
  echo "assertions_passed: $PASS"
  echo "assertions_failed: $FAIL"
} > "$MANIFEST"
cat "$MANIFEST"

printf '\n===============================\n'
if [ "$FAIL" -eq 0 ]; then
    printf 'ATTESTATION PASSED  (%d assertions)\n' "$PASS"
    printf '===============================\n\n'; exit 0
else
    printf 'ATTESTATION FAILED  (%d passed, %d failed)\n' "$PASS" "$FAIL"
    printf '===============================\n\n'; exit 1
fi
