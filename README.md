# touch-osc-utilities-rs

A Rust library and CLI for reading, writing, and building [TouchOSC](https://hexler.net/touchosc)
(`.tosc`) layouts.

## Why

TouchOSC layout files (`.tosc`) are zlib-compressed XML documents with a
nontrivial internal structure (typed properties, OSC/MIDI message bindings,
embedded Lua scripts). Hand-editing them is error-prone and easy to corrupt.
This project exists to make that editing safe and scriptable: a lossless
generic tree model as the source of truth, a diffable YAML text
representation for inspection and editing, typed builders for common
controls, and validation that catches mistakes before you open the file in
the editor.

## Status

Functional. Round-trip fidelity, the YAML text representation, typed
control builders, OSC/MIDI message builders, validation, and external
`.lua` script files are all implemented and tested against real `.tosc`
fixtures (see `tests/fixtures/` and `docs/FORMAT.md`). Not yet published
to crates.io.

## Using with Claude Code

If you want Claude to use this tool when editing `.tosc` files in some
*other* project, don't clone this repo into that project. Instead copy
the skill at [`.claude/skills/touchosc/SKILL.md`](.claude/skills/touchosc/SKILL.md)
into that project's `.claude/skills/touchosc/SKILL.md` (or into
`~/.claude/skills/touchosc/SKILL.md` to make it available everywhere).
It tells Claude to clone and build this repo on demand, then use the
`tosc` CLI instead of hand-editing `.tosc` XML.

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

# or for a dev shell with the Rust toolchain available
nix develop
```

## CLI usage

```sh
# Inspect a layout as diffable YAML
tosc dump layout.tosc > layout.yaml

# Edit layout.yaml, then rebuild a .tosc from it (validates first)
tosc build layout.yaml -o layout.tosc

# ...or build anyway even if validation finds errors
tosc build layout.yaml -o layout.tosc --force

# Validate a layout without building
tosc validate layout.tosc
```

A `script` property's value can reference an external `.lua` file
instead of inline text, resolved relative to the YAML file's directory:

```yaml
- type: s
  key: script
  value:
    file: scripts/on_init.lua
```

`tosc build` inlines the file's contents before compressing. `tosc dump`
always inlines the actual script text (there's no record of an original
file path once a layout has been loaded from a `.tosc`).

## Library usage

Loading, editing, and saving a layout:

```rust
use touch_osc_core::Layout;

let mut layout = Layout::from_file("layout.tosc")?;
layout.root.children.push(/* ... */);
layout.to_file("out.tosc")?;
```

Building a layout from scratch with the typed control/message builders
(see `touch-osc-core/examples/demo_layout.rs` for the full version):

```rust
use touch_osc_core::{controls, validate};
use touch_osc_core::messages::{osc_binding, Conversion};
use touch_osc_core::tree::{Layout, Node, Rect};

let root: Node = controls::group("mixer")
    .frame(Rect::new(0.0, 0.0, 300.0, 300.0))
    .child(
        controls::fader("level")
            .frame(Rect::new(20.0, 20.0, 60.0, 200.0))
            .message(osc_binding("x", Conversion::Float, true)),
    )
    .build();

let layout = Layout { lexml_version: "5".to_string(), root };
assert!(!validate::has_errors(&validate::validate(&layout)));
layout.to_file("mixer.tosc")?;
```

## Project layout

- `touch-osc-core/` — library crate: lossless tree model (`tree`), zlib
  container handling (`container`), XML parsing/serialization (`xml`),
  the YAML text representation (`yaml`), typed control builders
  (`controls`), OSC/MIDI message builders (`messages`), and validation
  (`validate`).
- `tosc-cli/` — thin binary crate wrapping the library (`dump`, `build`,
  `validate` subcommands).
- `docs/FORMAT.md` — everything this project knows about the `.tosc`
  format, derived from the real fixtures in `tests/fixtures/`.
- `CLAUDE.md` — architecture notes and rules for AI coding assistants
  working in this repo.

## Disclaimer

This is an unofficial, community tool. It is not affiliated with, endorsed
by, or supported by Hexler, the maker of TouchOSC.

## License

BSD 3-Clause. See [LICENSE](LICENSE).
