#!/usr/bin/env bash

set -o errexit

version="v0.2.0"
name="sniprun"

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Trying to build locally using Cargo.."
    cargo build --release
  else
    echo "Could not build binary. Your installation might be corrupt."
    return 1
  fi
}

download() {
  command -v curl >/dev/null &&
    curl --fail --location "$1" --output target/release/nvim-spotify
}

fetch_prebuilt_binary() {
  echo "Downloading binary.."
  url="not provided yet"
  echo $url
  mkdir -p target/release

  if (download "$url"); then
    chmod a+x target/release/nvim-spotify
    return
  else
    cargo_build || echo "Prebuilt binaries are not ready for this platform."
  fi
}

arch=$(uname)
echo "No pre-built binary available for ${arch}."
cargo_build
