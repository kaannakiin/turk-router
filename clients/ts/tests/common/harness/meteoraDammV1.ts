import { venues, type VenueWindow } from "../../../src/index.js";
import { slotAt, type Fixture } from "../fixture.js";
import { key } from "../keys.js";
import { assertProgram } from "./index.js";

const { PROGRAM_ADDRESS, resolve: build } = venues.meteoraDammV1;

export function resolve(fixture: Fixture): VenueWindow {
  assertProgram(fixture, PROGRAM_ADDRESS);
  return build({
    pool: slotAt(fixture, 0).pubkey,
    userSource: slotAt(fixture, 1).pubkey,
    userDest: slotAt(fixture, 2).pubkey,
    aVault: slotAt(fixture, 3).pubkey,
    bVault: slotAt(fixture, 4).pubkey,
    aTokenVault: slotAt(fixture, 5).pubkey,
    bTokenVault: slotAt(fixture, 6).pubkey,
    aVaultLpMint: slotAt(fixture, 7).pubkey,
    bVaultLpMint: slotAt(fixture, 8).pubkey,
    aVaultLp: slotAt(fixture, 9).pubkey,
    bVaultLp: slotAt(fixture, 10).pubkey,
    protocolTokenFee: slotAt(fixture, 11).pubkey,
    payer: slotAt(fixture, 12).pubkey,
  });
}

export function reachableAccountCounts(): Array<number> {
  return [
    build({
      pool: key(1),
      userSource: key(2),
      userDest: key(3),
      aVault: key(4),
      bVault: key(5),
      aTokenVault: key(6),
      bTokenVault: key(7),
      aVaultLpMint: key(8),
      bVaultLpMint: key(9),
      aVaultLp: key(10),
      bVaultLp: key(11),
      protocolTokenFee: key(12),
      payer: key(13),
    }).accountCount,
  ];
}
