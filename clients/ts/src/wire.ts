/**
 * The `find_route` wire: every number this package encodes, in one place.
 *
 * The instruction data is fixed-width: a 20-byte header, then one 4-byte menu entry per pool, and
 * nothing else — the program refuses data of any other length. The header is the discriminator,
 * the flags byte, `max_walk_steps`, `num_mints`, `num_pools`, and `min_profit_base_units` as a
 * little-endian `u64`. A menu entry is `hop_kind`, `account_count`, and the two transfer-hook group
 * lengths, which this package always sends as zero.
 *
 * Each constant here is held against `wire/wire-manifest.json` by the test suite. A wire literal
 * anywhere else in the package is a defect.
 */
import { address, type Address } from "@solana/addresses";
import {
  fixEncoderSize,
  type FixedSizeEncoder,
  type ReadonlyUint8Array,
} from "@solana/codecs-core";
import { getArrayEncoder, getBytesEncoder, getStructEncoder } from "@solana/codecs-data-structures";
import { getU64Encoder, getU8Encoder } from "@solana/codecs-numbers";

import { FindRouteError } from "./error.js";

/**
 * The wire revision this package encodes for. A consumer pins it; a mismatch against the deployed
 * program means the two disagree about the instruction's shape.
 */
export const WIRE_EPOCH = 2;

export type RouterProgramAddress = "TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am";

/** The router program. */
export const ROUTER_PROGRAM_ADDRESS: Address<RouterProgramAddress> = address(
  "TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am",
);

/** The one seed of the router's config account. */
export const CONFIG_SEED = "config";

/** The instruction discriminator: `sha256("global:find_route")[..8]`. */
export const FIND_ROUTE_DISC: ReadonlyUint8Array = new Uint8Array([
  0x63, 0x61, 0x70, 0x5d, 0x8f, 0x05, 0x5e, 0x00,
]);

/** Bytes before the first menu entry. */
export const HEADER_LEN = 20;

/** Bytes per menu entry. */
export const MENU_ENTRY_LEN = 4;

/** The most pools a menu may name. */
export const MAX_MENU_POOLS = 8;

/** The most route mints an instruction may name. */
export const MAX_ROUTE_MINTS = 4;

/** The ceiling on every window's `account_count`, summed over the menu. */
export const MAX_MENU_ACCOUNTS = 69;

/**
 * The ceiling on one transfer-hook account group. This package declares no groups; the number is
 * carried so the manifest agreement is complete.
 */
export const MAX_HOOK_GROUP_LEN = 21;

/**
 * Flags bit 0: the base token account holds borrowed principal a later instruction repays. The
 * program then behaves as if {@link FLAG_FAIL_IF_NO_PROFIT} were set.
 */
export const FLAG_FLASHLOAN = 1;

/**
 * Flags bit 1: fail the instruction, rather than settle a loss, when no cycle clears the profit
 * threshold.
 */
export const FLAG_FAIL_IF_NO_PROFIT = 2;

/** Wrapped SOL, the first base mint. */
export const WSOL_MINT_ADDRESS: Address<"So11111111111111111111111111111111111111112"> = address(
  "So11111111111111111111111111111111111111112",
);

/** USDC, the second base mint. */
export const USDC_MINT_ADDRESS: Address<"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"> = address(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
);

export interface PrefixAccountMeta {
  readonly name: string;
  readonly isSigner: boolean;
  readonly isWritable: boolean;
}

/** The six accounts before the route-mint section, with the flags a caller sends. */
export const PREFIX_ACCOUNT_METAS: readonly [
  PrefixAccountMeta,
  PrefixAccountMeta,
  PrefixAccountMeta,
  PrefixAccountMeta,
  PrefixAccountMeta,
  PrefixAccountMeta,
] = [
  { name: "user", isSigner: true, isWritable: false },
  { name: "base_ata", isSigner: false, isWritable: true },
  { name: "base_mint", isSigner: false, isWritable: false },
  { name: "base_token_program", isSigner: false, isWritable: false },
  { name: "config_account", isSigner: false, isWritable: false },
  { name: "fee_ata", isSigner: false, isWritable: true },
];

/** The header fields a caller chooses. Counts are supplied by the builder, which has bounded them. */
export interface FindRouteHeader {
  readonly flags: number;
  readonly maxWalkSteps: number;
  readonly numMints: number;
  readonly numPools: number;
  readonly minProfitBaseUnits: bigint;
}

/** One menu entry: the four bytes `[hop_kind, account_count, hook_lens[0], hook_lens[1]]`. */
export interface MenuEntry {
  readonly hopKind: number;
  readonly accountCount: number;
  readonly hookLen0: number;
  readonly hookLen1: number;
}

interface HeaderWithDiscriminator extends FindRouteHeader {
  readonly discriminator: ReadonlyUint8Array;
}

/** @internal */
export function getFindRouteHeaderEncoder(): FixedSizeEncoder<HeaderWithDiscriminator> {
  return getStructEncoder([
    ["discriminator", fixEncoderSize(getBytesEncoder(), FIND_ROUTE_DISC.length)],
    ["flags", getU8Encoder()],
    ["maxWalkSteps", getU8Encoder()],
    ["numMints", getU8Encoder()],
    ["numPools", getU8Encoder()],
    ["minProfitBaseUnits", getU64Encoder()],
  ]);
}

/** @internal */
export function getMenuEntryEncoder(): FixedSizeEncoder<MenuEntry> {
  return getStructEncoder([
    ["hopKind", getU8Encoder()],
    ["accountCount", getU8Encoder()],
    ["hookLen0", getU8Encoder()],
    ["hookLen1", getU8Encoder()],
  ]);
}

/**
 * Lays out the header and the entries; exactly `HEADER_LEN + MENU_ENTRY_LEN * entries.length`
 * bytes.
 * @internal
 */
export function encodeFindRouteData(
  header: FindRouteHeader,
  entries: ReadonlyArray<MenuEntry>,
): ReadonlyUint8Array {
  const headerBytes = getFindRouteHeaderEncoder().encode({
    discriminator: FIND_ROUTE_DISC,
    ...header,
  });
  const entryBytes = getArrayEncoder(getMenuEntryEncoder(), { size: entries.length }).encode([
    ...entries,
  ]);
  const data = new Uint8Array(headerBytes.length + entryBytes.length);
  data.set(headerBytes, 0);
  data.set(entryBytes, headerBytes.length);
  return data;
}

const U8_MAX = 255n;
const U64_MAX = 0xffff_ffff_ffff_ffffn;

/**
 * A byte the wire carries as given. Refused by name before any encoder runs, so a JavaScript
 * caller's out-of-range number is never silently wrapped.
 * @internal
 */
export function assertU8(field: "maxWalkSteps", value: number): void {
  if (!Number.isInteger(value) || value < 0 || value > 255) {
    throw new FindRouteError({
      kind: "IntegerOutOfRange",
      field,
      given: value,
      min: 0n,
      max: U8_MAX,
    });
  }
}

/** @internal */
export function assertU64(field: "minProfitBaseUnits", value: bigint): void {
  if (typeof value !== "bigint" || value < 0n || value > U64_MAX) {
    throw new FindRouteError({
      kind: "IntegerOutOfRange",
      field,
      given: value,
      min: 0n,
      max: U64_MAX,
    });
  }
}
