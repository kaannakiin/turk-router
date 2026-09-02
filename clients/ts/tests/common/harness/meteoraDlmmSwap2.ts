import type { Address } from "@solana/addresses";

import { assertTailLength, venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { MAX_BIN_ARRAYS, PROGRAM_ADDRESS, resolve: build } = venues.meteoraDlmmSwap2;

function optional(pubkey: Address): Address | undefined {
  return pubkey === PROGRAM_ADDRESS ? undefined : pubkey;
}

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  // slot 13 memo, slot 14 event authority, slot 15 program-again are module constants.
  const binArrays: ReadonlyArray<Address> = fixture.slots.slice(16).map((slot) => slot.pubkey);
  assertTailLength(binArrays, 1, MAX_BIN_ARRAYS);
  return build(
    {
      pool: slotAt(fixture, 0).pubkey,
      binArrayBitmapExtension: optional(slotAt(fixture, 1).pubkey),
      reserveX: slotAt(fixture, 2).pubkey,
      reserveY: slotAt(fixture, 3).pubkey,
      userTokenIn: slotAt(fixture, 4).pubkey,
      userTokenOut: slotAt(fixture, 5).pubkey,
      tokenXMint: slotAt(fixture, 6).pubkey,
      tokenYMint: slotAt(fixture, 7).pubkey,
      oracle: slotAt(fixture, 8).pubkey,
      hostFeeIn: optional(slotAt(fixture, 9).pubkey),
      user: slotAt(fixture, 10).pubkey,
      tokenXProgram: slotAt(fixture, 11).pubkey,
      tokenYProgram: slotAt(fixture, 12).pubkey,
    },
    binArrays,
  );
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (let length = 1; length <= MAX_BIN_ARRAYS; length += 1) {
    const binArrays: ReadonlyArray<Address> = Array.from({ length }, (_, index) =>
      key(101 + index),
    );
    assertTailLength(binArrays, 1, MAX_BIN_ARRAYS);
    for (const bitmapExtension of [undefined, key(90)]) {
      for (const hostFeeIn of [undefined, key(91)]) {
        counts.push(
          build(
            {
              pool: key(1),
              binArrayBitmapExtension: bitmapExtension,
              reserveX: key(2),
              reserveY: key(3),
              userTokenIn: key(4),
              userTokenOut: key(5),
              tokenXMint: key(6),
              tokenYMint: key(7),
              oracle: key(8),
              hostFeeIn,
              user: key(9),
              tokenXProgram: key(10),
              tokenYProgram: key(11),
            },
            binArrays,
          ).accountCount,
        );
      }
    }
  }
  return counts;
}
