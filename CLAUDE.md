# CLAUDE.md

This file will document the architecture of this project and the rules
future Claude Code sessions must follow when working on TouchOSC layouts.

Status: stub, to be completed in Step 6. At minimum it will state:

- The generic, lossless tree (`touch-osc-core`) is the source of truth for
  `.tosc` files. The typed layer (builders, validation) sits on top of it
  and must never discard unknown data.
- Future sessions should use this tool (library + `tosc` CLI) for all
  TouchOSC work and must never hand-edit raw `.tosc` XML.
