# Publish Checklist

## Publish to Cargo and AUR

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Change version in Cargo.toml
2. Update dependencies and make sure nothing broke with `./update_test.sh`
3. Update CHANGELOG.md with version number
4. Update README.md with help text `cargo run -- -h`
5. Add any new examples to README.md
7. Open PR for version and wait for it to pass
8. Commit and merge PR


```bash
gs && cargo publish
```


