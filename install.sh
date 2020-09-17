#!/usr/bin/env bash

set -o errexit

version="v0.2.0"
name="sniprun"

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Trying to build Sniprun locally using Cargo."
    echo "Building..."
    cargo build --release 2>/dev/null
    echo "Done"
  else
    echo "Could not find cargo in \$PATH: the Rust toolchain is required to build Sniprun"
    return 1
  fi
}

download() {
  command -v curl >/dev/null &&
    curl --fail --location "$1" --output target/release/sniprun
}

fetch_prebuilt_binary() {
  echo "Downloading binary.."
  url="not provided yet"
  echo $url
  mkdir -p target/release

  if (download "$url"); then
    chmod a+x target/release/sniprun
    return
  else
    cargo_build || echo "Prebuilt binaries are not available for this platform."
  fi
}

arch=$(uname)
echo "No pre-built binary available yet for platform: ${arch}."
cargo_build
