import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.raydiumAmmV4;

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  return build({
    pool: slotAt(fixture, 1).pubkey,
    baseVault: slotAt(fixture, 3).pubkey,
    quoteVault: slotAt(fixture, 4).pubkey,
    userSource: slotAt(fixture, 5).pubkey,
    userDestination: slotAt(fixture, 6).pubkey,
    payer: slotAt(fixture, 7).pubkey,
  });
}

export function reachableAccountCounts(): Array<number> {
  return [
    build({
      pool: key(1),
      baseVault: key(2),
      quoteVault: key(3),
      userSource: key(4),
      userDestination: key(5),
      payer: key(6),
    }).accountCount,
  ];
}
