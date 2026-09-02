/**
 * Feeds a fixture's slots into the venue module it belongs to and returns the window the module
 * builds; the caller compares the result to the fixture slot by slot. A port of the Rust suite's
 * `tests/common/harness`.
 */
import { assertNever } from "../../../src/assertNever.js";
import { HopKind, hopKindFromByte, type VenueWindow } from "../../../src/index.js";
import type { Fixture } from "../fixture.js";
import * as meteoraDammV1 from "./meteoraDammV1.js";
import * as meteoraDammV2 from "./meteoraDammV2.js";
import * as meteoraDlmmSwap from "./meteoraDlmmSwap.js";
import * as meteoraDlmmSwap2 from "./meteoraDlmmSwap2.js";
import * as pumpSwapBuy from "./pumpSwapBuy.js";
import * as pumpSwapSell from "./pumpSwapSell.js";
import * as raydiumAmmV4 from "./raydiumAmmV4.js";
import * as raydiumClmm from "./raydiumClmm.js";
import * as raydiumCpmm from "./raydiumCpmm.js";
import * as whirlpool from "./whirlpool.js";

export function resolve(fixture: Fixture): VenueWindow {
  const kind = hopKindFromByte(fixture.hopKind);
  switch (kind) {
    case HopKind.RaydiumAmmV4:
      return raydiumAmmV4.resolve(fixture);
    case HopKind.Whirlpool:
      return whirlpool.resolve(fixture);
    case HopKind.RaydiumClmm:
      return raydiumClmm.resolve(fixture);
    case HopKind.RaydiumCpmm:
      return raydiumCpmm.resolve(fixture);
    case HopKind.MeteoraDlmmSwap:
      return meteoraDlmmSwap.resolve(fixture);
    case HopKind.MeteoraDlmmSwap2:
      return meteoraDlmmSwap2.resolve(fixture);
    case HopKind.MeteoraDammV2:
      return meteoraDammV2.resolve(fixture);
    case HopKind.PumpSwapSell:
      return pumpSwapSell.resolve(fixture);
    case HopKind.PumpSwapBuy:
      return pumpSwapBuy.resolve(fixture);
    case HopKind.MeteoraDammV1:
      return meteoraDammV1.resolve(fixture);
    default:
      return assertNever(kind);
  }
}

/**
 * Every account count the kind's module can declare, one window per point of its parameter
 * space, built from placeholder addresses.
 */
export function reachableAccountCounts(kind: HopKind): Array<number> {
  switch (kind) {
    case HopKind.RaydiumAmmV4:
      return raydiumAmmV4.reachableAccountCounts();
    case HopKind.Whirlpool:
      return whirlpool.reachableAccountCounts();
    case HopKind.RaydiumClmm:
      return raydiumClmm.reachableAccountCounts();
    case HopKind.RaydiumCpmm:
      return raydiumCpmm.reachableAccountCounts();
    case HopKind.MeteoraDlmmSwap:
      return meteoraDlmmSwap.reachableAccountCounts();
    case HopKind.MeteoraDlmmSwap2:
      return meteoraDlmmSwap2.reachableAccountCounts();
    case HopKind.MeteoraDammV2:
      return meteoraDammV2.reachableAccountCounts();
    case HopKind.PumpSwapSell:
      return pumpSwapSell.reachableAccountCounts();
    case HopKind.PumpSwapBuy:
      return pumpSwapBuy.reachableAccountCounts();
    case HopKind.MeteoraDammV1:
      return meteoraDammV1.reachableAccountCounts();
    default:
      return assertNever(kind);
  }
}

export function assertProgram(fixture: Fixture, expected: string): void {
  if (fixture.programId !== expected) {
    throw new Error(`${fixture.poolB58}: program id ${fixture.programId}, expected ${expected}`);
  }
}
