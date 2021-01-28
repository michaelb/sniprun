#!/usr/bin/env bash

set -o errexit

# version="v0.2.0"

echo "Runnning Sniprun Installer"
local_version=vv$(cat Cargo.toml | grep version | cut -d "\"" -f 2)
name="sniprun"
force_build=$1

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Building..."
    cargo build --release &>/dev/null
    echo "Done"
  else
    echo "Could not find cargo in \$PATH: the Rust toolchain is required to build Sniprun"
    return 1
  fi
}
get_latest_release() {
  curl --silent "https://api.github.com/repos/michaelb/sniprun/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' # Pluck JSON value
}

download() {
  # command -v curl >/dev/null &&
  # curl --fail --location "$1" --output target/release/sniprun
  rm -rf download_dir
  mkdir -p download_dir
  cd download_dir
  curl -s https://api.github.com/repos/michaelb/sniprun/releases/latest | grep "sniprun" | cut -d ":" -f 2,3 | tr -d \" | wget -qi -
  mv -f sniprun ../target/release/
  cd ..
}

fetch_prebuilt_binary() {
  echo "Downloading binary.."
  mkdir -p target/release

  if (download "$url"); then
    chmod a+x target/release/sniprun
    echo "Done"
    return
  else
    return 1
  fi
}

arch=$(uname)
if [ $arch != "Linux" ]; then
  echo "Warning, sniprun needs Linux to work properly! Any behaviour from this point is not tested"
fi

remote_version=v$(get_latest_release)

if [ $force_build ]; then
  echo "Always compiling the latest commit (most bleeding-edge option)"
  cargo_build
else
  #check if release is up to date
  if [ $local_version == $remote_version ]; then
    echo "Trying to get a up-to-date precompiled binary"
    fetch_prebuilt_binary
    success=$?
  else
    echo "Release version is not up to date, building from source"
    cargo_build
    success=$?
  fi

  # if nothing succeeded
  if [ success == 1 ]; then
    echo "Could not build (missing rust/cargo toolchain?). Getting an out-of-date release if available"
    fetch_prebuilt_binary # get an older release
  fi
fi
