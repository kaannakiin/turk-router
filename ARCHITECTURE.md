# Architecture

This document is the map of the Rust client. The README says what the package is and is not;
this says how the crate is arranged, how one instruction is assembled, and where every number it
encodes comes from.

## What the crate is, and is not

`turk-router` builds one instruction, `find_route`, for the program named in
`wire/wire-manifest.json`. A caller hands it a list of candidate pools — each already resolved to
the accounts that pool's venue takes — and its own token accounts, and receives an `Instruction`
whose data and account list the program will accept. The program then does the rest at the moment
the instruction lands: it finds the profitable cycle among the candidates, sizes it, executes it,
and refuses the transaction if none exists.

The crate therefore makes no decision that depends on chain state. It reaches no RPC, decodes no
pool, quotes nothing, and derives none of the caller's accounts. The one address it derives is the
fee collector's token account, from constants, because the program's config stores the collector as
a wallet rather than a token account and the caller has no other way to know which account to send.

## Module layout

- `wire` — every wire number the crate encodes: the program id, the config seed, the
  discriminator, the header and menu-entry widths, the three ceilings, the two flag bits, and the
  six prefix slots with their flags. It is the only module allowed to carry a wire literal, and the
  test suite holds each one against the manifest.
- `hop_kind` — the ten venues a menu may name, as an enum whose discriminants are the wire bytes.
  The program dispatches on a wider numbering; the kinds it will not read from a menu cannot be
  constructed here.
- `venues` — one module per kind. Each exposes an accounts struct, a bounded type for any variable
  tail, and an infallible `resolve` that returns a `VenueWindow`. Only these modules can construct a
  window, and each computes the declared account count from what it holds, so the count the menu
  entry carries cannot disagree with the accounts the window appends.
- `programs` — the program and sysvar addresses several windows name in fixed positions.
- `builder` — `build_find_route_instruction`, the crate's single public entry point, with the
  parameter and flag types it takes.
- `pda` — the two derivations the builder performs offline: the config account and the fee
  collector's token account.

## One instruction, end to end

The account list has three sections, in this order.

1. **The prefix.** Six slots: the user (the signer), the user's token account for the base mint,
   the base mint, the base token program, the router's config account, and the fee collector's
   token account. The caller supplies the first two and the fee wallet; the builder fills the rest
   from constants and derivations. The fee slot is sent on every call: the program reads it only at
   a nonzero fee rate, but its position is fixed.
2. **The route mints.** One `(token program, user token account)` pair per mint the route may pass
   through. The base mint is node 0 of the graph the program searches; the `i`-th pair is node
   `i + 1`. This is why the order matters: reordering the pairs changes which cycles exist, not
   merely where an account sits.
3. **The menu.** Each window's accounts, in the order the windows were given, each beginning with
   its venue's program id. The program reads a window by the account count its menu entry declares
   and hands it to the venue's adapter, which validates the window against its own constants.

The instruction data is fixed-width: a header, then one menu entry per window, and nothing else.
The header carries the discriminator, the flags byte, the walk-step budget, the two counts and the
profit threshold. A menu entry carries the venue's kind, the window's account count, and two
transfer-hook group lengths that this crate always sends as zero — the program accepts hook account
groups on two kinds, and this crate does not build them.

Before encoding, the builder refuses what the program would refuse as malformed data: an empty or
oversized route-mint list, an empty or oversized menu, and a menu whose declared accounts sum past
the program's budget. Each refusal is a named error. Everything the crate cannot check — that the
user signs, that the token accounts are owned as declared, that a profitable cycle exists — is left
to the program, and the manifest's `router_errors` lists the names it answers with.

## Where the numbers come from

`wire/wire-manifest.json` is generated from the program's own constants and delivered to this
repository by the program's release process; nothing here fetches it. The crate does not parse the
manifest at build time. Its wire constants are declared once, in `wire`, and `manifest_agreement.rs`
holds every one of them against the manifest: the program id, the seed, the discriminator (also
recomputed from its name), the widths, the ceilings, the flag bits, the prefix names and flags, the
base mints, and — for every menu kind — the exact set of account counts the venue module can
declare against the window lengths the program accepts. A constant that drifts turns that test red;
a literal that appears anywhere else in the crate is a defect.

## How the fixtures test it

`wire/fixtures` is the program's own conformance corpus for `find_route`: one file per pool window,
each carrying the venue program, every account the window names with its writable and signer
flags, and the mint accounts. The addresses are rewritten before publication except for program
ids, token programs, the base mints and program-wide authorities, so the corpus names no live pool.

`fixture_conformance.rs` reads every file back byte for byte, then feeds each fixture's accounts
into the venue module it belongs to and compares the window the module builds against the fixture
slot by slot: length, order, address, writable, signer. Because the caller-supplied addresses are
fed from the fixture itself, what the comparison pins is the layout — the order the module emits,
the flags it sets, and the fixed addresses it supplies from its own constants.

The corpus covers every kind. A fixture is a window, not a transaction: the tests need no network,
hold no secret, and run the same on a fork's pull request as on `main`.

## Versioning

The README's versioning section is the contract. In short: the manifest's `wire_epoch` is the
crate's major version, a new venue is a minor version, and everything that leaves the wire untouched
is a patch.
