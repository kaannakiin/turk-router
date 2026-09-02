import { assertNever } from "./assertNever.js";
import { FindRouteError } from "./error.js";

/**
 * A venue the menu may name; the value is the byte the wire carries.
 *
 * The program dispatches on a wider numbering; only these ten are accepted in a `find_route` menu,
 * and the others cannot be spelled here: the type is the union of these ten literals.
 */
export const HopKind = {
  RaydiumAmmV4: 0,
  Whirlpool: 1,
  RaydiumClmm: 2,
  RaydiumCpmm: 3,
  MeteoraDlmmSwap: 4,
  MeteoraDlmmSwap2: 5,
  MeteoraDammV2: 6,
  PumpSwapSell: 7,
  PumpSwapBuy: 8,
  MeteoraDammV1: 9,
} as const;

export type HopKind = (typeof HopKind)[keyof typeof HopKind];

export type HopKindName = keyof typeof HopKind;

/** Every kind, in wire order. */
export const ALL_HOP_KINDS: ReadonlyArray<HopKind> = [
  HopKind.RaydiumAmmV4,
  HopKind.Whirlpool,
  HopKind.RaydiumClmm,
  HopKind.RaydiumCpmm,
  HopKind.MeteoraDlmmSwap,
  HopKind.MeteoraDlmmSwap2,
  HopKind.MeteoraDammV2,
  HopKind.PumpSwapSell,
  HopKind.PumpSwapBuy,
  HopKind.MeteoraDammV1,
];

export function isHopKind(value: number): value is HopKind {
  return ALL_HOP_KINDS.some((kind) => kind === value);
}

/** The kind a byte names; throws `UnknownHopKind` for any other byte. */
export function hopKindFromByte(raw: number): HopKind {
  if (isHopKind(raw)) {
    return raw;
  }
  throw new FindRouteError({ kind: "UnknownHopKind", raw });
}

/** The manifest's name for a kind. */
export function hopKindName(kind: HopKind): HopKindName {
  switch (kind) {
    case HopKind.RaydiumAmmV4:
      return "RaydiumAmmV4";
    case HopKind.Whirlpool:
      return "Whirlpool";
    case HopKind.RaydiumClmm:
      return "RaydiumClmm";
    case HopKind.RaydiumCpmm:
      return "RaydiumCpmm";
    case HopKind.MeteoraDlmmSwap:
      return "MeteoraDlmmSwap";
    case HopKind.MeteoraDlmmSwap2:
      return "MeteoraDlmmSwap2";
    case HopKind.MeteoraDammV2:
      return "MeteoraDammV2";
    case HopKind.PumpSwapSell:
      return "PumpSwapSell";
    case HopKind.PumpSwapBuy:
      return "PumpSwapBuy";
    case HopKind.MeteoraDammV1:
      return "MeteoraDammV1";
    default:
      return assertNever(kind);
  }
}
