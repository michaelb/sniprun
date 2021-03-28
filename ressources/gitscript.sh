#!/bin/sh


get_latest_release() {
  curl --silent "https://api.github.com/repos/michaelb/sniprun/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' # Pluck JSON value
}

local_version=v$(cat Cargo.toml | grep version | cut -d "\"" -f 2)
remote_version=$(get_latest_release)

echo -n "Version : " $local_version
if [ $local_version == $remote_version ]; then
  echo " (up-to-date)"
else
  echo  " (update to " $remote_version "is available)"
fi
