# Before releasing

 - check compilation success
 - update Cargo.lock: `cargo update`
 - create a version bump commit
 - merge
 - create a new tag vX.Y.Z on master
 - git push origin vX.Y.Z

# After release

 - Check CI status
 - Check Releases status
 - Bump Cargo.toml to next version
