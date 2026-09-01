//! Builds the `find_route` instruction, and nothing else.
//!
//! The instruction takes three account sections in order: a six-slot prefix (the user, the base
//! token account, the base mint, the base token program, the router's config account and the fee
//! collector's token account), then one `(token program, user token account)` pair per route mint,
//! then the venue windows that make up the menu. [`build_find_route_instruction`] lays them out;
//! the [`venues`] modules build each window so that its declared account count cannot disagree
//! with the accounts it carries.
//!
//! Every wire number this crate encodes lives in [`wire`], and the test suite holds each one
//! against `wire/wire-manifest.json`, which is generated from the deployed program's own
//! constants. No other module carries a wire literal.
//!
//! This crate discovers no pools, decodes no pool state, quotes no price, searches no cycle, sizes
//! no amount, derives none of the caller's token accounts, and builds no transaction. Those are
//! the caller's job or the program's.

// The same eight lints the program's shipped crates deny at their roots. A client that silently
// wraps or truncates a wire number produces an instruction the program will reject at best, and
// misread at worst. The only suppression vocabulary is `#[expect(clippy::…, reason = "…")]`.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::expect_used,
    clippy::float_arithmetic,
    clippy::unwrap_used
)]
// Tests may unwrap and count freely; a failed unwrap there is a failed test, not an aborted
// transaction.
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::arithmetic_side_effects
    )
)]
#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]

mod builder;
mod error;
mod hop_kind;
mod pda;
pub mod programs;
pub mod venues;
pub mod wire;

pub use builder::{
    build_find_route_instruction, BaseMint, FindRouteFlags, FindRouteParams, RouteMint,
};
pub use error::Error;
pub use hop_kind::HopKind;
pub use venues::VenueWindow;

pub use solana_instruction;
pub use solana_pubkey;

/// The wire revision this crate encodes for. A consumer pins it; a mismatch against the deployed
/// program means the two disagree about the instruction's shape.
pub const WIRE_EPOCH: u64 = 2;
