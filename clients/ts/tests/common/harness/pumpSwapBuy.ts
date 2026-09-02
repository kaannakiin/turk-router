import type { Address } from "@solana/addresses";

import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.pumpSwapBuy;

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  // The closing pair is always last; whatever sits between slot 22 and it is the optionals,
  // cashback (writable) before pool_v2 (readonly), told apart by the flag when only one is
  // present.
  const optionals = fixture.slots.slice(23, fixture.slots.length - 2);
  const [first, second] = optionals;
  let cashback: Address | undefined;
  let poolV2: Address | undefined;
  switch (optionals.length) {
    case 0:
      break;
    case 1:
      if (first?.writable === true) {
        cashback = first.pubkey;
      } else {
        poolV2 = first?.pubkey;
      }
      break;
    case 2:
      cashback = first?.pubkey;
      poolV2 = second?.pubkey;
      break;
    default:
      throw new Error(`${fixture.poolB58}: ${String(optionals.length)} optional tail slots`);
  }
  return build({
    pool: slotAt(fixture, 0).pubkey,
    user: slotAt(fixture, 1).pubkey,
    forwardedBeforeBaseMint: slotAt(fixture, 2).pubkey,
    baseMint: slotAt(fixture, 3).pubkey,
    quoteMint: slotAt(fixture, 4).pubkey,
    baseTokenAccount: slotAt(fixture, 5).pubkey,
    quoteTokenAccount: slotAt(fixture, 6).pubkey,
    baseVault: slotAt(fixture, 7).pubkey,
    quoteVault: slotAt(fixture, 8).pubkey,
    forwardedBeforeVolumeAccumulator: [
      slotAt(fixture, 9).pubkey,
      slotAt(fixture, 10).pubkey,
      slotAt(fixture, 11).pubkey,
      slotAt(fixture, 12).pubkey,
      slotAt(fixture, 13).pubkey,
      slotAt(fixture, 14).pubkey,
      slotAt(fixture, 15).pubkey,
      slotAt(fixture, 16).pubkey,
      slotAt(fixture, 17).pubkey,
      slotAt(fixture, 18).pubkey,
    ],
    userVolumeAccumulator: slotAt(fixture, 20).pubkey,
    forwardedClose: [
      slotAt(fixture, fixture.slots.length - 2).pubkey,
      slotAt(fixture, fixture.slots.length - 1).pubkey,
    ],
    poolV2,
    cashback,
  });
}

function base(): venues.pumpSwapBuy.PumpSwapBuyAccounts {
  return {
    pool: key(1),
    user: key(2),
    forwardedBeforeBaseMint: key(3),
    baseMint: key(4),
    quoteMint: key(5),
    baseTokenAccount: key(6),
    quoteTokenAccount: key(7),
    baseVault: key(8),
    quoteVault: key(9),
    forwardedBeforeVolumeAccumulator: [
      key(10),
      key(11),
      key(12),
      key(13),
      key(14),
      key(15),
      key(16),
      key(17),
      key(18),
      key(19),
    ],
    userVolumeAccumulator: key(20),
    forwardedClose: [key(21), key(22)],
    poolV2: undefined,
    cashback: undefined,
  };
}

export function reachableAccountCounts(): Array<number> {
  return [
    build(base()).accountCount,
    build({ ...base(), poolV2: key(30) }).accountCount,
    build({ ...base(), cashback: key(31) }).accountCount,
    build({ ...base(), poolV2: key(30), cashback: key(31) }).accountCount,
  ];
}
