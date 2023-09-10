# lnr

[![Build Status](https://github.com/alanvardy/lnr/workflows/ci/badge.svg)](https://github.com/alanvardy/lnr) [![codecov](https://codecov.io/gh/alanvardy/lnr/branch/main/graph/badge.svg?token=9FBJK1SU0K)](https://codecov.io/gh/alanvardy/lnr) [![Crates.io](https://img.shields.io/crates/v/lnr.svg)](https://crates.io/crates/lnr)

A Linear command line client

## Working with issues

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

## Working with templates


### Create a series of tickets from a TOML file

Input file (uses [handlebars-style variables](https://handlebarsjs.com/))

```toml
# build_batcave.toml
[variables]
name = "Alfred"

[parent]
title = "This is a parent issue"
description = """
We need to create a batcave

Make sure to coordinate with {{name}}

See child tickets
"""

[[children]]
title = "This is a child issue for {{name}} to complete"
description = """
Figure out where to put the batcave

 - Some place dark and dingy
 - Make sure to coordinate with {{name}}
"""

[[children]]
title = "This is a second child issue that will be linked to the parent issue"
description = """
Make sure that we have enough bats

### Acceptance Criteria

- [ ] They can't bite too much
- [ ] At least a dozen
- [ ] Don't overdo it this time
"""

```

Command

```bash
lnr template evaluate --path ~/Documents/build_batcave.toml
```

### Create a series of tickets from all TOML files in a directory

When passed a directory, Linear Templater will recursively walk through the directory and all sub-directories and create tickets from all the TOML files that are not `Cargo.toml`.

Command

```bash
# Create tickets from all TOML files in the current directory
lnr template evaluate --path .
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