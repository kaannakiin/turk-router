//! Builds the `find_route` instruction, and nothing else.
//!
//! The venue modules and `build_find_route_instruction` are not written yet. What is fixed already
//! is the contract they will encode against: `wire/wire-manifest.json` at the repository root is
//! generated from the deployed program's own constants, and every number this crate needs is read
//! from it rather than transcribed.

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

/// The wire revision this crate encodes for. A consumer pins it; a mismatch against the deployed
/// program means the two disagree about the instruction's shape.
pub const WIRE_EPOCH: u64 = 2;
