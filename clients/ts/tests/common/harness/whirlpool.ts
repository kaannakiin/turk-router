import type { Address } from "@solana/addresses";

import { assertTailLength, venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.whirlpool;

const BASE_SLOT_COUNT = 15;

function placeholderAccounts(): venues.whirlpool.WhirlpoolAccounts {
  return {
    tokenProgramA: key(1),
    tokenProgramB: key(2),
    tokenAuthority: key(3),
    whirlpool: key(4),
    mintA: key(5),
    mintB: key(6),
    tokenOwnerAccountA: key(7),
    tokenVaultA: key(8),
    tokenOwnerAccountB: key(9),
    tokenVaultB: key(10),
    tickArray0: key(11),
    tickArray1: key(12),
    tickArray2: key(13),
    oracle: key(14),
  };
}

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  const supplemental: ReadonlyArray<Address> = fixture.slots
    .slice(BASE_SLOT_COUNT)
    .map((slot) => slot.pubkey);
  assertTailLength(supplemental, 0, 3);
  return build(
    {
      tokenProgramA: slotAt(fixture, 0).pubkey,
      tokenProgramB: slotAt(fixture, 1).pubkey,
      tokenAuthority: slotAt(fixture, 3).pubkey,
      whirlpool: slotAt(fixture, 4).pubkey,
      mintA: slotAt(fixture, 5).pubkey,
      mintB: slotAt(fixture, 6).pubkey,
      tokenOwnerAccountA: slotAt(fixture, 7).pubkey,
      tokenVaultA: slotAt(fixture, 8).pubkey,
      tokenOwnerAccountB: slotAt(fixture, 9).pubkey,
      tokenVaultB: slotAt(fixture, 10).pubkey,
      tickArray0: slotAt(fixture, 11).pubkey,
      tickArray1: slotAt(fixture, 12).pubkey,
      tickArray2: slotAt(fixture, 13).pubkey,
      oracle: slotAt(fixture, 14).pubkey,
    },
    supplemental,
  );
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (let length = 0; length <= 3; length += 1) {
    const supplemental: ReadonlyArray<Address> = Array.from({ length }, (_, index) =>
      key(20 + index),
    );
    assertTailLength(supplemental, 0, 3);
    counts.push(build(placeholderAccounts(), supplemental).accountCount);
  }
  return counts;
}
