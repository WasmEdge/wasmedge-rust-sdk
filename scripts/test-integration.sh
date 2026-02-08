#!/bin/bash
# Integration test script for WasmEdge Rust SDK
# This script installs WasmEdge, builds the SDK, and runs integration tests.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo_info "WasmEdge Rust SDK Integration Test"
echo_info "==================================="
echo_info "Project directory: ${PROJECT_DIR}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo_error "Rust/Cargo is not installed. Please install Rust first."
    echo_info "Visit https://rustup.rs/ to install Rust."
    exit 1
fi

echo_info "Rust version: $(rustc --version)"
echo_info "Cargo version: $(cargo --version)"

# Check if WasmEdge is installed
WASMEDGE_INSTALLED=false
if [ -n "${WASMEDGE_DIR}" ] && [ -d "${WASMEDGE_DIR}" ]; then
    echo_info "WASMEDGE_DIR is set to: ${WASMEDGE_DIR}"
    WASMEDGE_INSTALLED=true
elif [ -d "${HOME}/.wasmedge" ] && [ -d "${HOME}/.wasmedge/lib" ]; then
    echo_info "WasmEdge found in ~/.wasmedge"
    export WASMEDGE_DIR="${HOME}/.wasmedge"
    WASMEDGE_INSTALLED=true
elif [ -f "/usr/local/lib/libwasmedge.so" ] || [ -f "/usr/local/lib/libwasmedge.dylib" ]; then
    echo_info "WasmEdge found in /usr/local"
    export WASMEDGE_DIR="/usr/local"
    WASMEDGE_INSTALLED=true
elif command -v wasmedge &> /dev/null; then
    WASMEDGE_VERSION=$(wasmedge --version 2>/dev/null | head -1)
    echo_info "WasmEdge found: ${WASMEDGE_VERSION}"
    # Try to determine WASMEDGE_DIR from wasmedge binary location
    WASMEDGE_BIN=$(which wasmedge)
    export WASMEDGE_DIR=$(dirname $(dirname "${WASMEDGE_BIN}"))
    echo_info "WASMEDGE_DIR set to: ${WASMEDGE_DIR}"
    WASMEDGE_INSTALLED=true
fi

# Install WasmEdge if not found
if [ "${WASMEDGE_INSTALLED}" = false ]; then
    echo_warn "WasmEdge is not installed. Installing..."

    # Try to install WasmEdge
    if curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh -o /tmp/wasmedge-install.sh; then
        bash /tmp/wasmedge-install.sh -p "${HOME}/.wasmedge" || {
            echo_error "Failed to install WasmEdge"
            exit 1
        }
        export WASMEDGE_DIR="${HOME}/.wasmedge"
        echo_info "WasmEdge installed to ${WASMEDGE_DIR}"
    else
        echo_error "Failed to download WasmEdge installer"
        exit 1
    fi
fi

# Change to project directory
cd "${PROJECT_DIR}"

echo_info ""
echo_info "Building the SDK..."
echo_info "-------------------"

# Build the project
cargo build --release 2>&1 | while IFS= read -r line; do
    if [[ "$line" == *"Compiling"* ]]; then
        echo -e "${GREEN}[BUILD]${NC} $line"
    elif [[ "$line" == *"warning"* ]]; then
        echo -e "${YELLOW}[BUILD]${NC} $line"
    elif [[ "$line" == *"error"* ]]; then
        echo -e "${RED}[BUILD]${NC} $line"
    elif [[ "$line" == *"Finished"* ]]; then
        echo -e "${GREEN}[BUILD]${NC} $line"
    fi
done

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo_error "Build failed!"
    exit 1
fi

echo_info ""
echo_info "Building test-wasm module..."
echo_info "----------------------------"

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo_info "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build test-wasm module
cd "${PROJECT_DIR}/test-wasm"
cargo build --release --target wasm32-unknown-unknown 2>&1 | while IFS= read -r line; do
    if [[ "$line" == *"Compiling"* ]]; then
        echo -e "${GREEN}[BUILD]${NC} $line"
    elif [[ "$line" == *"warning"* ]]; then
        echo -e "${YELLOW}[BUILD]${NC} $line"
    elif [[ "$line" == *"error"* ]]; then
        echo -e "${RED}[BUILD]${NC} $line"
    elif [[ "$line" == *"Finished"* ]]; then
        echo -e "${GREEN}[BUILD]${NC} $line"
    fi
done

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo_error "test-wasm build failed!"
    exit 1
fi

cd "${PROJECT_DIR}"

echo_info ""
echo_info "Running integration tests..."
echo_info "---------------------------"

# Set library path based on OS
if [[ "$(uname)" == "Darwin" ]]; then
    export DYLD_LIBRARY_PATH="${WASMEDGE_DIR}/lib:${DYLD_LIBRARY_PATH}"
else
    export LD_LIBRARY_PATH="${WASMEDGE_DIR}/lib:${LD_LIBRARY_PATH}"
fi

# Run integration tests
TEST_OUTPUT=$(mktemp)
cargo test --test integration_test --release -- --nocapture 2>&1 | tee "${TEST_OUTPUT}"
TEST_EXIT_CODE=${PIPESTATUS[0]}

echo_info ""
echo_info "Test Results Summary"
echo_info "===================="

# Parse test output
PASSED=$(grep -c "test .* ok$" "${TEST_OUTPUT}" 2>/dev/null || echo "0")
FAILED=$(grep -c "test .* FAILED$" "${TEST_OUTPUT}" 2>/dev/null || echo "0")
IGNORED=$(grep -c "test .* ignored$" "${TEST_OUTPUT}" 2>/dev/null || echo "0")

echo_info "Passed:  ${PASSED}"
if [ "${FAILED}" -gt 0 ]; then
    echo_error "Failed:  ${FAILED}"
else
    echo_info "Failed:  ${FAILED}"
fi
echo_info "Ignored: ${IGNORED}"

# Cleanup
rm -f "${TEST_OUTPUT}"

if [ ${TEST_EXIT_CODE} -eq 0 ]; then
    echo_info ""
    echo_info "============================================"
    echo -e "${GREEN}[SUCCESS]${NC} All integration tests passed!"
    echo_info "============================================"
    exit 0
else
    echo_info ""
    echo_info "============================================"
    echo_error "Some tests failed. Please check the output above."
    echo_info "============================================"
    exit 1
fi
