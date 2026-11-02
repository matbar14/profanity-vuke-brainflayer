#!/usr/bin/env bash
# build-all.sh — build the three programs of this toolkit:
#
#   profanity   GPU vanity generator (C++ / OpenCL)      -> ./profanity.x64
#   vuke        CPU transform generator (Rust)           -> vuke/target/release/vuke
#   brain       brainflayer-MultiBlooms matcher (C)      -> brainflayer-MultiBlooms/brainflayer
#
# Usage:
#   ./build-all.sh                    # build everything
#   ./build-all.sh vuke brain         # build only the named components
#   ./build-all.sh -j 8               # run make with 8 jobs
#
# Component names: profanity | vuke | brain (alias: brainflayer) | all

set -u
cd "$(dirname "$0")" || exit 1

JOBS="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
TARGETS=""

usage() { sed -n '2,18p' "$0"; exit 0; }

while [ $# -gt 0 ]; do
    case "$1" in
        -j)          JOBS="$2"; shift 2 ;;
        -j*)         JOBS="${1#-j}"; shift ;;
        profanity)   TARGETS="$TARGETS profanity"; shift ;;
        vuke)        TARGETS="$TARGETS vuke"; shift ;;
        brain|brainflayer) TARGETS="$TARGETS brain"; shift ;;
        all)         TARGETS="profanity vuke brain"; shift ;;
        -h|--help)   usage ;;
        *) echo "build-all.sh: unknown argument '$1' (see --help)" >&2; exit 2 ;;
    esac
done
[ -z "$TARGETS" ] && TARGETS="profanity vuke brain"

have() { command -v "$1" >/dev/null 2>&1; }
say()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }
ok()   { printf '\033[32m[ OK ]\033[0m %s\n' "$*"; }
bad()  { printf '\033[31m[FAIL]\033[0m %s\n' "$*"; }
hint() { printf '       hint: %s\n' "$*"; }

# ---------------------------------------------------------------- profanity
build_profanity() {
    say "profanity — GPU vanity generator (C++/OpenCL)"
    if ! have g++ || ! have make; then
        bad "g++/make not found"; hint "Debian/Ubuntu: apt install build-essential"; return 1
    fi
    if ! printf '#include <CL/cl.h>\nint main(void){return 0;}\n' \
        | g++ -x c++ - -o /dev/null -lOpenCL 2>/dev/null \
        && ! printf '#include <OpenCL/cl.h>\nint main(void){return 0;}\n' \
        | g++ -x c++ - -o /dev/null -framework OpenCL 2>/dev/null; then
        bad "OpenCL headers/loader not found"
        hint "Debian/Ubuntu: apt install ocl-icd-opencl-dev + your GPU vendor ICD"
        hint "macOS: Xcode command line tools provide the OpenCL framework"
        return 1
    fi
    if make -j "$JOBS"; then
        ok "built ./profanity.x64"
        return 0
    fi
    bad "profanity build failed"; return 1
}

# ---------------------------------------------------------------- vuke
build_vuke() {
    say "vuke — CPU transform generator (Rust)"
    if ! have cargo && [ -x "$HOME/.cargo/bin/cargo" ]; then
        PATH="$HOME/.cargo/bin:$PATH"   # rustup installs here but non-login shells may miss it
    fi
    if ! have cargo; then
        bad "cargo not found"; hint "install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        return 1
    fi
    if (cd vuke && cargo build --release); then
        ok "built vuke/target/release/vuke"
        return 0
    fi
    bad "vuke build failed"; return 1
}

# ---------------------------------------------------------------- brainflayer
build_brain() {
    say "brainflayer-MultiBlooms — bloom-filter matcher (C)"
    if ! have gcc || ! have make; then
        bad "gcc/make not found"; hint "Debian/Ubuntu: apt install build-essential"; return 1
    fi
    if ! { [ -f /usr/include/gmp.h ] || [ -f /usr/include/x86_64-linux-gnu/gmp.h ]; } \
        && ! printf '#include <gmp.h>\nint main(void){return 0;}\n' | cc -x c - -o /dev/null 2>/dev/null; then
        bad "GMP development headers not found"; hint "Debian/Ubuntu: apt install libgmp-dev"; return 1
    fi
    if ! printf '#include <openssl/crypto.h>\nint main(void){return 0;}\n' | cc -x c - -o /dev/null -lcrypto 2>/dev/null; then
        bad "OpenSSL development headers not found"; hint "Debian/Ubuntu: apt install libssl-dev"; return 1
    fi
    if make -C brainflayer-MultiBlooms -j "$JOBS"; then
        ok "built brainflayer-MultiBlooms/brainflayer (+ hex2blf, blfchk, ecmtabgen)"
        return 0
    fi
    bad "brainflayer build failed"; return 1
}

# ---------------------------------------------------------------- run
FAILED=""
for t in $TARGETS; do
    if ! "build_$t"; then
        FAILED="$FAILED $t"
    fi
done

printf '\n\033[1m========== SUMMARY ==========\033[0m\n'
echo "profanity : ./profanity.x64"
echo "vuke      : vuke/target/release/vuke"
echo "brain     : brainflayer-MultiBlooms/brainflayer"
if [ -n "$FAILED" ]; then
    bad "failed:$FAILED"
    printf '\nFix the missing dependencies above and re-run (only the failed parts:\n./build-all.sh%s)\n' "$FAILED"
    exit 1
fi
ok "all requested components built"
printf '\nSmoke test the pipeline:\n'
echo '  ./profanity.x64 --benchmark --stream-keys | ./brainflayer-MultiBlooms/brainflayer -x -t priv -v -b bitcoin.blf'
echo '  ./vuke/target/release/vuke generate --transform=sha256 --stream-keys wordlist --file words.txt | ./brainflayer-MultiBlooms/brainflayer -x -t priv -b bitcoin.blf'
exit 0
