# Contributing

## What belongs here

Encoding for one instruction, `find_route`. A pull request that adds pool discovery, quote math,
route search, RPC calls, or transaction building is out of scope no matter how useful it is. The
README's opening paragraph is the boundary, and it is a product decision rather than a backlog item.

Builders for the program's other instructions are never published here.

## Numbers come from the manifest

`wire/wire-manifest.json` is generated from the deployed program's own constants. Read every wire
number from it. A literal copied into source is the defect this repository is arranged to prevent,
and it is the one review comment guaranteed to block a change.

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

## Before you open a pull request

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test
```

CI runs the same four commands and a fifth: it pulls the newest wire manifest published by the
program's repository and diffs it against the committed copy. A red diff there means the program's
wire moved and this repository has not caught up yet.

## Commit messages

`scope: description`, in the imperative. Not Conventional Commits. Explain why in the body; the
diff already shows what.

## Tests

No test may depend on network state. Conformance is measured against `wire/fixtures`. A test that
needs a live RPC to pass will not be merged.
