# Before releasing

## on dev branch
 - check compilation success
 - update Cargo.lock: `cargo update`
 - cargo fmt --all / cargo check / cargo clippy
 - update the changelog
 - remove the 'beta' from the version field in Cargo.toml

## merge
 - dev -> master

## on master branch
 - create a new SIGNED tag vX.Y.Z on master: `git tag -s v8.9.5` (tag message should be equal to tag number, eg: v8.9.5)
 - verify the signed tag: `git tag -v v8.9.5`
 - git push origin vX.Y.Z

# After releasing

 - Check CI status
 - Check Releases status
 - Edit release name
 - Bump Cargo.toml to next version on dev, suffixed by 'beta'

