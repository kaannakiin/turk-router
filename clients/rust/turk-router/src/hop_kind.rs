use crate::Error;

/// A venue the menu may name. The discriminant is the byte the wire carries.
///
/// The program dispatches on a wider numbering; only these ten are accepted in a `find_route`
/// menu, and the others cannot be constructed here. Deliberately not `#[non_exhaustive]`: the set
/// is the contract, and a `match` over it should break when a kind is added.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HopKind {
    /// Raydium AMM v4.
    RaydiumAmmV4 = 0,
    /// Orca Whirlpool.
    Whirlpool = 1,
    /// Raydium concentrated liquidity (CLMM).
    RaydiumClmm = 2,
    /// Raydium constant-product (CPMM).
    RaydiumCpmm = 3,
    /// Meteora DLMM, the `swap` instruction.
    MeteoraDlmmSwap = 4,
    /// Meteora DLMM, the `swap2` instruction.
    MeteoraDlmmSwap2 = 5,
    /// Meteora DAMM v2.
    MeteoraDammV2 = 6,
    /// PumpSwap, selling the base token.
    PumpSwapSell = 7,
    /// PumpSwap, buying the base token.
    PumpSwapBuy = 8,
    /// Meteora DAMM v1.
    MeteoraDammV1 = 9,
}

impl HopKind {
    /// Every kind, in wire order.
    pub const ALL: [HopKind; 10] = [
        HopKind::RaydiumAmmV4,
        HopKind::Whirlpool,
        HopKind::RaydiumClmm,
        HopKind::RaydiumCpmm,
        HopKind::MeteoraDlmmSwap,
        HopKind::MeteoraDlmmSwap2,
        HopKind::MeteoraDammV2,
        HopKind::PumpSwapSell,
        HopKind::PumpSwapBuy,
        HopKind::MeteoraDammV1,
    ];

    /// The byte the menu entry carries.
    #[must_use]
    pub const fn discriminant(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for HopKind {
    type Error = Error;

    fn try_from(raw: u8) -> Result<Self, Error> {
        HopKind::ALL
            .into_iter()
            .find(|kind| kind.discriminant() == raw)
            .ok_or(Error::UnknownHopKind { raw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_are_dense_from_zero() {
        for (index, kind) in HopKind::ALL.iter().enumerate() {
            assert_eq!(usize::from(kind.discriminant()), index);
            assert_eq!(HopKind::try_from(kind.discriminant()), Ok(*kind));
        }
    }

    #[test]
    fn bytes_past_the_menu_set_are_refused() {
        for raw in 10u8..=u8::MAX {
            assert_eq!(HopKind::try_from(raw), Err(Error::UnknownHopKind { raw }));
        }
    }
}
