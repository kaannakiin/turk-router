import type { Address } from "@solana/addresses";
import type { ReadonlyUint8Array } from "@solana/codecs-core";
import type {
  AccountMeta,
  Instruction,
  InstructionWithAccounts,
  InstructionWithData,
} from "@solana/instructions";

import { FindRouteError, type FindRouteErrorDetail } from "./error.js";
import { findAssociatedTokenAddress, findConfigAccountAddress } from "./pda.js";
import { TOKEN_PROGRAM_ADDRESS } from "./programs.js";
import type { VenueWindow } from "./venues/window.js";
import { readonlyAccount, readonlySignerAccount, writableAccount } from "./venues/window.js";
import {
  FLAG_FAIL_IF_NO_PROFIT,
  FLAG_FLASHLOAN,
  MAX_MENU_ACCOUNTS,
  MAX_MENU_POOLS,
  MAX_ROUTE_MINTS,
  ROUTER_PROGRAM_ADDRESS,
  USDC_MINT_ADDRESS,
  WSOL_MINT_ADDRESS,
  assertU64,
  assertU8,
  encodeFindRouteData,
  type RouterProgramAddress,
} from "./wire.js";

/** The two mints a cycle may start and end on; the value is the mint address. */
export const BaseMint: {
  readonly Wsol: typeof WSOL_MINT_ADDRESS;
  readonly Usdc: typeof USDC_MINT_ADDRESS;
} = {
  Wsol: WSOL_MINT_ADDRESS,
  Usdc: USDC_MINT_ADDRESS,
};

export type BaseMint = (typeof BaseMint)[keyof typeof BaseMint];

/** Both base mints, in the program's order. */
export const ALL_BASE_MINTS: readonly [BaseMint, BaseMint] = [BaseMint.Wsol, BaseMint.Usdc];

/**
 * The flags byte, as the two bits it defines. The other six bits are refused by the program and
 * cannot be set from here.
 */
export interface FindRouteFlags {
  /**
   * The base token account holds borrowed principal a later instruction in the same transaction
   * repays. The program then also fails rather than settle a loss.
   */
  readonly flashloan: boolean;
  /** Fail the instruction, rather than settle a loss, when no cycle clears the profit threshold. */
  readonly failIfNoProfit: boolean;
}

export function findRouteFlagsToByte(flags: FindRouteFlags): number {
  return (
    (flags.flashloan ? FLAG_FLASHLOAN : 0) | (flags.failIfNoProfit ? FLAG_FAIL_IF_NO_PROFIT : 0)
  );
}

/** One mint the route may pass through, with the user's token account for it. */
export interface RouteMint {
  /** The program that owns `userAta`: the Token program or the Token Extensions program. */
  readonly tokenProgram: Address;
  /** The user's token account for the mint. The program requires `tokenProgram` to own it. */
  readonly userAta: Address;
}

/**
 * Everything one `find_route` instruction needs.
 *
 * The base mint is node 0 of the graph the program searches, and `routeMints[i]` is node `i + 1`.
 * Reordering the entries changes which cycles exist, not merely the account list. A route mint
 * equal to the base mint, or repeated, is refused by the program.
 */
export interface FindRouteParams {
  /** The signer whose token accounts the route moves through. */
  readonly user: Address;
  /** Which of the two base mints the cycle starts and ends on. */
  readonly baseMint: BaseMint;
  /** The user's token account for the base mint. Supplied, never derived. */
  readonly baseAta: Address;
  /**
   * The router's fee collector, as the wallet address its config stores. The builder derives that
   * wallet's token account for `baseMint` and sends it writable on every call; the program reads
   * it only at a nonzero fee rate. A readonly fee slot surfaces only onchain, as
   * `FeeAccountMismatch`.
   */
  readonly feeWallet: Address;
  readonly flags: FindRouteFlags;
  /**
   * How many steps a walk-venue quote may take. Sent as given: the program substitutes its
   * default for zero and clamps a value above its cap.
   */
  readonly maxWalkSteps: number;
  /**
   * The least profit, in the base mint's minor units and net of the router's fee, a cycle must
   * clear to be executed. The program treats zero as one.
   */
  readonly minProfitBaseUnits: bigint;
  /** The mints the route may pass through, in node order. One to `MAX_ROUTE_MINTS`. */
  readonly routeMints: ReadonlyArray<RouteMint>;
  /**
   * The pools the program may choose among, in the order they are declared. One to
   * `MAX_MENU_POOLS` windows whose account counts sum to at most `MAX_MENU_ACCOUNTS`.
   */
  readonly menu: ReadonlyArray<VenueWindow>;
}

export type FindRouteInstruction = Instruction<RouterProgramAddress, ReadonlyArray<AccountMeta>> &
  InstructionWithAccounts<ReadonlyArray<AccountMeta>> &
  InstructionWithData<ReadonlyUint8Array>;

/**
 * Builds the instruction. Async because the two derivations it performs, the config account and
 * the fee collector's token account, hash with WebCrypto; nothing reaches the network.
 *
 * Throws `FindRouteError` for a route mint list outside `1..=MAX_ROUTE_MINTS`, a menu outside
 * `1..=MAX_MENU_POOLS`, windows whose account counts sum past `MAX_MENU_ACCOUNTS`, or a
 * `maxWalkSteps`/`minProfitBaseUnits` outside its wire width. What this package cannot check is
 * left to the program: that `user` signs, that the token accounts are owned as declared, that
 * the fee collector's token account exists when the fee rate is nonzero, that the router is not
 * paused, and that a profitable cycle exists.
 */
export async function buildFindRouteInstruction(
  params: FindRouteParams,
): Promise<FindRouteInstruction> {
  const numMints = boundedCount(
    params.routeMints.length,
    MAX_ROUTE_MINTS,
    { kind: "NoRouteMints" },
    (given) => ({ kind: "TooManyRouteMints", given, max: MAX_ROUTE_MINTS }),
  );
  const numPools = boundedCount(
    params.menu.length,
    MAX_MENU_POOLS,
    { kind: "EmptyMenu" },
    (given) => ({ kind: "TooManyMenuPools", given, max: MAX_MENU_POOLS }),
  );
  const declared = params.menu.reduce((total, window) => total + window.accountCount, 0);
  if (declared > MAX_MENU_ACCOUNTS) {
    throw new FindRouteError({
      kind: "MenuAccountBudgetExceeded",
      declared,
      budget: MAX_MENU_ACCOUNTS,
    });
  }
  assertU8("maxWalkSteps", params.maxWalkSteps);
  assertU64("minProfitBaseUnits", params.minProfitBaseUnits);

  const [configAccount, feeAta] = await Promise.all([
    findConfigAccountAddress(),
    findAssociatedTokenAddress(params.feeWallet, params.baseMint, TOKEN_PROGRAM_ADDRESS),
  ]);

  const accounts: ReadonlyArray<AccountMeta> = Object.freeze([
    readonlySignerAccount(params.user),
    writableAccount(params.baseAta),
    readonlyAccount(params.baseMint),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
    readonlyAccount(configAccount),
    writableAccount(feeAta),
    ...params.routeMints.flatMap((mint) => [
      readonlyAccount(mint.tokenProgram),
      writableAccount(mint.userAta),
    ]),
    ...params.menu.flatMap((window) => window.accounts),
  ]);
  const data = encodeFindRouteData(
    {
      flags: findRouteFlagsToByte(params.flags),
      maxWalkSteps: params.maxWalkSteps,
      numMints,
      numPools,
      minProfitBaseUnits: params.minProfitBaseUnits,
    },
    params.menu.map((window) => window.menuEntry()),
  );
  return Object.freeze({ programAddress: ROUTER_PROGRAM_ADDRESS, accounts, data });
}

function boundedCount(
  given: number,
  max: number,
  empty: FindRouteErrorDetail,
  over: (given: number) => FindRouteErrorDetail,
): number {
  if (given === 0) {
    throw new FindRouteError(empty);
  }
  if (given > max) {
    throw new FindRouteError(over(given));
  }
  return given;
}
