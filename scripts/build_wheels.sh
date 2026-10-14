#!/usr/bin/env bash
# Build release wheels for every supported platform from one macOS machine.
#
#   scripts/build_wheels.sh            # → dist/*.whl
#
# One-time setup:
#   rustup target add x86_64-apple-darwin x86_64-unknown-linux-gnu \
#       aarch64-unknown-linux-gnu x86_64-pc-windows-msvc
#   uv pip install --python .venv/bin/python maturin 'ziglang<0.14'
#   cargo install cargo-xwin --locked
#
# Linux wheels cross-compile with zig (manylinux2014); Windows uses cargo-xwin, with zig's
# dlltool generating python3.lib. The wheels are abi3: one per platform covers Python 3.9+.
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="$PWD/.venv/bin:$HOME/.cargo/bin:$PATH"
export ZIG_COMMAND="$PWD/.venv/bin/python -m ziglang"
OUT=dist
rm -rf "$OUT"

maturin build --release --out "$OUT" --target universal2-apple-darwin
maturin build --release --out "$OUT" --target x86_64-unknown-linux-gnu --zig --compatibility manylinux2014
maturin build --release --out "$OUT" --target aarch64-unknown-linux-gnu --zig --compatibility manylinux2014
maturin build --release --out "$OUT" --target x86_64-pc-windows-msvc

ls -1 "$OUT"
