/**
 * The cross-language golden corpus, read from `clients/golden/find_route.json`: typed inputs to
 * `buildFindRouteInstruction` and the bytes and account list the Rust client emitted for each.
 * This package only reads the file; the Rust crate's `cross_language.rs` writes it.
 */
import { address, type Address } from "@solana/addresses";
import { getBase16Decoder } from "@solana/codecs-strings";
import { AccountRole, type AccountMeta } from "@solana/instructions";

import { assertNever } from "../../src/assertNever.js";
import {
  ALL_BASE_MINTS,
  assertTailLength,
  venues,
  type BaseMint,
  type FindRouteInstruction,
  type FindRouteParams,
  type VenueWindow,
} from "../../src/index.js";
import {
  arrayField,
  booleanField,
  field,
  goldenPath,
  isRecord,
  numberField,
  readText,
  stringField,
} from "./paths.js";

export interface Expected {
  readonly accounts: Array<string>;
  readonly data_hex: string;
  readonly program_address: string;
}

export interface CorpusCase {
  readonly id: string;
  readonly params: unknown;
  readonly expected: Expected | undefined;
  readonly error: unknown;
}

export interface Corpus {
  readonly cases: Array<CorpusCase>;
  readonly wireEpoch: number;
}

export function loadCorpus(): Corpus {
  const root: unknown = JSON.parse(readText(goldenPath));
  const cases = field(root, "cases");
  if (!isRecord(cases)) {
    throw new Error("cases is not an object");
  }
  return {
    wireEpoch: numberField(root, "wire_epoch"),
    cases: Object.entries(cases).map(([id, value]) => ({
      id,
      params: field(value, "params"),
      expected: isRecord(value) && "expected" in value ? expectedOf(value["expected"]) : undefined,
      error: isRecord(value) && "error" in value ? value["error"] : undefined,
    })),
  };
}

function expectedOf(value: unknown): Expected {
  return {
    accounts: arrayField(value, "accounts").map((entry) => {
      if (typeof entry !== "string") {
        throw new Error("an account entry is not a string");
      }
      return entry;
    }),
    data_hex: stringField(value, "data_hex"),
    program_address: stringField(value, "program_address"),
  };
}

function addr(record: unknown, key: string): Address {
  return address(stringField(record, key));
}

function optionalAddr(record: unknown, key: string): Address | undefined {
  const value = field(record, key);
  if (value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`${key} is not an address or null`);
  }
  return address(value);
}

function addrArray(record: unknown, key: string): Array<Address> {
  return arrayField(record, key).map((entry) => {
    if (typeof entry !== "string") {
      throw new Error(`${key} holds a non-address`);
    }
    return address(entry);
  });
}

function addrPair(record: unknown, key: string): readonly [Address, Address] {
  const [first, second, ...rest] = addrArray(record, key);
  if (first === undefined || second === undefined || rest.length > 0) {
    throw new Error(`${key} is not a pair`);
  }
  return [first, second];
}

function optionalAddrPair(record: unknown, key: string): readonly [Address, Address] | undefined {
  return field(record, key) === null ? undefined : addrPair(record, key);
}

function addrTen(
  record: unknown,
  key: string,
): readonly [
  Address,
  Address,
  Address,
  Address,
  Address,
  Address,
  Address,
  Address,
  Address,
  Address,
] {
  const list = addrArray(record, key);
  const [a, b, c, d, e, f, g, h, i, j] = list;
  if (
    list.length !== 10 ||
    a === undefined ||
    b === undefined ||
    c === undefined ||
    d === undefined ||
    e === undefined ||
    f === undefined ||
    g === undefined ||
    h === undefined ||
    i === undefined ||
    j === undefined
  ) {
    throw new Error(`${key} is not ten addresses`);
  }
  return [a, b, c, d, e, f, g, h, i, j];
}

export function baseMintFromAddress(text: string): BaseMint {
  const mint = ALL_BASE_MINTS.find((candidate) => candidate === text);
  if (mint === undefined) {
    throw new Error(`${text} is not a base mint`);
  }
  return mint;
}

export function windowFromInput(input: unknown): VenueWindow {
  const kind = stringField(input, "kind");
  const accounts = field(input, "accounts");
  switch (kind) {
    case "RaydiumAmmV4":
      return venues.raydiumAmmV4.resolve({
        pool: addr(accounts, "pool"),
        baseVault: addr(accounts, "base_vault"),
        quoteVault: addr(accounts, "quote_vault"),
        userSource: addr(accounts, "user_source"),
        userDestination: addr(accounts, "user_destination"),
        payer: addr(accounts, "payer"),
      });
    case "Whirlpool": {
      const supplemental = addrArray(input, "supplemental_tick_arrays");
      assertTailLength(supplemental, 0, 3);
      return venues.whirlpool.resolve(
        {
          tokenProgramA: addr(accounts, "token_program_a"),
          tokenProgramB: addr(accounts, "token_program_b"),
          tokenAuthority: addr(accounts, "token_authority"),
          whirlpool: addr(accounts, "whirlpool"),
          mintA: addr(accounts, "mint_a"),
          mintB: addr(accounts, "mint_b"),
          tokenOwnerAccountA: addr(accounts, "token_owner_account_a"),
          tokenVaultA: addr(accounts, "token_vault_a"),
          tokenOwnerAccountB: addr(accounts, "token_owner_account_b"),
          tokenVaultB: addr(accounts, "token_vault_b"),
          tickArray0: addr(accounts, "tick_array_0"),
          tickArray1: addr(accounts, "tick_array_1"),
          tickArray2: addr(accounts, "tick_array_2"),
          oracle: addr(accounts, "oracle"),
        },
        supplemental,
      );
    }
    case "RaydiumClmm": {
      const tail = addrArray(input, "tail");
      assertTailLength(tail, 1, 7);
      return venues.raydiumClmm.resolve(
        {
          payer: addr(accounts, "payer"),
          ammConfig: addr(accounts, "amm_config"),
          pool: addr(accounts, "pool"),
          inputTokenAccount: addr(accounts, "input_token_account"),
          outputTokenAccount: addr(accounts, "output_token_account"),
          inputVault: addr(accounts, "input_vault"),
          outputVault: addr(accounts, "output_vault"),
          observationState: addr(accounts, "observation_state"),
          inputMint: addr(accounts, "input_mint"),
          outputMint: addr(accounts, "output_mint"),
        },
        tail,
      );
    }
    case "RaydiumCpmm":
      return venues.raydiumCpmm.resolve({
        user: addr(accounts, "user"),
        ammConfig: addr(accounts, "amm_config"),
        pool: addr(accounts, "pool"),
        inputTokenAccount: addr(accounts, "input_token_account"),
        outputTokenAccount: addr(accounts, "output_token_account"),
        inputVault: addr(accounts, "input_vault"),
        outputVault: addr(accounts, "output_vault"),
        inputTokenProgram: addr(accounts, "input_token_program"),
        outputTokenProgram: addr(accounts, "output_token_program"),
        inputMint: addr(accounts, "input_mint"),
        outputMint: addr(accounts, "output_mint"),
        observationState: addr(accounts, "observation_state"),
      });
    case "MeteoraDlmmSwap": {
      const binArrays = addrArray(input, "bin_arrays");
      assertTailLength(binArrays, 1, venues.meteoraDlmmSwap.MAX_BINS);
      return venues.meteoraDlmmSwap.resolve(
        {
          lbPair: addr(accounts, "lb_pair"),
          binArrayBitmapExtension: optionalAddr(accounts, "bin_array_bitmap_extension"),
          reserveX: addr(accounts, "reserve_x"),
          reserveY: addr(accounts, "reserve_y"),
          userTokenIn: addr(accounts, "user_token_in"),
          userTokenOut: addr(accounts, "user_token_out"),
          mintX: addr(accounts, "mint_x"),
          mintY: addr(accounts, "mint_y"),
          oracle: addr(accounts, "oracle"),
          hostFeeIn: optionalAddr(accounts, "host_fee_in"),
          user: addr(accounts, "user"),
        },
        binArrays,
      );
    }
    case "MeteoraDlmmSwap2": {
      const binArrays = addrArray(input, "bin_arrays");
      assertTailLength(binArrays, 1, venues.meteoraDlmmSwap2.MAX_BIN_ARRAYS);
      return venues.meteoraDlmmSwap2.resolve(
        {
          pool: addr(accounts, "pool"),
          binArrayBitmapExtension: optionalAddr(accounts, "bin_array_bitmap_extension"),
          reserveX: addr(accounts, "reserve_x"),
          reserveY: addr(accounts, "reserve_y"),
          userTokenIn: addr(accounts, "user_token_in"),
          userTokenOut: addr(accounts, "user_token_out"),
          tokenXMint: addr(accounts, "token_x_mint"),
          tokenYMint: addr(accounts, "token_y_mint"),
          oracle: addr(accounts, "oracle"),
          hostFeeIn: optionalAddr(accounts, "host_fee_in"),
          user: addr(accounts, "user"),
          tokenXProgram: addr(accounts, "token_x_program"),
          tokenYProgram: addr(accounts, "token_y_program"),
        },
        binArrays,
      );
    }
    case "MeteoraDammV2": {
      const form = stringField(input, "form");
      if (form !== "Base" && form !== "RateLimited") {
        throw new Error(`unknown DAMM v2 form ${form}`);
      }
      return venues.meteoraDammV2.resolve(
        {
          pool: addr(accounts, "pool"),
          inputTokenAccount: addr(accounts, "input_token_account"),
          outputTokenAccount: addr(accounts, "output_token_account"),
          tokenAVault: addr(accounts, "token_a_vault"),
          tokenBVault: addr(accounts, "token_b_vault"),
          tokenAMint: addr(accounts, "token_a_mint"),
          tokenBMint: addr(accounts, "token_b_mint"),
          payer: addr(accounts, "payer"),
          tokenAProgram: addr(accounts, "token_a_program"),
          tokenBProgram: addr(accounts, "token_b_program"),
          referralTokenAccount: optionalAddr(accounts, "referral_token_account"),
        },
        venues.meteoraDammV2.DammV2Form[form],
      );
    }
    case "PumpSwapSell":
      return venues.pumpSwapSell.resolve({
        pool: addr(accounts, "pool"),
        user: addr(accounts, "user"),
        forwardedBeforeBaseMint: addr(accounts, "forwarded_before_base_mint"),
        baseMint: addr(accounts, "base_mint"),
        quoteMint: addr(accounts, "quote_mint"),
        baseAta: addr(accounts, "base_ata"),
        quoteAta: addr(accounts, "quote_ata"),
        baseVault: addr(accounts, "base_vault"),
        quoteVault: addr(accounts, "quote_vault"),
        forwardedBeforeFeeConfig: addrTen(accounts, "forwarded_before_fee_config"),
        cashback: optionalAddrPair(accounts, "cashback"),
        poolV2: optionalAddr(accounts, "pool_v2"),
        forwardedClose: addrPair(accounts, "forwarded_close"),
      });
    case "PumpSwapBuy":
      return venues.pumpSwapBuy.resolve({
        pool: addr(accounts, "pool"),
        user: addr(accounts, "user"),
        forwardedBeforeBaseMint: addr(accounts, "forwarded_before_base_mint"),
        baseMint: addr(accounts, "base_mint"),
        quoteMint: addr(accounts, "quote_mint"),
        baseTokenAccount: addr(accounts, "base_token_account"),
        quoteTokenAccount: addr(accounts, "quote_token_account"),
        baseVault: addr(accounts, "base_vault"),
        quoteVault: addr(accounts, "quote_vault"),
        forwardedBeforeVolumeAccumulator: addrTen(accounts, "forwarded_before_volume_accumulator"),
        userVolumeAccumulator: addr(accounts, "user_volume_accumulator"),
        forwardedClose: addrPair(accounts, "forwarded_close"),
        poolV2: optionalAddr(accounts, "pool_v2"),
        cashback: optionalAddr(accounts, "cashback"),
      });
    case "MeteoraDammV1":
      return venues.meteoraDammV1.resolve({
        pool: addr(accounts, "pool"),
        userSource: addr(accounts, "user_source"),
        userDest: addr(accounts, "user_dest"),
        aVault: addr(accounts, "a_vault"),
        bVault: addr(accounts, "b_vault"),
        aTokenVault: addr(accounts, "a_token_vault"),
        bTokenVault: addr(accounts, "b_token_vault"),
        aVaultLpMint: addr(accounts, "a_vault_lp_mint"),
        bVaultLpMint: addr(accounts, "b_vault_lp_mint"),
        aVaultLp: addr(accounts, "a_vault_lp"),
        bVaultLp: addr(accounts, "b_vault_lp"),
        protocolTokenFee: addr(accounts, "protocol_token_fee"),
        payer: addr(accounts, "payer"),
      });
    default:
      throw new Error(`${kind} is not a menu kind`);
  }
}

export function paramsFromCase(params: unknown): FindRouteParams {
  const flags = field(params, "flags");
  return {
    user: addr(params, "user"),
    baseMint: baseMintFromAddress(stringField(params, "base_mint")),
    baseAta: addr(params, "base_ata"),
    feeWallet: addr(params, "fee_wallet"),
    flags: {
      flashloan: booleanField(flags, "flashloan"),
      failIfNoProfit: booleanField(flags, "fail_if_no_profit"),
    },
    maxWalkSteps: numberField(params, "max_walk_steps"),
    minProfitBaseUnits: BigInt(stringField(params, "min_profit_base_units")),
    routeMints: arrayField(params, "route_mints").map((mint) => ({
      tokenProgram: addr(mint, "token_program"),
      userAta: addr(mint, "user_ata"),
    })),
    menu: arrayField(params, "menu").map(windowFromInput),
  };
}

export function roleName(role: AccountRole): string {
  switch (role) {
    case AccountRole.READONLY:
      return "readonly";
    case AccountRole.WRITABLE:
      return "writable";
    case AccountRole.READONLY_SIGNER:
      return "readonly_signer";
    case AccountRole.WRITABLE_SIGNER:
      return "writable_signer";
    default:
      return assertNever(role);
  }
}

export function metaText(meta: AccountMeta): string {
  return `${meta.address}:${roleName(meta.role)}`;
}

export function instructionToExpected(instruction: FindRouteInstruction): Expected {
  return {
    accounts: instruction.accounts.map(metaText),
    data_hex: getBase16Decoder().decode(instruction.data),
    program_address: instruction.programAddress,
  };
}
