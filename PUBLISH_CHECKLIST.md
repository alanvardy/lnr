# Publish Checklist

## Setup lnr-bin

Create lnr-bin directory for pushing to AUR

```bash
cd ~/dev
git clone ssh://aur@aur.archlinux.org/lnr-bin.git
cd lnr-bin
git remote add aur ssh://aur@aur.archlinux.org/lnr-bin.git
```

## Publish to Cargo and AUR

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Change version in Cargo.toml
2. Update dependencies and make sure nothing broke with `./update_test.sh`
3. Update CHANGELOG.md with version number
4. Update README.md with help text `cargo run -- -h`
5. Add any new examples to README.md
6. Refresh table of contents with LSP action
7. Open PR for version and wait for it to pass
8. Commit and merge PR

9. Build release

```bash
git checkout main
git pull
cargo aur
```

10. [Create a new release](https://github.com/alanvardy/lnr/releases/new)

- Make sure to use the label and title in format `v0.3.8`
- Add binary from lnr directory

11. Publish to Cargo with `cargo publish`
12. Push to AUR with `./push_aur.sh`

