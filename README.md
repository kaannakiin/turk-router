# turk-router

This package builds ONE instruction: `find_route`. Nothing else. It does not discover pools, decode
pool state, fetch accounts from the chain, quote a price, search for a profitable cycle, choose or
size a trade amount, derive or create your associated token accounts, verify token ownership, check
whether the router is paused, pick an address lookup table, build or sign or send a transaction, or
simulate anything. Those are either your job (choosing the menu, supplying your own accounts) or the
program's, at the moment the instruction lands. The SDK stands between the two.

## Why the boundary sits there

Everything past "which pools are candidates" already happens onchain. Which pair of pools forms a
profitable cycle, at what price it fills, how much to enter with: `find_route` decides all of it
when the instruction lands, against live pool state, in closed form. The program trusts no offchain
quote. So the pitch is narrow on purpose: hand it the candidates and your own token accounts, and it
finds the cycle and executes it atomically. No quote on your side, no price decision that depends on
an RPC.

What is left for a client is turning candidates into bytes without adding a decision: encode the
20-byte header and a 4-byte menu entry per pool, lay out the remaining account list in the order the
program reads it, and refuse input that already violates the budget before it costs a transaction.

## What is here today

The wire contract and the Rust client that encodes it.

`wire/wire-manifest.json` is generated from the deployed program's own constants: the instruction
discriminator, the header and menu-entry widths, the account-count ceilings, the flag bits, the
six-slot account prefix with its signer and writable flags, the two base mints, the menu-eligible
hop kinds with the window lengths each venue accepts, the config PDA seed, and every `RouterError`
with its number. Every constant the Rust client declares is held against it by a test.

`wire/fixtures` is a corpus of synthetic `find_route` windows covering every menu kind, published
under rewritten addresses. The pool state in them is synthesized, and the identities they were
planted against are replaced before publication — only program ids, token programs, the base mints
and program-wide authorities survive — so nothing here names a live pool.

`clients/rust/turk-router` builds the instruction: `build_find_route_instruction` lays out the
prefix, the route mints and the menu, and one module per venue turns a pool's accounts into a
window whose declared account count cannot disagree with what it carries. Every window a module
builds is compared slot by slot against the fixture corpus, and the set of account counts each
module can declare is compared against the manifest. [ARCHITECTURE.md](ARCHITECTURE.md) is the map.
The crate is not yet published to crates.io. `clients/ts` is where the TypeScript client will live.

## Which program this targets

`TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am`, wire epoch 2. The program ID and the epoch are both
fields of `wire/wire-manifest.json`; read them from there rather than from this paragraph, which can
go stale.

The router's own source is not public. What is published is the wire contract it accepts, which is
what a caller needs and all a caller needs.

## Building and testing

```sh
cargo build
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --all --check
```

No test here reaches the network. Conformance is measured against the committed fixture corpus, the
same discipline the program's own offline test tier uses.

## Versioning

MAJOR tracks `wire_epoch`: when the program changes a byte a caller sends, the epoch moves and so
does the major version. MINOR adds capability, a new venue most often. PATCH is everything that
leaves the wire untouched.

An older client does not silently misbehave against a newer program. Solana checks the account count
itself, so a stale instruction is refused with a clean error rather than executed against the wrong
accounts.
