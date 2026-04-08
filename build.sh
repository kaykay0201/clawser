#!/usr/bin/env bash
# build.sh — Clawser build script with graceful shutdown and cache protection.
#
# Usage:
#   ./build.sh                    # Build clawser_browser with 70% cores
#   ./build.sh chrome             # Build chrome
#   ./build.sh clawser_fetch      # Build clawser_fetch DLL
#   ./build.sh -j8 clawser_browser  # Custom thread count
#   ./build.sh --clean            # Force gn gen + build
#   ./build.sh --kill             # Kill any running build gracefully
#   ./build.sh --status           # Show build process status

# === Config ===
DEPOT_TOOLS="C:/depot_tools"
GN="buildtools/win/gn.exe"
BUILD_DIR="out/Default"
LOCKFILE="$BUILD_DIR/.build_lock"

export DEPOT_TOOLS_WIN_TOOLCHAIN=0
export GYP_MSVS_OVERRIDE_PATH="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"
export GYP_MSVS_VERSION=2022
export vs2022_install="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"

# === Helpers ===
die() { echo "ERROR: $*" >&2; exit 1; }

# Strip \r from powershell output (Windows line endings)
ps_cmd() { powershell -Command "$1" 2>/dev/null | tr -d '\r'; }

cpu_count() {
    local c
    c=$(ps_cmd "(Get-CimInstance Win32_Processor).NumberOfLogicalProcessors")
    c="${c//[^0-9]/}"
    echo "${c:-16}"
}

kill_build() {
    echo "Stopping build gracefully..."
    ps_cmd "Stop-Process -Name ninja -Force -ErrorAction SilentlyContinue" > /dev/null
    local i
    for i in $(seq 1 30); do
        local count
        count=$(ps_cmd "(Get-Process clang-cl -ErrorAction SilentlyContinue).Count")
        count="${count//[^0-9]/}"
        count="${count:-0}"
        [ "$count" = "0" ] && break
        echo "  Waiting for $count clang-cl ($i/30s)..."
        sleep 1
    done
    ps_cmd "Stop-Process -Name clang-cl -Force -ErrorAction SilentlyContinue" > /dev/null
    rm -f "$LOCKFILE"
    echo "Build stopped."
}

check_running() {
    local count
    count=$(ps_cmd "(Get-Process ninja -ErrorAction SilentlyContinue).Count")
    count="${count//[^0-9]/}"
    count="${count:-0}"
    [ "$count" != "0" ]
}

# === Parse args ===
THREADS=""
TARGET="clawser_browser"
DO_CLEAN=false
DO_KILL=false
DO_STATUS=false

while [ $# -gt 0 ]; do
    case "$1" in
        --clean)  DO_CLEAN=true; shift ;;
        --kill)   DO_KILL=true; shift ;;
        --status) DO_STATUS=true; shift ;;
        -j*)      THREADS="${1#-j}"; shift ;;
        *)        TARGET="$1"; shift ;;
    esac
done

if $DO_STATUS; then
    if check_running; then
        echo "Build RUNNING"
    else
        echo "No build running."
    fi
    exit 0
fi

$DO_KILL && { kill_build; exit 0; }

# === Pre-flight ===
cd "$(dirname "$0")" || exit 1

if check_running; then
    die "Build already running. Use './build.sh --kill' first."
fi

if [ -z "$THREADS" ]; then
    CORES=$(cpu_count)
    THREADS=$(( CORES * 70 / 100 ))
    [ "$THREADS" -lt 1 ] && THREADS=1
fi

echo "=== Clawser Build ==="
echo "Target:  $TARGET"
echo "Threads: $THREADS"
echo "Dir:     $BUILD_DIR"
echo ""

# === GN gen (if needed) ===
if $DO_CLEAN || [ ! -f "$BUILD_DIR/build.ninja" ]; then
    echo "[gn gen] Regenerating..."
    $GN gen "$BUILD_DIR"
    echo ""
fi

# Corruption check
if $DEPOT_TOOLS/ninja.exe -C "$BUILD_DIR" -n "$TARGET" 2>&1 | grep -q "premature end of file"; then
    echo "[gn gen] Ninja deps corrupted, regenerating..."
    $GN gen "$BUILD_DIR"
    echo ""
fi

# === Build ===
echo "[ninja] Building $TARGET with -j$THREADS..."
echo ""

START=$(date +%s)
$DEPOT_TOOLS/ninja.exe -C "$BUILD_DIR" "$TARGET" -j"$THREADS"
EXIT_CODE=$?
END=$(date +%s)
ELAPSED=$((END - START))

rm -f "$LOCKFILE"

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    MINS=$((ELAPSED / 60))
    SECS=$((ELAPSED % 60))
    echo "=== BUILD SUCCEEDED in ${MINS}m ${SECS}s ==="
    ls -lh "$BUILD_DIR/$TARGET.exe" 2>/dev/null || ls -lh "$BUILD_DIR/$TARGET.dll" 2>/dev/null || true
else
    echo "=== BUILD FAILED (exit $EXIT_CODE) in ${ELAPSED}s ==="
    exit $EXIT_CODE
fi
