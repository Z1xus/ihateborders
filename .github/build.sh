#!/usr/bin/env bash
# reproducible build of dist/ihateborders.exe
set -euo pipefail

target=x86_64-pc-windows-msvc
upx_version=5.2.1
upx_package=eabc6792a347d45e945be7748423e7868fd01b0d2bcaa2f4b1031fd71ff69bda

source="$(cygpath -m "$(pwd -P)")"
cargo_home="$(cygpath -m "${CARGO_HOME:-$HOME/.cargo}")"
export RUSTFLAGS="--remap-path-prefix=$source=/src --remap-path-prefix=$cargo_home=/cargo -C target-feature=+crt-static -C link-arg=/Brepro"
export CARGO_INCREMENTAL=0
# not the link.exe of git bash
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="$(cygpath -m "$VCToolsInstallDir/bin/Hostx64/x64/link.exe")"

cargo build --release --locked --target "$target"
mkdir -p dist
cp "target/$target/release/ihateborders.exe" dist/

tools="$(mktemp -d)"
curl -fsSL -o "$tools/upx.zip" "https://github.com/upx/upx/releases/download/v$upx_version/upx-$upx_version-win64.zip"
echo "$upx_package  $tools/upx.zip" | sha256sum -c -
unzip -q "$tools/upx.zip" -d "$tools"
"$tools/upx-$upx_version-win64/upx.exe" --best --lzma dist/ihateborders.exe

{
  echo "Commit: $(git rev-parse HEAD)"
  echo "UPX: $upx_version"
  rustc -Vv
} > dist/BUILD.txt
