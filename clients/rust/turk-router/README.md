# turk-router

Builds one Solana instruction: `find_route`, for the Turk Router program at
`TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am`.

Nothing else. It does not discover pools, decode pool state, fetch accounts from the chain, quote a
price, search for a profitable cycle, choose or size a trade amount, derive or create your
associated token accounts, verify token ownership, check whether the router is paused, pick an
address lookup table, build or sign or send a transaction, or simulate anything. Choosing which
pools to offer is the caller's job; finding the profitable cycle among them is the program's, at
landing time. This crate stands between the two.

```rust
use turk_router::{build_find_route_instruction, venues, BaseMint, FindRouteFlags, FindRouteParams,
                  RouteMint};

// Choosing the menu is your job; the client only turns each pool into its window.
let menu = [venues::raydium_cpmm::resolve(cpmm_accounts)];

let instruction = build_find_route_instruction(&FindRouteParams {
    user,
    base_mint: BaseMint::Wsol,
    base_ata,
    fee_wallet,
    flags: FindRouteFlags { flashloan: false, fail_if_no_profit: true },
    max_walk_steps: 0,
    min_profit_base_units: 1,
    route_mints: &[RouteMint { token_program, user_ata }],
    menu: &menu,
})?;
// Sign and send with your own stack; this crate stops here.
```

The window each venue module builds is compared slot by slot against a committed fixture corpus,
and the account counts it can declare are held against the wire manifest the program generates.
Venue windows and their slots: [VENUES.md](https://github.com/kaannakiin/turk-router/blob/main/VENUES.md).
Architecture and the rest of the API:
[ARCHITECTURE.md](https://github.com/kaannakiin/turk-router/blob/main/ARCHITECTURE.md).

Licensed under Apache-2.0.
