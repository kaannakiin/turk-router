# turk-router (TypeScript)

The same client as `clients/rust/turk-router`, in TypeScript. The boundary is the first paragraph of
the [repository README](../../README.md): this package builds one instruction, `find_route`, and
nothing else.

## Status

Not published. The package name is a placeholder and `package.json` is `"private": true`, so
`npm publish` is refused until the name is decided.

## Requirements

Node 22 or newer, ESM only. The package sits on `@solana/kit`'s granular packages as peer
dependencies: `@solana/addresses`, `@solana/instructions`, `@solana/codecs-core`,
`@solana/codecs-numbers`, `@solana/codecs-data-structures`. It returns a Kit `Instruction`; a caller
on `@solana/web3.js` v1 converts it with `@solana/web3-compat` on their own side.

## The API in one screen

```ts
import { BaseMint, buildFindRouteInstruction, venues } from "turk-router";

const menu = [
  venues.raydiumAmmV4.resolve({ pool, baseVault, quoteVault, userSource, userDestination, payer }),
  venues.whirlpool.resolve(whirlpoolAccounts, [supplementalTickArray]),
];

const instruction = await buildFindRouteInstruction({
  user,
  baseMint: BaseMint.Wsol,
  baseAta,
  feeWallet,
  flags: { flashloan: false, failIfNoProfit: true },
  maxWalkSteps: 0,
  minProfitBaseUnits: 1n,
  routeMints: [{ tokenProgram, userAta }],
  menu,
});
```

- `buildFindRouteInstruction` is `async` because the two addresses it derives, the config account
  and the fee collector's token account, are hashed with WebCrypto, whose API returns promises.
  Nothing reaches the network.
- The base mint is node 0 of the graph the program searches and `routeMints[i]` is node `i + 1`; the
  order of route mints is part of the input.
- Every `venues.<kind>.resolve` takes the venue's accounts as named fields, in the Rust client's
  field order with camelCase names, plus a tuple for any variable tail: `whirlpool` takes zero to
  three supplemental tick arrays, `raydiumClmm` one to seven tail accounts, both DLMM forms one to
  eight bin arrays. A literal outside the range is a type error; an array of unknown length is
  narrowed with `assertTailLength`, and `resolve` refuses a wrong length at run time as well.
- Optional accounts (`hostFeeIn`, `binArrayBitmapExtension`, `referralTokenAccount`, `poolV2`,
  `cashback`) are required keys typed `Address | undefined`, so an omitted slot is a compile error.
- Eight pools never fit: the shortest window is nine accounts and the budget is sixty-nine, so a
  menu holds at most seven.
- Refusals throw `FindRouteError`, whose `detail.kind` names the Rust error variant with the same
  fields; `IntegerOutOfRange` is the one variant the Rust client does not have, for a `maxWalkSteps`
  or `minProfitBaseUnits` outside its wire width.
- `HopKind`, `wire`, and `programs` mirror the Rust `HopKind`, `wire` and `programs` modules; a Rust
  `*_ID` is a TypeScript `*_ADDRESS`.

## Toolchain

Every non-default here has a reason, since `package.json` cannot carry comments.

- `"private": true` and the placeholder name: no publish before the name is decided.
- Peer dependencies with a `^8.2.0` range and exact-pinned dev dependencies: a consumer already
  holds Kit and must hold one copy of it (`AccountRole` is a runtime enum); the lockfile pins the
  tree CI tests, which is what `npm ci` reproduces.
- `@types/node` on the 22 line, not the newest: types track the engine floor, so code that uses an
  API Node 22 lacks fails to typecheck.
- `"files": ["dist", "README.md"]`: tests, configs and the golden corpus never enter a tarball;
  `npm pack --dry-run` in CI lists what would.
- `npm ci --ignore-scripts`: nothing in the tree needs a lifecycle script, and a dependency that
  gained one would otherwise run code on every fork's pull request.
- `tsconfig.json` is the TypeScript half of the Rust crate's eight-lint deny list; its header says
  why it exists. `oxlint --type-aware` (oxlint with `oxlint-tsgolint`) carries the rules a compiler
  flag cannot: no `any`, no non-null assertion, no narrowing `as`, exhaustive `switch`, no floating
  promise. `typescript-eslint` does not support TypeScript 7, which is why oxlint.
- Prettier at 100 columns, the width the Rust side and the program's docs use.

## Tests

`npm test` runs, with `node:test` under `tsx`:

- `manifestAgreement`: every wire constant against `wire/wire-manifest.json`, and for every menu
  kind the exact set of account counts the module can build against the lengths the program accepts.
- `fixtureConformance`: every fixture under `wire/fixtures` round-trips byte for byte, and every
  window a module builds matches its fixture slot by slot.
- `buildFindRoute`: both sides of every limit, the bytes, the account list, and the refusals only a
  TypeScript caller can provoke.
- `crossLanguage`: every case of `clients/golden/find_route.json`, the bytes and account list the
  Rust client emitted, rebuilt here and compared.
- `lintPin`: the tsconfig flags, the oxlint rules, the import allow-list, and the rule that wire
  literals live in `src/wire.ts`.
- Unit tests per module and per venue.

`tests/types.typecheck.ts` is compiled by `npm run typecheck` and never executed: its
`@ts-expect-error` lines are the TypeScript form of the Rust `compile_fail` doctests, including the
fourth Whirlpool tick array.

## How it is held to the Rust client

`clients/golden/find_route.json` is written only by the Rust crate's `cross_language.rs` and
verified by both clients in CI, so neither can drift from the other without a job turning red.
[ARCHITECTURE.md](../../ARCHITECTURE.md) has the mechanism.
