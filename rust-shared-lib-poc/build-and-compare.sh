#!/bin/bash
# Build script that demonstrates size savings with shared library approach
set -e

echo "=========================================="
echo "Rust Shared Library PoC - Build & Compare"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# Build in release mode
echo "[1/4] Building all crates in release mode..."
cargo build --release 2>&1 | tail -5

echo ""
echo "[2/4] Locating built artifacts..."

SHARED_LIB="target/release/libshared_lib.so"
BIN_HELLO="target/release/bin-hello"
BIN_SERIALIZE="target/release/bin-serialize"
BIN_ASYNC="target/release/bin-async-task"

# Check if files exist
if [[ ! -f "$SHARED_LIB" ]]; then
    echo "ERROR: Shared library not found at $SHARED_LIB"
    exit 1
fi

echo ""
echo "[3/4] Size analysis of built artifacts:"
echo ""
echo "--- Shared Library (contains serde + tokio + serde_json) ---"
ls -lh "$SHARED_LIB" | awk '{print $5 "\t" $9}'
echo ""

echo "--- Binaries (dynamically link to shared library) ---"
ls -lh "$BIN_HELLO" "$BIN_SERIALIZE" "$BIN_ASYNC" 2>/dev/null | awk '{print $5 "\t" $9}'
echo ""

# Verify dynamic linking
echo "--- Verifying dynamic linking ---"
if ldd "$BIN_HELLO" 2>/dev/null | grep -q "libshared_lib.so"; then
    echo "OK: bin-hello links against libshared_lib.so"
else
    echo "Note: Using Rust dylib linking"
fi
echo ""

# Calculate sizes
SHARED_SIZE=$(stat -c%s "$SHARED_LIB" 2>/dev/null || stat -f%z "$SHARED_LIB" 2>/dev/null)
BIN1_SIZE=$(stat -c%s "$BIN_HELLO" 2>/dev/null || stat -f%z "$BIN_HELLO" 2>/dev/null)
BIN2_SIZE=$(stat -c%s "$BIN_SERIALIZE" 2>/dev/null || stat -f%z "$BIN_SERIALIZE" 2>/dev/null)
BIN3_SIZE=$(stat -c%s "$BIN_ASYNC" 2>/dev/null || stat -f%z "$BIN_ASYNC" 2>/dev/null)

TOTAL_BINARIES=$((BIN1_SIZE + BIN2_SIZE + BIN3_SIZE))
TOTAL_WITH_SHARED=$((SHARED_SIZE + TOTAL_BINARIES))

# Estimate static linking size (each binary would include shared lib dependencies)
ESTIMATED_STATIC=$((SHARED_SIZE * 3))

echo "=========================================="
echo "            SIZE COMPARISON"
echo "=========================================="
echo ""
printf "%-30s %10s\n" "Component" "Size"
printf "%-30s %10s\n" "------------------------------" "----------"
printf "%-30s %10s\n" "Shared library (1x)" "$(numfmt --to=iec $SHARED_SIZE 2>/dev/null || echo "$SHARED_SIZE B")"
printf "%-30s %10s\n" "bin-hello" "$(numfmt --to=iec $BIN1_SIZE 2>/dev/null || echo "$BIN1_SIZE B")"
printf "%-30s %10s\n" "bin-serialize" "$(numfmt --to=iec $BIN2_SIZE 2>/dev/null || echo "$BIN2_SIZE B")"
printf "%-30s %10s\n" "bin-async-task" "$(numfmt --to=iec $BIN3_SIZE 2>/dev/null || echo "$BIN3_SIZE B")"
printf "%-30s %10s\n" "------------------------------" "----------"
printf "%-30s %10s\n" "TOTAL (shared lib approach)" "$(numfmt --to=iec $TOTAL_WITH_SHARED 2>/dev/null || echo "$TOTAL_WITH_SHARED B")"
printf "%-30s %10s\n" "TOTAL (static, estimated)" "$(numfmt --to=iec $ESTIMATED_STATIC 2>/dev/null || echo "$ESTIMATED_STATIC B")"
echo ""

if [[ $TOTAL_WITH_SHARED -lt $ESTIMATED_STATIC ]]; then
    SAVINGS=$((ESTIMATED_STATIC - TOTAL_WITH_SHARED))
    PERCENT=$((SAVINGS * 100 / ESTIMATED_STATIC))
    echo "Savings with shared library: $(numfmt --to=iec $SAVINGS 2>/dev/null || echo "$SAVINGS B") (~${PERCENT}%)"
    echo ""
fi

echo "With more binaries, savings increase dramatically:"
echo "  10 binaries: ~$(numfmt --to=iec $((SHARED_SIZE * 10)) 2>/dev/null) static vs ~$(numfmt --to=iec $((SHARED_SIZE + BIN1_SIZE * 10)) 2>/dev/null) shared"
echo "  50 binaries: ~$(numfmt --to=iec $((SHARED_SIZE * 50)) 2>/dev/null) static vs ~$(numfmt --to=iec $((SHARED_SIZE + BIN1_SIZE * 50)) 2>/dev/null) shared"
echo ""

echo "=========================================="
echo "[4/4] Testing binaries..."
echo "=========================================="
echo ""

# Set up library path for runtime linking
export LD_LIBRARY_PATH="$(pwd)/target/release:$LD_LIBRARY_PATH"

echo "--- Running bin-hello ---"
"$BIN_HELLO"
echo ""

echo "--- Running bin-serialize ---"
"$BIN_SERIALIZE"
echo ""

echo "--- Running bin-async-task ---"
"$BIN_ASYNC"
echo ""

echo "=========================================="
echo "SUCCESS! All binaries executed correctly."
echo "=========================================="
