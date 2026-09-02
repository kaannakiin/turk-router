# The golden corpus

`find_route.json` records, for a fixed sweep of inputs, the instruction data and account list the
Rust client emits from `build_find_route_instruction`. Both clients verify it in CI: the Rust crate
in `clients/rust/turk-router/tests/cross_language.rs`, the TypeScript package in
`clients/ts/tests/crossLanguage.test.ts`. Neither can drift from the other without one job turning
red.

Only the Rust test writes the file:

```sh
TURK_ROUTER_WRITE_GOLDEN=1 cargo test -p turk-router --test cross_language
```

Commit the regenerated file in the same pull request as the change that moved it, and read the
diff: the switch makes drift visible, not impossible. A hand edit is refused in review.

Addresses are base58, instruction bytes are hex, and the `u64` is a decimal string, so the file
carries no numeric array the publish gate could mistake for a keypair. Every address is synthetic.
