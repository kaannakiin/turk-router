import type { Address } from "@solana/addresses";

import { assertTailLength, venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.raydiumClmm;

function placeholderAccounts(): venues.raydiumClmm.RaydiumClmmAccounts {
  return {
    payer: key(1),
    ammConfig: key(2),
    pool: key(3),
    inputTokenAccount: key(4),
    outputTokenAccount: key(5),
    inputVault: key(6),
    outputVault: key(7),
    observationState: key(8),
    inputMint: key(9),
    outputMint: key(10),
  };
}

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  const tail: ReadonlyArray<Address> = fixture.slots.slice(13).map((slot) => slot.pubkey);
  assertTailLength(tail, 1, 7);
  return build(
    {
      payer: slotAt(fixture, 0).pubkey,
      ammConfig: slotAt(fixture, 1).pubkey,
      pool: slotAt(fixture, 2).pubkey,
      inputTokenAccount: slotAt(fixture, 3).pubkey,
      outputTokenAccount: slotAt(fixture, 4).pubkey,
      inputVault: slotAt(fixture, 5).pubkey,
      outputVault: slotAt(fixture, 6).pubkey,
      observationState: slotAt(fixture, 7).pubkey,
      inputMint: slotAt(fixture, 11).pubkey,
      outputMint: slotAt(fixture, 12).pubkey,
    },
    tail,
  );
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (let length = 1; length <= 7; length += 1) {
    const tail: ReadonlyArray<Address> = Array.from({ length }, (_, index) => key(100 + index));
    assertTailLength(tail, 1, 7);
    counts.push(build(placeholderAccounts(), tail).accountCount);
  }
  return counts;
}
