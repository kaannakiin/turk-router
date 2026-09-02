# Architecture

This document is the map of both clients. The README says what the package is and is not; this
says how the Rust crate is arranged, how one instruction is assembled, where every number it
encodes comes from, how the TypeScript client mirrors it, and what holds the two together.

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

## The TypeScript mirror

`clients/ts` mirrors the crate module for module: `wire.rs` is `wire.ts`, `hop_kind.rs` is
`hopKind.ts`, `error.rs` is `error.ts`, `pda.rs` is `pda.ts`, `programs.rs` is `programs.ts`,
`builder.rs` is `builder.ts`, and each `venues/<kind>.rs` is `venues/<kind>.ts`. Names translate by
one rule: a Rust `snake_case` field is a TypeScript `camelCase` field, a Rust `*_ID` address is a
TypeScript `*_ADDRESS`, `HopKind::X` is `HopKind.X` with the wire byte as its value, and
`venues::x::resolve` is `venues.x.resolve`. `Option<Pubkey>` is a required key typed
`Address | undefined`, so an omitted slot is a compile error. `u64` is `bigint`; `u8` is `number`
with a range guard before any encoder runs, which is the one error the TypeScript client can raise
that the Rust one cannot.

Three places are not literal translations. A bounded tail is a union of tuple types, so a literal
outside the venue's range is a type error and the fourth Whirlpool tick array is refused by `tsc`;
`tests/types.typecheck.ts` carries those proofs as `@ts-expect-error` lines, the TypeScript form of
the Rust `compile_fail` doctest, and every `resolve` also checks the length at run time for a caller
holding an untyped array. `VenueWindow` is a class with private fields, so an object literal cannot
pass as a window and the count it declares is the length of what it holds. And
`buildFindRouteInstruction` is `async`: Kit derives a program address by awaiting
`crypto.subtle.digest`, so the config account and the fee collector's token account arrive as
promises. Nothing reaches the network.

The discipline is mirrored too. The `tsconfig.json` flags are the TypeScript half of the crate's
eight-lint deny list, `oxlint --type-aware` carries what a compiler flag cannot, `src/wire.ts` is
the only module with a wire literal, and `tests/lintPin.test.ts` pins all of it the way
`lint_pin.rs` pins the Rust list. The same `manifest_agreement` and `fixture_conformance` checks run in
TypeScript against the same `wire/` files.

## The golden corpus

`clients/golden/find_route.json` is the second corpus, and it runs the other way: where the
fixtures come from the program and pin the venue modules, the golden is written by this crate and
pins whatever else claims to build the same instruction. For a fixed sweep of typed inputs — one
window per account count the program accepts, every flag byte, every route-mint count, both base
mints, the account budget on both sides of its limit, and every error the builder raises — it
records the instruction data as hex and the account list as `address:role` strings.

`cross_language.rs` regenerates the file when `TURK_ROUTER_WRITE_GOLDEN` is set and otherwise
asserts the committed text is what the crate builds now; it also rebuilds every case from the
committed inputs alone, so the file is proven sufficient by construction. The TypeScript client
verifies the same file in `tests/crossLanguage.test.ts`, one test per case, and never writes it.
If that test is red and this crate's is green, the TypeScript client is wrong; if both are red, this
crate's output moved and the file needs regenerating. The two CI jobs read the same file, so neither
client can drift without one of them turning red.

## Versioning

The README's versioning section is the contract. In short: the manifest's `wire_epoch` is the
crate's major version, a new venue is a minor version, and everything that leaves the wire untouched
is a patch.
