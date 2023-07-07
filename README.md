# lnr

[![Build Status](https://github.com/alanvardy/lnr/workflows/ci/badge.svg)](https://github.com/alanvardy/lnr) [![codecov](https://codecov.io/gh/alanvardy/lnr/branch/main/graph/badge.svg?token=9FBJK1SU0K)](https://codecov.io/gh/alanvardy/lnr) [![Crates.io](https://img.shields.io/crates/v/lnr.svg)](https://crates.io/crates/lnr)

A Linear command line client

## Examples

Add an organization and token. Your token can be generated in [Linear Settings](https://linear.app/settings/api)

```bash
lnr org add
```

Create a new issue

```bash
lnr issue create
```

View issue (linked to current branch)

```bash
lnr issue view
```


Edit issue (linked to current branch)

```bash
lnr issue edit
```

## Installation

[Install Rust](https://www.rust-lang.org/tools/install)

```bash
# Linux and MacOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install `lnr`

```bash
cargo install lnr
```

## Contributing

Contributions are welcome, be sure to open up an issue first!