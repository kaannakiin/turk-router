# Contributing

## What belongs here

Encoding for one instruction, `find_route`. A pull request that adds pool discovery, quote math,
route search, RPC calls, or transaction building is out of scope no matter how useful it is. The
README's opening paragraph is the boundary, and it is a product decision rather than a backlog item.

Builders for the program's other instructions are never published here.

## Numbers come from the manifest

`wire/wire-manifest.json` is generated from the deployed program's own constants. The Rust crate
declares its wire constants once, in `src/wire.rs`, and `tests/manifest_agreement.rs` holds every
one of them against the manifest — that test is the wire gate. A wire literal anywhere else in the
crate is the defect this repository is arranged to prevent, and it is the one review comment
guaranteed to block a change.

A venue module carries the addresses its venue fixes (its program id, program-wide authorities), as
byte arrays with the base58 form in the doc comment and a unit test that pins the two together. The
fixture corpus is what holds those against the program.

## Rust

Both crate roots deny the same eight lints:

- `clippy::arithmetic_side_effects`
- `clippy::cast_possible_truncation`
- `clippy::cast_possible_wrap`
- `clippy::cast_precision_loss`
- `clippy::cast_sign_loss`
- `clippy::expect_used`
- `clippy::float_arithmetic`
- `clippy::unwrap_used`

The only way to suppress one is `#[expect(clippy::…, reason = "…")]`, and the reason states the
invariant that makes the lint unreachable. Never `#[allow]`. Reach for a `Result` first.

Money math uses `checked_*` and says what it wants on overflow. Widen to `u128` before multiplying
two `u64`s. Offsets derived from input are bounds-checked before they index.

Public items are documented; the workspace denies `missing_docs`.

## Documentation

A `///` on a public item is the API contract, not a comment: say what the item is, what the caller
must supply, and — for anything that returns a `Result` — an `# Errors` section. Plain `//`
comments stay near zero; one earns its place only for a wire fact the code cannot express, a
security invariant, or a trap. Describe what is, never what was: no history, no "previously".

A venue module's doc lists its window: every slot by index, with `(writable)` and `(signer)`
markers, in exactly the order the module emits. Where the program forwards a slot without reading
it, the doc gives the position and withholds the role name until it is verified against the
venue's published interface.

Write "Token program" and "Token Extensions program", "onchain" and "offchain", and "instruction"
for an instruction — a transaction is what the caller builds around it.

## Before you open a pull request

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

CI runs the same commands and nothing else. It holds no secret and reaches no network, so it runs
identically on a fork's pull request as it does on main.

The wire gate is a test rather than a fetch. `manifest_agreement.rs` holds every constant the
clients declare against the committed `wire/wire-manifest.json`. When the deployed program's wire
moves, its own repository opens a pull request here carrying the new manifest, and that test turns
red until the clients are updated to match.

## Commit messages

`scope: description`, in the imperative. Not Conventional Commits. Explain why in the body; the
diff already shows what.

## Tests

No test may depend on network state. Conformance is measured against `wire/fixtures`. A test that
needs a live RPC to pass will not be merged.
