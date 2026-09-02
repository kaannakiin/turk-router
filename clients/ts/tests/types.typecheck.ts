/**
 * Compile-time proofs, checked by `npm run typecheck` and never executed: the runner's glob does
 * not match this file, and tsx would strip the types and run the erroneous lines. Each
 * `@ts-expect-error` is the TypeScript form of a Rust `compile_fail` doctest — tsc fails on an
 * unused directive, so a type that loosens turns the check red.
 */
import type { Address } from "@solana/addresses";

import {
  BaseMint,
  venues,
  type FindRouteParams,
  type HopKind,
  type HopKindName,
  type TupleRange,
  type VenueWindow,
  type WIRE_EPOCH,
} from "../src/index.js";
import { key } from "./common/keys.js";

type AssertEqual<T, U> = [T] extends [U] ? ([U] extends [T] ? true : never) : never;

const one = key(1);

const whirlpoolAccounts: venues.whirlpool.WhirlpoolAccounts = {
  tokenProgramA: one,
  tokenProgramB: one,
  tokenAuthority: one,
  whirlpool: one,
  mintA: one,
  mintB: one,
  tokenOwnerAccountA: one,
  tokenVaultA: one,
  tokenOwnerAccountB: one,
  tokenVaultB: one,
  tickArray0: one,
  tickArray1: one,
  tickArray2: one,
  oracle: one,
};

export const threeSupplemental: VenueWindow = venues.whirlpool.resolve(whirlpoolAccounts, [
  one,
  one,
  one,
]);

// @ts-expect-error a fourth supplemental tick array does not type-check (Rust: compile_fail)
export const fourSupplemental: VenueWindow = venues.whirlpool.resolve(whirlpoolAccounts, [
  one,
  one,
  one,
  one,
]);

const clmmAccounts: venues.raydiumClmm.RaydiumClmmAccounts = {
  payer: one,
  ammConfig: one,
  pool: one,
  inputTokenAccount: one,
  outputTokenAccount: one,
  inputVault: one,
  outputVault: one,
  observationState: one,
  inputMint: one,
  outputMint: one,
};

// @ts-expect-error an empty CLMM tail does not type-check
export const emptyClmmTail: VenueWindow = venues.raydiumClmm.resolve(clmmAccounts, []);

// @ts-expect-error an eighth CLMM tail account does not type-check
export const longClmmTail: VenueWindow = venues.raydiumClmm.resolve(clmmAccounts, [
  one,
  one,
  one,
  one,
  one,
  one,
  one,
  one,
]);

// @ts-expect-error 10 is not a menu kind
export const notAKind: HopKind = 10;

// @ts-expect-error 'PumpSwapSwap' is not a venue name
export const notAName: HopKindName = "PumpSwapSwap";

const okParams: FindRouteParams = {
  user: one,
  baseMint: BaseMint.Wsol,
  baseAta: one,
  feeWallet: one,
  flags: { flashloan: false, failIfNoProfit: false },
  maxWalkSteps: 0,
  minProfitBaseUnits: 0n,
  routeMints: [],
  menu: [],
};

export const numberProfit: FindRouteParams = {
  ...okParams,
  // @ts-expect-error min_profit is a u64: bigint only
  minProfitBaseUnits: 1,
};

export const extraFlag: FindRouteParams = {
  ...okParams,
  flags: {
    flashloan: true,
    failIfNoProfit: true,
    // @ts-expect-error the flags byte has two bits, not a third field
    extra: true,
  },
};

export const otherMint: FindRouteParams = {
  ...okParams,
  // @ts-expect-error a base mint is one of two addresses
  baseMint: one,
};

// @ts-expect-error an object literal is not a VenueWindow: the class is nominal
export const forgedWindow: VenueWindow = { hopKind: 0, accountCount: 9, accounts: [] };

export const epochPin: AssertEqual<typeof WIRE_EPOCH, 2> = true;

export const supplementalPin: AssertEqual<
  venues.whirlpool.SupplementalTickArrays,
  | readonly []
  | readonly [Address]
  | readonly [Address, Address]
  | readonly [Address, Address, Address]
> = true;

export const rangePin: AssertEqual<
  TupleRange<Address, 1, 3>,
  readonly [Address] | readonly [Address, Address] | readonly [Address, Address, Address]
> = true;

export const kindPin: AssertEqual<HopKind, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9> = true;

export const namePin: AssertEqual<
  HopKindName,
  | "RaydiumAmmV4"
  | "Whirlpool"
  | "RaydiumClmm"
  | "RaydiumCpmm"
  | "MeteoraDlmmSwap"
  | "MeteoraDlmmSwap2"
  | "MeteoraDammV2"
  | "PumpSwapSell"
  | "PumpSwapBuy"
  | "MeteoraDammV1"
> = true;

export const formPin: AssertEqual<venues.meteoraDammV2.DammV2Form, "Base" | "RateLimited"> = true;
