import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { assertTailLength, isFindRouteError } from "../src/index.js";
import { key } from "./common/keys.js";

test("a tail holds min to max accounts", () => {
  const one: ReadonlyArray<Address> = [key(1)];
  assertTailLength(one, 1, 3);
  const three: ReadonlyArray<Address> = [key(1), key(2), key(3)];
  assertTailLength(three, 1, 3);
  const none: ReadonlyArray<Address> = [];
  assertTailLength(none, 0, 3);
});

test("assertTailLength reports given, min and max", () => {
  for (const [length, min, max] of [
    [0, 1, 3],
    [4, 1, 3],
    [4, 0, 3],
    [9, 1, 8],
  ] as const) {
    const keys: ReadonlyArray<Address> = Array.from({ length }, (_, index) => key(index + 1));
    assert.throws(
      () => {
        assertTailLength(keys, min, max);
      },
      (thrown: unknown) =>
        isFindRouteError(thrown, "TailLength") &&
        thrown.detail.given === length &&
        thrown.detail.min === min &&
        thrown.detail.max === max,
    );
  }
});
