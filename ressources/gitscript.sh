#!/bin/sh


get_latest_release() {
  curl --silent "https://api.github.com/repos/michaelb/sniprun/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' # Pluck JSON value
}

local_version=v$(cat Cargo.toml | grep -m 1 version | cut -d "\"" -f 2)
remote_version=$(get_latest_release)

branch_name="$(git symbolic-ref HEAD 2>/dev/null)"

echo -n "Version : " $local_version
if [ $local_version == $remote_version ]; then
  echo -n " (up-to-date)"
elif [[ "$local_version" == *"beta"* ]];then
  echo -n  " (latest stable is $remote_version)"
else
  echo -n  " (update to " $remote_version "is available)"
fi


if [[ "$branch_name" == *"dev" ]]; then
    commit=$(git rev-parse --short HEAD)
    echo -n " dev branch, git HEAD: $commit"
else
    # on main branch, only show version
    echo ""
fi
