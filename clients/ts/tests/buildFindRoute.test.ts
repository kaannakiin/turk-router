/**
 * The builder's local checks, both sides of every boundary, the bytes and account list it emits,
 * and the refusals only a TypeScript caller can provoke.
 */
import assert from "node:assert/strict";
import { test } from "node:test";

import { getAddressEncoder, getProgramDerivedAddress, type Address } from "@solana/addresses";
import { AccountRole } from "@solana/instructions";

import {
  BaseMint,
  buildFindRouteInstruction,
  isFindRouteError,
  programs,
  venues,
  wire,
  type FindRouteErrorDetail,
  type FindRouteParams,
  type RouteMint,
  type VenueWindow,
} from "../src/index.js";
import { key } from "./common/keys.js";

function ammV4(seed: number): VenueWindow {
  return venues.raydiumAmmV4.resolve({
    pool: key(seed),
    baseVault: key(seed + 1),
    quoteVault: key(seed + 2),
    userSource: key(seed + 3),
    userDestination: key(seed + 4),
    payer: key(200),
  });
}

function cpmm(seed: number): VenueWindow {
  return venues.raydiumCpmm.resolve({
    user: key(200),
    ammConfig: key(seed),
    pool: key(seed + 1),
    inputTokenAccount: key(seed + 2),
    outputTokenAccount: key(seed + 3),
    inputVault: key(seed + 4),
    outputVault: key(seed + 5),
    inputTokenProgram: programs.TOKEN_PROGRAM_ADDRESS,
    outputTokenProgram: programs.TOKEN_PROGRAM_ADDRESS,
    inputMint: key(seed + 6),
    outputMint: key(seed + 7),
    observationState: key(seed + 8),
  });
}

function mints(count: number): Array<RouteMint> {
  return Array.from({ length: count }, (_, index) => ({
    tokenProgram: programs.TOKEN_PROGRAM_ADDRESS,
    userAta: key(100 + index),
  }));
}

function params(
  routeMints: ReadonlyArray<RouteMint>,
  menu: ReadonlyArray<VenueWindow>,
  overrides: Partial<FindRouteParams> = {},
): FindRouteParams {
  return {
    user: key(200),
    baseMint: BaseMint.Wsol,
    baseAta: key(201),
    feeWallet: key(202),
    flags: { flashloan: false, failIfNoProfit: true },
    maxWalkSteps: 7,
    minProfitBaseUnits: 0x0102030405060708n,
    routeMints,
    menu,
    ...overrides,
  };
}

async function refusal(input: FindRouteParams): Promise<FindRouteErrorDetail> {
  try {
    await buildFindRouteInstruction(input);
  } catch (thrown: unknown) {
    assert.ok(isFindRouteError(thrown), "a FindRouteError");
    return thrown.detail;
  }
  assert.fail("the builder accepted the input");
}

function at<T>(list: ReadonlyArray<T>, index: number): T {
  const item = list[index];
  assert.ok(item !== undefined, `index ${String(index)}`);
  return item;
}

test("the route mint count is bounded on both sides", async () => {
  const menu = [ammV4(1)];
  assert.deepEqual(await refusal(params([], menu)), { kind: "NoRouteMints" });
  await buildFindRouteInstruction(params(mints(1), menu));
  await buildFindRouteInstruction(params(mints(wire.MAX_ROUTE_MINTS), menu));
  assert.deepEqual(await refusal(params(mints(wire.MAX_ROUTE_MINTS + 1), menu)), {
    kind: "TooManyRouteMints",
    given: wire.MAX_ROUTE_MINTS + 1,
    max: wire.MAX_ROUTE_MINTS,
  });
});

test("the menu pool count is bounded on both sides", async () => {
  const route = mints(1);
  assert.deepEqual(await refusal(params(route, [])), { kind: "EmptyMenu" });
  await buildFindRouteInstruction(params(route, [ammV4(1)]));
  // Eight nine-account windows are seventy-two declared accounts: the pool count is legal and the
  // budget is what refuses them.
  const eight = Array.from({ length: wire.MAX_MENU_POOLS }, (_, index) => ammV4(index));
  assert.deepEqual(await refusal(params(route, eight)), {
    kind: "MenuAccountBudgetExceeded",
    declared: 72,
    budget: wire.MAX_MENU_ACCOUNTS,
  });
  const nine = Array.from({ length: wire.MAX_MENU_POOLS + 1 }, (_, index) => ammV4(index));
  assert.deepEqual(await refusal(params(route, nine)), {
    kind: "TooManyMenuPools",
    given: wire.MAX_MENU_POOLS + 1,
    max: wire.MAX_MENU_POOLS,
  });
});

test("the account budget admits sixty-nine and refuses seventy", async () => {
  const route = mints(1);
  const exact = [cpmm(1), cpmm(20), cpmm(40), ammV4(60), ammV4(70), ammV4(80)];
  assert.equal(
    exact.reduce((total, window) => total + window.accountCount, 0),
    wire.MAX_MENU_ACCOUNTS,
  );
  await buildFindRouteInstruction(params(route, exact));
  const over = [cpmm(1), cpmm(20), cpmm(40), cpmm(60), cpmm(80)];
  assert.deepEqual(await refusal(params(route, over)), {
    kind: "MenuAccountBudgetExceeded",
    declared: 70,
    budget: wire.MAX_MENU_ACCOUNTS,
  });
});

test("the data is the header then one entry per pool", async () => {
  const menu = [ammV4(1), cpmm(20)];
  const instruction = await buildFindRouteInstruction(params(mints(2), menu));
  const data = Array.from(instruction.data);
  assert.equal(instruction.programAddress, wire.ROUTER_PROGRAM_ADDRESS);
  assert.equal(data.length, wire.HEADER_LEN + wire.MENU_ENTRY_LEN * menu.length);
  assert.deepEqual(data.slice(0, 8), Array.from(wire.FIND_ROUTE_DISC));
  assert.equal(data[8], 0b10, "flags: fail_if_no_profit only");
  assert.equal(data[9], 7, "max_walk_steps passes through as given");
  assert.equal(data[10], 2, "num_mints");
  assert.equal(data[11], 2, "num_pools");
  assert.deepEqual(data.slice(12, 20), [8, 7, 6, 5, 4, 3, 2, 1]);
  assert.deepEqual(data.slice(20, 24), [0, 9, 0, 0]);
  assert.deepEqual(data.slice(24, 28), [3, 14, 0, 0]);
});

test("both flag bits and neither are representable and nothing else", async () => {
  for (const [flags, byte] of [
    [{ flashloan: false, failIfNoProfit: false }, 0],
    [{ flashloan: true, failIfNoProfit: false }, 1],
    [{ flashloan: false, failIfNoProfit: true }, 2],
    [{ flashloan: true, failIfNoProfit: true }, 3],
  ] as const) {
    const instruction = await buildFindRouteInstruction(params(mints(1), [ammV4(1)], { flags }));
    assert.equal(instruction.data[8], byte);
  }
});

test("the account list is prefix then route mints then windows in order", async () => {
  const route = mints(2);
  const menu = [ammV4(1), cpmm(20)];
  const input = params(route, menu);
  const { accounts } = await buildFindRouteInstruction(input);
  const encoder = getAddressEncoder();
  const [config] = await getProgramDerivedAddress({
    programAddress: wire.ROUTER_PROGRAM_ADDRESS,
    seeds: [wire.CONFIG_SEED],
  });
  // Seed order is the contract; a readonly fee slot surfaces only onchain as FeeAccountMismatch.
  const [feeAta] = await getProgramDerivedAddress({
    programAddress: programs.ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    seeds: [
      encoder.encode(input.feeWallet),
      encoder.encode(programs.TOKEN_PROGRAM_ADDRESS),
      encoder.encode(BaseMint.Wsol),
    ],
  });
  const prefix: Array<[Address, AccountRole]> = [
    [input.user, AccountRole.READONLY_SIGNER],
    [input.baseAta, AccountRole.WRITABLE],
    [BaseMint.Wsol, AccountRole.READONLY],
    [programs.TOKEN_PROGRAM_ADDRESS, AccountRole.READONLY],
    [config, AccountRole.READONLY],
    [feeAta, AccountRole.WRITABLE],
  ];
  prefix.forEach(([address, role], index) => {
    assert.deepEqual(at(accounts, index), { address, role }, `slot ${String(index)}`);
  });
  route.forEach((mint, index) => {
    assert.deepEqual(at(accounts, 6 + 2 * index), {
      address: mint.tokenProgram,
      role: AccountRole.READONLY,
    });
    assert.deepEqual(at(accounts, 7 + 2 * index), {
      address: mint.userAta,
      role: AccountRole.WRITABLE,
    });
  });
  assert.deepEqual(
    accounts.slice(6 + 2 * route.length),
    menu.flatMap((window) => window.accounts),
  );
});

test("the fee token account follows the base mint", async () => {
  const route = mints(1);
  const menu = [ammV4(1)];
  const wsol = await buildFindRouteInstruction(params(route, menu, { baseMint: BaseMint.Wsol }));
  const usdc = await buildFindRouteInstruction(params(route, menu, { baseMint: BaseMint.Usdc }));
  assert.equal(at(wsol.accounts, 2).address, BaseMint.Wsol);
  assert.equal(at(usdc.accounts, 2).address, BaseMint.Usdc);
  assert.notEqual(at(wsol.accounts, 5).address, at(usdc.accounts, 5).address);
});

test("max walk steps passes through unclamped and is refused outside a byte", async () => {
  for (const steps of [0, 4, 255]) {
    const instruction = await buildFindRouteInstruction(
      params(mints(1), [ammV4(1)], { maxWalkSteps: steps }),
    );
    assert.equal(instruction.data[9], steps);
  }
  for (const steps of [256, -1, 1.5, Number.NaN]) {
    assert.deepEqual(await refusal(params(mints(1), [ammV4(1)], { maxWalkSteps: steps })), {
      kind: "IntegerOutOfRange",
      field: "maxWalkSteps",
      given: steps,
      min: 0n,
      max: 255n,
    });
  }
});

test("min profit is little-endian across the full u64 range and refused outside it", async () => {
  const cases: Array<[bigint, Array<number>]> = [
    [0n, [0, 0, 0, 0, 0, 0, 0, 0]],
    [1n, [1, 0, 0, 0, 0, 0, 0, 0]],
    [2n ** 64n - 1n, [255, 255, 255, 255, 255, 255, 255, 255]],
  ];
  for (const [profit, bytes] of cases) {
    const instruction = await buildFindRouteInstruction(
      params(mints(1), [ammV4(1)], { minProfitBaseUnits: profit }),
    );
    assert.deepEqual(Array.from(instruction.data.slice(12, 20)), bytes);
  }
  for (const profit of [2n ** 64n, -1n]) {
    assert.deepEqual(await refusal(params(mints(1), [ammV4(1)], { minProfitBaseUnits: profit })), {
      kind: "IntegerOutOfRange",
      field: "minProfitBaseUnits",
      given: profit,
      min: 0n,
      max: 2n ** 64n - 1n,
    });
  }
});

test("the data is exactly twenty plus four per pool", async () => {
  for (let pools = 1; pools <= 7; pools += 1) {
    const menu = Array.from({ length: pools }, (_, index) => ammV4(index * 10));
    const instruction = await buildFindRouteInstruction(params(mints(1), menu));
    assert.equal(instruction.data.length, 20 + 4 * pools);
    assert.equal(instruction.data[11], pools);
    assert.equal(instruction.accounts.length, 6 + 2 + 9 * pools);
  }
});

test("the checks run mints then pools then budget", async () => {
  const nine = Array.from({ length: 9 }, (_, index) => ammV4(index));
  assert.deepEqual(await refusal(params([], nine)), { kind: "NoRouteMints" });
  assert.deepEqual(await refusal(params(mints(1), nine)), {
    kind: "TooManyMenuPools",
    given: 9,
    max: 8,
  });
});

test("the instruction owns its bytes", async () => {
  const input = params(mints(1), [ammV4(1)]);
  const first = await buildFindRouteInstruction(input);
  const copy = new Uint8Array(first.data);
  copy[8] = 0xff;
  const second = await buildFindRouteInstruction(input);
  assert.deepEqual(Array.from(second.data), Array.from(first.data));
  assert.notDeepEqual(Array.from(copy), Array.from(second.data));
});
