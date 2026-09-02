import assert from "node:assert/strict";
import { test } from "node:test";

import { isFindRouteError, wire } from "../src/index.js";
import {
  assertU64,
  assertU8,
  encodeFindRouteData,
  getFindRouteHeaderEncoder,
  getMenuEntryEncoder,
} from "../src/wire.js";

test("the program address is the documented one", () => {
  assert.equal(wire.ROUTER_PROGRAM_ADDRESS, "TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am");
});

test("the header is twenty bytes in field order", () => {
  const data = Array.from(
    encodeFindRouteData(
      {
        flags: wire.FLAG_FAIL_IF_NO_PROFIT,
        maxWalkSteps: 3,
        numMints: 2,
        numPools: 1,
        minProfitBaseUnits: 0x0102030405060708n,
      },
      [{ hopKind: 0, accountCount: 9, hookLen0: 0, hookLen1: 0 }],
    ),
  );
  assert.equal(data.length, wire.HEADER_LEN + wire.MENU_ENTRY_LEN);
  assert.deepEqual(data.slice(0, 8), Array.from(wire.FIND_ROUTE_DISC));
  assert.deepEqual(data.slice(8, 12), [2, 3, 2, 1]);
  assert.deepEqual(data.slice(12, 20), [8, 7, 6, 5, 4, 3, 2, 1]);
  assert.deepEqual(data.slice(20), [0, 9, 0, 0]);
});

test("the codec sizes equal the wire widths", () => {
  assert.equal(getFindRouteHeaderEncoder().fixedSize, wire.HEADER_LEN);
  assert.equal(getMenuEntryEncoder().fixedSize, wire.MENU_ENTRY_LEN);
});

test("the byte guards refuse non-integers, negatives and values above the width", () => {
  assertU8("maxWalkSteps", 0);
  assertU8("maxWalkSteps", 255);
  for (const value of [256, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () => {
        assertU8("maxWalkSteps", value);
      },
      (thrown: unknown) => isFindRouteError(thrown, "IntegerOutOfRange"),
    );
  }
  assertU64("minProfitBaseUnits", 0n);
  assertU64("minProfitBaseUnits", 2n ** 64n - 1n);
  for (const value of [2n ** 64n, -1n]) {
    assert.throws(
      () => {
        assertU64("minProfitBaseUnits", value);
      },
      (thrown: unknown) => isFindRouteError(thrown, "IntegerOutOfRange"),
    );
  }
});
