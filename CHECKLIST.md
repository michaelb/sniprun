# External contributions
 - only accept contributions to 'dev' branch OR release immediately after
 (doc contribution can be exempted)

# Prepare the release

## on dev branch
 - check compilation success
 - cargo fmt --all / cargo check / cargo clippy
 - update the changelog
 - remove the 'beta' from the version field in Cargo.toml
 - `cargo update --offline` # update sniprun's version in committed Cargo.lock

## Merge process
 - create a PR dev -> master
 - merge

## Post-merge (tag creation and push MUST be done one after the other!! DON'T add a commit in the meantime)

 - git pull the changes in master
 - create a new SIGNED tag vX.Y.Z on master: `git tag -s v8.9.5` (tag message should be equal to tag number, eg: v8.9.5)
 - verify the signed tag: `git tag -v v8.9.5`
 - git push origin vX.Y.Z

 - merge master in dev
 - Bump Cargo.toml to next version on dev, suffixed by 'beta'

# When the tag gets pushed, the 'release' github action will automatically add the new tag to GitHub's 'Releases'

# Check the release

 - Check CI status
 - Check Releases status
 - Edit release description if necessary
