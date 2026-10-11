# Advanced Password Generator (adg)

Command-line password generator written in Rust.

## Usage

```
adg <length> [groups] [-c <n>]
```

| Group | Characters |
|-------|------------|
| `U`   | Uppercase `A-Z` |
| `L`   | Lowercase `a-z` |
| `N`   | Digits `0-9` |
| `S`   | Printable ASCII symbols ``!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~`` |
| `all` | All four groups (default when none is given) |

- Length: 1 to 256.
- Groups are case-insensitive and can be combined or separate (`ULN` = `U L N` = `uln`).
- `-c <n>` generates `n` passwords (1-1000), one per line.
- At least one character from each selected group is guaranteed.
- Uses the operating system's RNG (`OsRng`), which is cryptographically secure.

## Examples

```
adg 128 all
adg 16 ULNS
adg 64 NS
adg 32 S
adg 20 -c 5
```

## Build and install

```
cargo build --release      # binary at target/release/adg
cargo install --path .     # installs 'adg' into ~/.cargo/bin
cargo test                 # runs the tests
```

When a password contains symbols, wrap it in single quotes before pasting it into a shell command.
