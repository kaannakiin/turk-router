import type { Address } from "@solana/addresses";

import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { DammV2Form, PROGRAM_ADDRESS, resolve: build } = venues.meteoraDammV2;

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  const referralSlot = slotAt(fixture, 11).pubkey;
  const accounts: venues.meteoraDammV2.MeteoraDammV2Accounts = {
    pool: slotAt(fixture, 1).pubkey,
    inputTokenAccount: slotAt(fixture, 2).pubkey,
    outputTokenAccount: slotAt(fixture, 3).pubkey,
    tokenAVault: slotAt(fixture, 4).pubkey,
    tokenBVault: slotAt(fixture, 5).pubkey,
    tokenAMint: slotAt(fixture, 6).pubkey,
    tokenBMint: slotAt(fixture, 7).pubkey,
    payer: slotAt(fixture, 8).pubkey,
    tokenAProgram: slotAt(fixture, 9).pubkey,
    tokenBProgram: slotAt(fixture, 10).pubkey,
    referralTokenAccount: referralSlot === PROGRAM_ADDRESS ? undefined : referralSlot,
  };
  switch (fixture.slots.length) {
    case 14:
      return build(accounts, DammV2Form.Base);
    case 15:
      return build(accounts, DammV2Form.RateLimited);
    default:
      throw new Error(
        `${fixture.poolB58}: unexpected meteora damm v2 slot count ${String(fixture.slots.length)}`,
      );
  }
}

function accountsFor(
  referralTokenAccount: Address | undefined,
): venues.meteoraDammV2.MeteoraDammV2Accounts {
  return {
    pool: key(1),
    inputTokenAccount: key(2),
    outputTokenAccount: key(3),
    tokenAVault: key(4),
    tokenBVault: key(5),
    tokenAMint: key(6),
    tokenBMint: key(7),
    payer: key(8),
    tokenAProgram: key(9),
    tokenBProgram: key(10),
    referralTokenAccount,
  };
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (const form of [DammV2Form.Base, DammV2Form.RateLimited]) {
    for (const referralTokenAccount of [undefined, key(11)]) {
      counts.push(build(accountsFor(referralTokenAccount), form).accountCount);
    }
  }
  return counts;
}
