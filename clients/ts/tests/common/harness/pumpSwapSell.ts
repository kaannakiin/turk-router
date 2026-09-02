import type { Address } from "@solana/addresses";

import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.pumpSwapSell;

interface Tail {
  readonly cashback: readonly [Address, Address] | undefined;
  readonly poolV2: Address | undefined;
  readonly forwardedClose: readonly [Address, Address];
}

function splitTail(fixture: Fixture): Tail {
  const tail = fixture.slots.slice(21).map((slot) => slot.pubkey);
  const [a, b, c, d, e] = tail;
  if (a === undefined || b === undefined) {
    throw new Error(`${fixture.poolB58}: pump swap sell tail shorter than the closing pair`);
  }
  switch (tail.length) {
    case 2:
      return { cashback: undefined, poolV2: undefined, forwardedClose: [a, b] };
    case 3:
      if (c === undefined) break;
      return { cashback: undefined, poolV2: a, forwardedClose: [b, c] };
    case 4:
      if (c === undefined || d === undefined) break;
      return { cashback: [a, b], poolV2: undefined, forwardedClose: [c, d] };
    case 5:
      if (c === undefined || d === undefined || e === undefined) break;
      return { cashback: [a, b], poolV2: c, forwardedClose: [d, e] };
    default:
      break;
  }
  throw new Error(
    `${fixture.poolB58}: unexpected pump swap sell tail length ${String(tail.length)}`,
  );
}

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  const { cashback, poolV2, forwardedClose } = splitTail(fixture);
  return build({
    pool: slotAt(fixture, 0).pubkey,
    user: slotAt(fixture, 1).pubkey,
    forwardedBeforeBaseMint: slotAt(fixture, 2).pubkey,
    baseMint: slotAt(fixture, 3).pubkey,
    quoteMint: slotAt(fixture, 4).pubkey,
    baseAta: slotAt(fixture, 5).pubkey,
    quoteAta: slotAt(fixture, 6).pubkey,
    baseVault: slotAt(fixture, 7).pubkey,
    quoteVault: slotAt(fixture, 8).pubkey,
    forwardedBeforeFeeConfig: [
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
    cashback,
    poolV2,
    forwardedClose,
  });
}

function accountsFor(
  cashback: readonly [Address, Address] | undefined,
  poolV2: Address | undefined,
): venues.pumpSwapSell.PumpSwapSellAccounts {
  return {
    pool: key(1),
    user: key(2),
    forwardedBeforeBaseMint: key(3),
    baseMint: key(4),
    quoteMint: key(5),
    baseAta: key(6),
    quoteAta: key(7),
    baseVault: key(8),
    quoteVault: key(9),
    forwardedBeforeFeeConfig: [
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
    cashback,
    poolV2,
    forwardedClose: [key(20), key(21)],
  };
}

export function reachableAccountCounts(): Array<number> {
  const counts: Array<number> = [];
  for (const cashback of [undefined, [key(31), key(32)] as const]) {
    for (const poolV2 of [undefined, key(30)]) {
      counts.push(build(accountsFor(cashback, poolV2)).accountCount);
    }
  }
  return counts;
}
