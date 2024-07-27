#!/bin/sh

echo "Runnning Sniprun Installer"
local_version="v$(grep ^version Cargo.toml | cut -d "\"" -f 2)"

force_build=$1
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" = "dev" ]; then
  force_build=1
fi

arch=$(uname)
if [ "$arch" != "Linux" ] && [ "$force_build" != 1 ]; then
  echo "Looks you are not running Linux: Mac users have to compile sniprun themselves and thus need the Rust toolchain"
  force_build=1
fi

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Building sniprun from source..."
    cargo build --release 2>&1
    echo "Done (status: $?)"
    return 0
  else
    echo "Could not find cargo in \$PATH: the Rust toolchain is required to build Sniprun"
    return 1
  fi
}

get_latest_release() {
  curl --silent "https://api.github.com/repos/michaelb/sniprun/releases/latest" | LC_ALL=C tr -d "\n" | sed -e 's|^.*\("tag_name"[^,]*\).*|\1|' | cut -d'"' -f4 # sed keeps matchgroup "tag_name": "vX.Z.Z", cut -d '"' does the rest
}

# download the sniprun binary (of the specified version) from Releases
download() {
  echo "Downloading sniprun binary: $1"
  curl -fsSL "https://github.com/michaelb/sniprun/releases/download/$1/sniprun" --output sniprun
  mkdir -p target/release/
  mv -f sniprun target/release/
}

# call download, make executable, and return status
fetch_prebuilt_binary() {
  if (download "$1"); then
    chmod a+x target/release/sniprun
    echo "Done"
    return 0
  else
    return 1
  fi
}


remote_version=$(get_latest_release)

if [ "$force_build" ]; then
  echo "Compiling sniprun locally:"
  neovim_version=$(nvim --version | head -n 1 | cut -d . -f 2) # 4 -> neovim 0.4.x
  if [ "$neovim_version" = "4" ]; then
    echo "Sniprun 0.4.9 is the highest version supported on neovim 0.4.x"
    git reset --hard v0.4.9
  fi
  cargo_build
else

  tag_to_fetch=$remote_version
  neovim_version=$(nvim --version | head -n 1 | cut -d . -f 2) # 4 -> neovim 0.4.x
  if [ "$neovim_version" = "4" ]; then
    echo "Sniprun 0.4.9 is the highest version supported on neovim 0.4.x"
    git reset --hard v0.4.9
    tag_to_fetch="v0.4.9"
  fi

  fetch_prebuilt_binary "$tag_to_fetch"

  # if download failed
  if [ $? = 1 ]; then
    echo "Failed to download sniprun, check your network or build locally?"
  fi
fi
