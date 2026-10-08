# `.tosc` file format

This document describes the on-disk format of current TouchOSC (Hexler)
`.tosc` layout files, as reverse-engineered from the fixtures in
`tests/fixtures/`. Nothing here comes from official documentation — Hexler
does not publish a `.tosc` schema — so treat every statement as derived from
observed examples, not a specification. Where something is ambiguous or
under-sampled, that is called out explicitly below.

Fixtures used:

- `blank.tosc` — a single empty root `GROUP`, no children.
- `complex_setup.tosc` — one page, nested `GROUP`s containing `BUTTON`,
  `FADER`, `LABEL`, `RADIAL`, `TEXT` controls, OSC messages, and two
  embedded Lua scripts.
- `multipage.tosc` — a `PAGER` with multiple page `GROUP` children, plus
  MIDI and OSC message bindings on the pager itself.
- `everything.tosc` — one of each control type the editor exposes easily:
  `BOX`, `BUTTON`, `ENCODER` (x2), `FADER` (x4), `GRID`, `GROUP`, `LABEL`,
  `PAGER`, `RADAR`, `RADIAL`, `RADIO`, `TEXT`, `XY`.

## Container

A `.tosc` file is a raw [zlib][] stream (header bytes `78 9c` observed in all
four fixtures — no gzip/zip wrapper, no length prefix) that decompresses
directly to a UTF-8 XML document. There is no outer envelope: the entire
file is the compressed XML, start to finish.

[zlib]: https://www.rfc-editor.org/rfc/rfc1950

## XML shape

The decompressed XML as written by the editor is **unindented** (no
insignificant whitespace between tags) and uses **single-quoted**
attribute values, e.g.:

```xml
<?xml version='1.0' encoding='UTF-8'?><lexml version='5'><node ID='...' type='GROUP'>...
```

Single- vs. double-quoted attributes are not semantically distinct in XML,
and this project's round-trip tests compare parsed structure, not raw
bytes, so the writer is not required to reproduce the exact quote style or
the lack of whitespace. Everything else in this document *is* treated as
semantically significant and must round-trip exactly.

### Root element

```xml
<lexml version='5'> ... one <node> ... </lexml>
```

- `version` was `5` in every fixture. Not validated further; treat as an
  opaque attribute to preserve, not a value to branch on.
- `<lexml>` contains exactly one root `<node>` (the top-level canvas/group).

### `<node>`

```xml
<node ID='<uuid>' type='<TYPE>'>
  <properties> ... </properties>
  <values> ... </values>
  <messages> ... </messages>   <!-- optional -->
  <children> ... </children>   <!-- optional -->
</node>
```

- Attributes are exactly `ID` and `type`; nothing else was observed.
- `ID` is a UUID string. Fixtures mix UUID versions — some look like
  UUIDv1 (e.g. `c2ab52f8-c330-11f1-b92e-00e003960cda`, version nibble `1`),
  others like UUIDv4 (e.g. `d267155c-42e7-4e71-848d-c19986a821f8`, version
  nibble `4`). The editor does not appear to validate the UUID version, so
  this library generates UUIDv4 for new nodes and treats `ID` as an opaque
  string everywhere else (**assumption**, not confirmed against the editor
  internals).
- `type` observed values: `GROUP`, `PAGER`, `BOX`, `BUTTON`, `LABEL`,
  `TEXT`, `FADER`, `XY`, `RADIAL`, `ENCODER`, `RADAR`, `RADIO`, `GRID`.
  This is not necessarily the full set the editor supports — only what
  appears in `everything.tosc`.
- Section order is always `properties`, `values`, then optionally
  `messages`, then optionally `children`. `messages` and `children` are
  **omitted entirely** when empty — there is no empty `<children/>` or
  `<messages/>` form in any fixture. The generic tree model preserves
  presence/absence of these sections rather than normalizing to empty
  elements.
- A node with no sub-nodes has no `<children>` tag at all (not an empty
  one). A node with no message bindings has no `<messages>` tag at all.

### `<properties>`

An ordered list of `<property type='X'>` entries:

```xml
<property type='b'>
  <key><![CDATA[background]]></key>
  <value>1</value>
</property>
```

Observed `type` codes:

| code | meaning | value shape |
|------|---------|-------------|
| `b`  | boolean | `0` or `1` (not `true`/`false`) |
| `i`  | integer | plain integer text, e.g. `40` |
| `f`  | float   | plain numeral text — **note:** whole-number floats are written without a decimal point (`cornerRadius` of `1`, not `1.0`) |
| `s`  | string  | `<value><![CDATA[...]]></value>` |
| `c`  | color   | `<value><r>..</r><g>..</g><b>..</b><a>..</a></value>`, components as float text in `0..1` |
| `r`  | rect/frame | `<value><x>..</x><y>..</y><w>..</w><h>..</h></value>`, components as plain numbers |

No other type codes were observed. **Numeric text must be preserved
verbatim** (not reformatted/renormalized) since the generic tree stores
property values as raw text, not as parsed f64/i64 — this is required for
lossless round-trip of files that may use different numeral formatting
than this library's own writer would produce.

The `key` is always wrapped in `CDATA`, even for simple identifiers with no
special characters — the generic tree preserves this (it always emits
`key` as CDATA) rather than switching between CDATA and plain text.

Property keys are emitted by the editor in **alphabetical order** within a
node (confirmed across all four fixtures), but the parser does not assume
or enforce this — it preserves whatever order is present in the source
file, since a hand-edited or third-party-generated file is not guaranteed
to be alphabetical.

**Property meaning is contextual on node `type`**, not global: the same
key can have different semantics (and even different `type` codes)
depending on the owning node. Observed example: `gridX`/`gridY` appear as
type `i` (grid step count) on some node types and type `b` (snap-to-grid
toggle) on others, based on raw grep across fixtures — the exact type-to-
meaning mapping per node type was not exhaustively traced, only noted as a
hazard. **Assumption/gap:** the typed builder layer must look up property
type by `(node type, key)`, not by `key` alone, and should not assume a
single canonical type per key name.

Full set of property keys observed across all fixtures (see
`tests/fixtures/`, not reproduced in full here since the mapping is
per-node-type): `background`, `bar`, `barDisplay`, `buttonType`,
`centered`, `color`, `cornerRadius`, `cursor`, `cursorDisplay`,
`exclusive`, `font`, `frame`, `grabFocus`, `grid`, `gridColor`,
`gridNaming`, `gridOrder`, `gridStart`, `gridSteps`, `gridStepsX`,
`gridStepsY`, `gridType`, `gridX`, `gridY`, `interactive`, `inverted`,
`lines`, `linesDisplay`, `lockX`, `lockY`, `locked`, `name`, `orientation`,
`outline`, `outlineStyle`, `pointerPriority`, `press`, `radioType`,
`release`, `response`, `responseFactor`, `script`, `shape`, `steps`,
`tabColorOff`, `tabColorOn`, `tabLabel`, `tabLabels`, `tabbar`,
`tabbarDoubleTap`, `tabbarSize`, `tag`, `textAlignH`, `textAlignV`,
`textClip`, `textColor`, `textColorOff`, `textColorOn`, `textLength`,
`textSize`, `textSizeOff`, `textSizeOn`, `textWrap`, `valuePosition`,
`visible`.

Lua scripts are stored as an ordinary string property with key `script`
(type `s`); the code itself is the CDATA value. See [Lua scripts](#lua-scripts)
below.

### `<values>`

An ordered list of `<value>` entries describing the interactive/runtime
value(s) a control exposes:

```xml
<value>
  <key><![CDATA[touch]]></key>
  <locked>0</locked>
  <lockedDefaultCurrent>0</lockedDefaultCurrent>
  <default><![CDATA[false]]></default>
  <defaultPull>0</defaultPull>
</value>
```

Every node observed has at least a `touch` value. Additional value keys
depend on node `type`:

| node type | value keys |
|---|---|
| `GROUP`, `BOX`, `GRID` | `touch` |
| `BUTTON`, `FADER`, `RADIAL`, `RADIO` | `x`, `touch` |
| `XY`, `ENCODER`, `RADAR` | `x`, `y`, `touch` |
| `LABEL`, `TEXT` | `text`, `touch` |
| `PAGER` | `page`, `touch` |

This table is derived only from `everything.tosc` and is **not guaranteed
exhaustive** — e.g. it does not confirm whether `RADIO` exposes additional
per-segment state. `default` is always CDATA regardless of the underlying
type (booleans are stored as the literal strings `true`/`false` here,
*unlike* `b`-type properties which use `0`/`1` — the encoding is
inconsistent between `<properties>` and `<values>` and must not be
unified).

### `<messages>`

Optional. Contains zero or more `<midi>` and/or `<osc>` elements, in that
order when both are present (observed in `multipage.tosc` and
`everything.tosc`; `midi` always precedes `osc`). Multiple `<midi>`
elements can coexist on one node (one control can have several message
bindings).

Common sub-structure for both `<midi>` and `<osc>`:

```xml
<enabled>1</enabled>
<send>1</send>
<receive>1</receive>
<feedback>0</feedback>
<noDuplicates>0</noDuplicates>
<connections>1111111111</connections>
<triggers>
  <trigger>
    <var><![CDATA[x]]></var>
    <condition>ANY</condition>
  </trigger>
</triggers>
```

- `connections` is a fixed-width bitstring, 10 characters in every
  fixture, one digit per connection slot in the TouchOSC app (which
  connection target each binding applies to). Only the all-`1` value
  (`1111111111`, i.e. "all connections") was observed — the per-slot
  meaning was not exercised and is **not validated** by this library
  beyond checking it's a string of `0`/`1` of consistent width.
- `<condition>` only observed value: `ANY`. Other values (e.g. `RISE`,
  `FALL`) are documented by Hexler's manual/UI but not present in any
  fixture; treat as an open string, not a closed enum, until seen.

#### `<midi>` specific

```xml
<message>
  <type>CONTROLCHANGE</type>
  <channel>0</channel>
  <data1>0</data1>
  <data2>0</data2>
</message>
<values>
  <value><type>CONSTANT</type><key/><scaleMin>0</scaleMin><scaleMax>15</scaleMax></value>
  <value><type>INDEX</type><key/><scaleMin>0</scaleMin><scaleMax>1</scaleMax></value>
  <value><type>VALUE</type><key><![CDATA[x]]></key><scaleMin>0</scaleMin><scaleMax>127</scaleMax></value>
</values>
```

- Only MIDI message `type` observed: `CONTROLCHANGE`. Hexler's manual
  documents others (note on/off, program change, etc.) — not represented
  in fixtures, so not specifically handled, just preserved generically.
- `<values><value>` entries each have a `type` (`CONSTANT`, `INDEX`, or
  `VALUE` observed), a `key` (empty for `CONSTANT`/`INDEX`, a property/
  value name as CDATA for `VALUE`), and a `scaleMin`/`scaleMax` pair. Note
  this inner `<values>` is a *different* schema from the node-level
  `<values>` described above — same tag name, unrelated structure. Don't
  conflate them in the tree model; they're just nested XML elements that
  happen to share a name.
- Empty `key` is written as `<key/>` (self-closing) in this context, not
  `<key><![CDATA[]]></key>` — contrast with the always-CDATA `key` under
  `<property>`. The generic tree preserves this literally rather than
  normalizing empty-string representations.

#### `<osc>` specific

```xml
<path>
  <partial><type>CONSTANT</type><conversion>STRING</conversion><value><![CDATA[/]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>
  <partial><type>PROPERTY</type><conversion>STRING</conversion><value><![CDATA[name]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>
</path>
<arguments>
  <partial><type>VALUE</type><conversion>FLOAT</conversion><value><![CDATA[x]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>
</arguments>
```

- OSC addresses are built by concatenating `<path>` partials at
  runtime/save-time, not stored as a single string. A literal `/`
  separator is itself a `CONSTANT` partial. `PROPERTY` partials reference
  a node property (commonly `name` or `parent.name`) so the address
  reflects the control's name in the editor. This means **validating an
  "OSC address pattern"** means validating the reconstructed path (see
  below), not a single XML field.
- Partial `type` observed: `CONSTANT`, `PROPERTY`, `VALUE`, `INDEX`.
  `conversion` observed: `STRING`, `INTEGER`, `FLOAT`, `BOOLEAN`.
- `<arguments>` lists the OSC argument partials sent with the message,
  same partial schema as `<path>`.

### Lua scripts

Lua code is stored as a plain string property (`<property type='s'>` with
key `script`), CDATA-encoded, attached directly to the node it scripts
(observed on a `BUTTON` and a `LABEL` in `complex_setup.tosc`). There is no
dedicated `<script>` or `<lua>` XML element — it is just another property.
This means:

- Reading/writing scripts is a special case of reading/writing string
  properties, not a separate code path in the generic tree.
- The "keep scripts as separate `.lua` files, inline at build time" feature
  (goal 5) is a text-representation/build-time convenience only: in the
  YAML form, a `script` property's value can reference an external file;
  `tosc build` inlines its contents into the `script` CDATA property
  before compressing. The `.tosc`/XML layer itself has no concept of
  external files.
- No fixture contains more than one script per node, and no fixture has
  a script on a `GROUP`/`PAGER`/non-interactive node — but nothing in the
  format suggests this is disallowed; it is just unexercised.

### Pages

There is no dedicated `PAGE` node type. A "page" is an ordinary `GROUP`
node that is a direct child of a `PAGER` node. Page-specific chrome
(`tabLabel`, `tabColorOff`, `tabColorOn`) is attached to that `GROUP` as
regular properties, same mechanism as everything else — multi-page layouts
are not a structurally distinct feature, just a `PAGER`/`GROUP` idiom.

## Known gaps / unverified assumptions

- No MIDI message types besides `CONTROLCHANGE` were available to sample.
- No trigger `<condition>` besides `ANY` was available to sample.
- The `connections` bitstring's per-position meaning (which physical
  connection slot each bit maps to) is unverified; only the all-ones case
  was observed.
- UUID version is assumed not to matter to the editor (see `<node>`
  above) — not confirmed by opening a v4-only file in the real app.
- The per-`(node type, key)` property type table is built only from these
  four fixtures and is very likely missing keys/types for control
  variants not present here (e.g. less common `GRID`/`RADIO` modes).

If you hit a case not covered here while using this tool, prefer trusting
a real `.tosc` file over this document, and update this file alongside the
code.
