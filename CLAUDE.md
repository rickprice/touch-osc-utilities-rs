# CLAUDE.md

Guidance for Claude Code (or any AI coding assistant) working in this
repository.

## The rule

**Use this tool for all TouchOSC work. Never hand-edit a `.tosc` file's
raw XML, and never write ad-hoc scripts to poke at the zlib/XML bytes
directly.** That includes one-off "just decompress it and grep" tasks —
use `tosc dump` instead, which gives the same visibility without risking
a silent corruption. If this tool is missing a feature you need (a new
control type, a validation check, a message shape), add it here first,
then use it. The entire point of this project is that `.tosc` is no
longer something anyone — human or model — edits by hand.

If you're reading this from a different repository and need to work
with a `.tosc` file, prefer vendoring or depending on `touch-osc-core`
from here over reimplementing any part of the format.

## Architecture

```
touch-osc-core/
  src/
    container.rs   zlib decompress/compress (the .tosc file is just a
                    zlib stream containing UTF-8 XML, see docs/FORMAT.md)
    tree.rs         Layout / Node / Property / ValueEntry / Message --
                    the generic, lossless tree. SOURCE OF TRUTH.
    xml.rs          hand-written quick-xml parser/serializer, in terms
                    of the tree types above
    layout.rs       Layout::from_file/to_file/from_bytes/to_bytes
    yaml.rs         the diffable YAML text representation used by
                    `tosc dump`/`tosc build`
    controls.rs     typed builders for common controls (group, button,
                    fader, ...), built on top of Node
    messages.rs     typed builders for OSC/MIDI message bindings, built
                    on top of Message
    validate.rs     structural/content validation over a Layout
    text.rs         small CDATA/entity/float-formatting helpers shared
                    by tree.rs, xml.rs, and yaml.rs
    idgen.rs        UUID generation for new nodes
tosc-cli/
  src/main.rs       thin CLI: dump / build / validate
docs/FORMAT.md      everything known about the .tosc format, derived
                    from tests/fixtures/, with assumptions called out
tests/fixtures/     real .tosc files used by every round-trip test
```

### The generic tree is the source of truth

`Node`'s `properties`/`values`/`messages` store their content as **raw
XML text fragments** (`Property::value_raw`, `ValueEntry::body_raw`,
`Message::raw`), sliced directly out of the source document when
parsing, not reconstructed from a parsed/typed representation. This is
deliberate: it means the parser never has to *understand* a shape to
preserve it byte-for-byte. A property type this crate has never seen, an
OSC message shape from a future TouchOSC version, an unusual CDATA
encoding — all of it survives a load → save cycle untouched, because
nothing downstream of the raw slice ever normalizes it unless something
explicitly asks to.

**Everything built on top of `tree.rs` (`controls.rs`, `messages.rs`,
`validate.rs`, `yaml.rs`) must preserve this property.** Concretely:

- Typed accessors (`Property::as_bool`, `as_color`, `as_rect`, ...)
  *read* the raw text; they never replace it.
- Typed constructors (`Property::bool`, `Property::color`, ...) produce
  a new raw string in the same shape the editor would write — they're
  used by the builders in `controls.rs`/`messages.rs` to create new
  content, not to reinterpret existing content.
- `yaml.rs` converts a property/value to a friendly YAML form (bool,
  number, `{r,g,b,a}`, ...) **only after verifying** that reconstructing
  the raw XML from the decoded value reproduces the original
  byte-for-byte. If that check fails — a type code this crate doesn't
  know, an unusual encoding — it falls back to a `{ raw_xml: "..." }`
  wrapper instead of guessing. **Do not remove or weaken this check** to
  make the YAML prettier; a wrong guess here means `tosc build` silently
  corrupts data it didn't understand. If you add support for a new
  shape, add it as another verified case, not a replacement for the
  fallback.
- `validate.rs` checks are advisory on top of the tree; they never
  mutate it.

If you need to add a new property/message shape, follow the same
pattern: extend the typed layer with a new accessor/constructor/verified
YAML case, but never make the generic tree model (`tree.rs`'s struct
shapes) depend on knowing every possible property. See `docs/FORMAT.md`
for the "Known gaps" section — anything not yet observed in a fixture
should stay generic/raw until it is.

### Evidence over guessing

Every default property set in `controls.rs`, every message shape in
`messages.rs`, and every check in `validate.rs` is derived from what's
actually in `tests/fixtures/*.tosc` (documented in `docs/FORMAT.md`), not
from the Hexler manual or assumption. If you add a new control type or
property, add or extend a fixture first (see "Fixtures" below), confirm
what the real editor actually writes, and only then write the code.
Note any remaining assumption explicitly in `docs/FORMAT.md`'s "Known
gaps" section rather than presenting it as confirmed.

### Fixtures

`tests/fixtures/*.tosc` are real files saved by the TouchOSC editor, not
hand-constructed. Every round-trip/validation test runs against all of
them. If you need a fixture for a control type or feature that isn't
covered yet, ask the user to create one in the real editor rather than
hand-writing a `.tosc`/XML file — a hand-written fixture can't tell you
what the editor actually does, which defeats the purpose.

Before adding any new fixture to this (public) repository, check its
decompressed content for anything private — IP addresses, hostnames,
personal names, credentials in scripts — the same way the fixtures
already here were checked (see the "Add .tosc fixtures..." commit).

## Workflow notes

- `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D
  warnings`, and `cargo test` must all pass before committing. CI
  enforces the same three.
- `nix build` and `nix develop` should keep working; if you add a
  dependency, make sure `Cargo.lock` is committed (it is — this ships a
  binary, don't gitignore it) so the Nix build stays reproducible.
- Keep commits small and focused, matching the step/feature they
  implement, consistent with the existing history.
