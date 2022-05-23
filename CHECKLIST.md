# Before releasing

 - check compilation success
 - update Cargo.lock: `cargo update`
 - Bump Cargo.toml to next version
 - cargo fmt --all / cargo check / cargo clippy
 - update the changelog
 - create a version bump commit
 - merge
 - create a new tag vX.Y.Z on master
 - git push origin vX.Y.Z

# After release

 - Check CI status
 - Check Releases status
 - Edit release name
