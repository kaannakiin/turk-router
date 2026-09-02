import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.raydiumCpmm;

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  return build({
    user: slotAt(fixture, 0).pubkey,
    ammConfig: slotAt(fixture, 2).pubkey,
    pool: slotAt(fixture, 3).pubkey,
    inputTokenAccount: slotAt(fixture, 4).pubkey,
    outputTokenAccount: slotAt(fixture, 5).pubkey,
    inputVault: slotAt(fixture, 6).pubkey,
    outputVault: slotAt(fixture, 7).pubkey,
    inputTokenProgram: slotAt(fixture, 8).pubkey,
    outputTokenProgram: slotAt(fixture, 9).pubkey,
    inputMint: slotAt(fixture, 10).pubkey,
    outputMint: slotAt(fixture, 11).pubkey,
    observationState: slotAt(fixture, 12).pubkey,
  });
}

export function reachableAccountCounts(): Array<number> {
  return [
    build({
      user: key(1),
      ammConfig: key(2),
      pool: key(3),
      inputTokenAccount: key(4),
      outputTokenAccount: key(5),
      inputVault: key(6),
      outputVault: key(7),
      inputTokenProgram: key(8),
      outputTokenProgram: key(9),
      inputMint: key(10),
      outputMint: key(11),
      observationState: key(12),
    }).accountCount,
  ];
}
