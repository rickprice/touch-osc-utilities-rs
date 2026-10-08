# touch-osc-utilities-rs

A Rust library and CLI for reading, writing, and building [TouchOSC](https://hexler.net/touchosc)
(`.tosc`) layouts.

## Why

TouchOSC layout files (`.tosc`) are zlib-compressed XML documents with a
nontrivial internal structure (typed properties, OSC/MIDI message bindings,
embedded Lua scripts). Hand-editing them is error-prone and easy to corrupt.
This project exists to make that editing safe and scriptable: a lossless
generic tree model as the source of truth, a diffable text representation
for inspection and editing, typed builders for common controls, and
validation that catches mistakes before you open the file in the editor.

## Status

Early development. Round-trip fidelity and the CLI are the current focus;
see `docs/FORMAT.md` for findings from real fixture files as they are added.

## Install / Build

### Cargo

```sh
cargo build --release
./target/release/tosc --help
```

### Nix

```sh
nix build
./result/bin/tosc --help

# or for a dev shell with Rust toolchain available
nix develop
```

## CLI usage

```sh
# Inspect a layout as diffable YAML
tosc dump layout.tosc > layout.yaml

# Edit layout.yaml, then rebuild a .tosc from it
tosc build layout.yaml -o layout.tosc

# Validate a layout without building
tosc validate layout.tosc
```

## Library usage

```rust
use touch_osc_core::Layout;

let layout = Layout::from_file("layout.tosc")?;
layout.to_file("out.tosc")?;
```

(Typed builder examples will be added once Step 4 lands.)

## Project layout

- `touch-osc-core/` — library crate: lossless tree model, parsing,
  serialization, typed builders, validation.
- `tosc-cli/` — thin binary crate wrapping the library.

## Disclaimer

This is an unofficial, community tool. It is not affiliated with, endorsed
by, or supported by Hexler, the maker of TouchOSC.

## License

BSD 3-Clause. See [LICENSE](LICENSE).
