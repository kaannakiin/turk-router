import type { Address } from "@solana/addresses";

import { assertTailLength, venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { MAX_BINS, PROGRAM_ADDRESS, resolve: build } = venues.meteoraDlmmSwap;

function sentinelToOption(pubkey: Address): Address | undefined {
  return pubkey === PROGRAM_ADDRESS ? undefined : pubkey;
}

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  const bins: ReadonlyArray<Address> = fixture.slots.slice(15).map((slot) => slot.pubkey);
  assertTailLength(bins, 1, MAX_BINS);
  return build(
    {
      lbPair: slotAt(fixture, 0).pubkey,
      binArrayBitmapExtension: sentinelToOption(slotAt(fixture, 1).pubkey),
      reserveX: slotAt(fixture, 2).pubkey,
      reserveY: slotAt(fixture, 3).pubkey,
      userTokenIn: slotAt(fixture, 4).pubkey,
      userTokenOut: slotAt(fixture, 5).pubkey,
      mintX: slotAt(fixture, 6).pubkey,
      mintY: slotAt(fixture, 7).pubkey,
      oracle: slotAt(fixture, 8).pubkey,
      hostFeeIn: sentinelToOption(slotAt(fixture, 9).pubkey),
      user: slotAt(fixture, 10).pubkey,
    },
    bins,
  );
}

function accountsFor(
  bitmapPresent: boolean,
  hostFeePresent: boolean,
): venues.meteoraDlmmSwap.MeteoraDlmmSwapAccounts {
  return {
    lbPair: key(1),
    binArrayBitmapExtension: bitmapPresent ? key(2) : undefined,
    reserveX: key(3),
    reserveY: key(4),
    userTokenIn: key(5),
    userTokenOut: key(6),
    mintX: key(7),
    mintY: key(8),
    oracle: key(9),
    hostFeeIn: hostFeePresent ? key(10) : undefined,
    user: key(11),
  };
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (let binCount = 1; binCount <= MAX_BINS; binCount += 1) {
    const bins: ReadonlyArray<Address> = Array.from({ length: binCount }, () => key(20));
    assertTailLength(bins, 1, MAX_BINS);
    for (const bitmapPresent of [false, true]) {
      for (const hostFeePresent of [false, true]) {
        counts.push(build(accountsFor(bitmapPresent, hostFeePresent), bins).accountCount);
      }
    }
  }
  return counts;
}
