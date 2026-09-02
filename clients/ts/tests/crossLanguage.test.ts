/**
 * The TypeScript half of the golden corpus: for every case the Rust crate recorded, this package
 * must build the same bytes and the same account list, or refuse with the same error. The file is
 * read at module scope, synchronously, so every `test()` registers before the runner starts.
 */
import assert from "node:assert/strict";
import { test } from "node:test";

import { getAddressEncoder, getProgramDerivedAddress } from "@solana/addresses";
import { getBase16Encoder } from "@solana/codecs-strings";

import {
  buildFindRouteInstruction,
  HopKind,
  hopKindName,
  isFindRouteError,
  programs,
  WIRE_EPOCH,
  wire,
} from "../src/index.js";
import {
  baseMintFromAddress,
  instructionToExpected,
  loadCorpus,
  paramsFromCase,
  windowFromInput,
} from "./common/corpus.js";
import { arrayField, field, manifest, numberField, stringField } from "./common/paths.js";

const corpus = loadCorpus();
const positive = corpus.cases.filter((c) => c.expected !== undefined);

test("the corpus carries this package's wire epoch", () => {
  assert.equal(corpus.wireEpoch, WIRE_EPOCH);
});

for (const c of corpus.cases) {
  test(c.id, async () => {
    assert.ok((c.expected === undefined) !== (c.error === undefined), "one of expected, error");
    if (c.expected !== undefined) {
      const instruction = await buildFindRouteInstruction(paramsFromCase(c.params));
      assert.deepEqual(instructionToExpected(instruction), c.expected);
    } else {
      // Parameter construction runs inside the promise so that a tail refused while reading the
      // inputs is compared exactly like a builder refusal.
      await assert.rejects(
        Promise.resolve().then(() => buildFindRouteInstruction(paramsFromCase(c.params))),
        (thrown: unknown) => {
          assert.ok(isFindRouteError(thrown), "a FindRouteError");
          assert.deepEqual(thrown.detail, c.error);
          return true;
        },
      );
    }
  });
}

test("the sweep covers every window length the manifest accepts and every runtime error", () => {
  const accepted = new Set<string>();
  for (const entry of arrayField(field(manifest(), "find_route"), "menu_eligible_hop_kinds")) {
    const name = stringField(entry, "name");
    for (const length of arrayField(entry, "window_lens")) {
      accepted.add(`${name}/${String(length)}`);
    }
  }
  const swept = new Set<string>();
  for (const c of corpus.cases.filter((candidate) => candidate.id.startsWith("window/"))) {
    const menu = arrayField(c.params, "menu");
    assert.equal(menu.length, 1, `${c.id}: a window case names one window`);
    const window = windowFromInput(menu[0]);
    swept.add(`${hopKindName(window.hopKind)}/${String(window.accountCount)}`);
  }
  const byText = (a: string, b: string): number => a.localeCompare(b);
  assert.deepEqual([...swept].sort(byText), [...accepted].sort(byText));

  const flagBytes = new Set(
    positive.map((c) => {
      const flags = field(c.params, "flags");
      return (
        (field(flags, "flashloan") === true ? 1 : 0) |
        (field(flags, "fail_if_no_profit") === true ? 2 : 0)
      );
    }),
  );
  assert.deepEqual(
    [...flagBytes].sort((a, b) => a - b),
    [0, 1, 2, 3],
  );
  const mintCounts = new Set(positive.map((c) => arrayField(c.params, "route_mints").length));
  assert.deepEqual(
    [...mintCounts].sort((a, b) => a - b),
    [1, 2, 3, 4],
  );
  const baseMints = new Set(positive.map((c) => stringField(c.params, "base_mint")));
  assert.equal(baseMints.size, 2, "both base mints appear");

  const errorKinds = new Set(
    corpus.cases.filter((c) => c.error !== undefined).map((c) => stringField(c.error, "kind")),
  );
  assert.deepEqual(
    [...errorKinds].sort((a, b) => a.localeCompare(b)),
    [
      "EmptyMenu",
      "MenuAccountBudgetExceeded",
      "NoRouteMints",
      "TailLength",
      "TooManyMenuPools",
      "TooManyRouteMints",
    ],
  );
});

// A third derivation of the wire, from the case's own inputs and outputs and the manifest, with
// no client code: a mistake both encoders share still fails here.
test("the corpus expectations obey the wire spec independently of both clients", () => {
  const discriminant = new Map<string, number>();
  const windowLens = new Map<string, Set<number>>();
  for (const entry of arrayField(field(manifest(), "find_route"), "menu_eligible_hop_kinds")) {
    const name = stringField(entry, "name");
    discriminant.set(name, numberField(entry, "discriminant"));
    windowLens.set(name, new Set(arrayField(entry, "window_lens").map((length) => Number(length))));
  }
  for (const c of positive) {
    assert.ok(c.expected !== undefined);
    const data = getBase16Encoder().encode(c.expected.data_hex);
    const menu = arrayField(c.params, "menu");
    const mints = arrayField(c.params, "route_mints").length;
    assert.equal(data.length, 20 + 4 * menu.length, c.id);
    assert.equal(
      getBase16Encoder().encode("6361705d8f055e00").join(","),
      data.slice(0, 8).join(","),
      c.id,
    );
    const flags = field(c.params, "flags");
    const flagByte =
      (field(flags, "flashloan") === true ? 1 : 0) |
      (field(flags, "fail_if_no_profit") === true ? 2 : 0);
    assert.equal(data[8], flagByte, `${c.id}: flags`);
    assert.equal(data[9], numberField(c.params, "max_walk_steps"), `${c.id}: max_walk_steps`);
    assert.equal(data[10], mints, `${c.id}: num_mints`);
    assert.equal(data[11], menu.length, `${c.id}: num_pools`);
    const profit = new DataView(new ArrayBuffer(8));
    profit.setBigUint64(0, BigInt(stringField(c.params, "min_profit_base_units")), true);
    assert.deepEqual(
      Array.from(data.slice(12, 20)),
      Array.from(new Uint8Array(profit.buffer)),
      `${c.id}: min_profit`,
    );
    let declared = 0;
    menu.forEach((window, index) => {
      const name = stringField(window, "kind");
      const entry = data.slice(20 + 4 * index, 24 + 4 * index);
      assert.equal(entry[0], discriminant.get(name), `${c.id}: entry ${String(index)} kind`);
      const count = entry[1] ?? 0;
      assert.ok(
        windowLens.get(name)?.has(count) === true,
        `${c.id}: entry ${String(index)} count ${String(count)}`,
      );
      assert.deepEqual([entry[2], entry[3]], [0, 0], `${c.id}: hook lens`);
      declared += count;
    });
    assert.ok(declared <= 69, c.id);
    assert.equal(c.expected.accounts.length, 6 + 2 * mints + declared, `${c.id}: account count`);
    assert.equal(c.expected.program_address, "TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am", c.id);
  }
});

test("every positive case sends the fee ata writable at slot five", async () => {
  const encoder = getAddressEncoder();
  const [config] = await getProgramDerivedAddress({
    programAddress: wire.ROUTER_PROGRAM_ADDRESS,
    seeds: [wire.CONFIG_SEED],
  });
  for (const c of positive) {
    assert.ok(c.expected !== undefined);
    const feeWallet = stringField(c.params, "fee_wallet");
    const baseMint = baseMintFromAddress(stringField(c.params, "base_mint"));
    const [feeAta] = await getProgramDerivedAddress({
      programAddress: programs.ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
      seeds: [
        encoder.encode(feeWallet as typeof baseMint),
        encoder.encode(programs.TOKEN_PROGRAM_ADDRESS),
        encoder.encode(baseMint),
      ],
    });
    assert.equal(c.expected.accounts[4], `${config}:readonly`, c.id);
    assert.equal(c.expected.accounts[5], `${feeAta}:writable`, c.id);
  }
  assert.ok(positive.length > 0);
  assert.ok(Object.keys(HopKind).length === 10);
});
