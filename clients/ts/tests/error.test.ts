import assert from "node:assert/strict";
import { test } from "node:test";

import { FindRouteError, isFindRouteError } from "../src/index.js";

test("messages mirror the Rust display strings", () => {
  const cases: Array<[FindRouteError, string]> = [
    [
      new FindRouteError({ kind: "UnknownHopKind", raw: 12 }),
      "hop kind 12 is not one the menu accepts",
    ],
    [new FindRouteError({ kind: "NoRouteMints" }), "a route needs at least one route mint"],
    [
      new FindRouteError({ kind: "TooManyRouteMints", given: 5, max: 4 }),
      "5 route mints given, the wire carries at most 4",
    ],
    [new FindRouteError({ kind: "EmptyMenu" }), "a menu needs at least one pool"],
    [
      new FindRouteError({ kind: "TooManyMenuPools", given: 9, max: 8 }),
      "9 menu pools given, the wire carries at most 8",
    ],
    [
      new FindRouteError({ kind: "MenuAccountBudgetExceeded", declared: 70, budget: 69 }),
      "the menu declares 70 accounts, the program budgets 69",
    ],
    [
      new FindRouteError({ kind: "TailLength", given: 8, min: 1, max: 7 }),
      "8 tail accounts given, the venue accepts 1..=7",
    ],
  ];
  for (const [error, message] of cases) {
    assert.equal(error.message, message);
    assert.equal(error.name, "FindRouteError");
    assert.ok(error instanceof Error);
  }
});

test("isFindRouteError narrows by kind", () => {
  const error: unknown = new FindRouteError({ kind: "TailLength", given: 0, min: 1, max: 7 });
  assert.ok(isFindRouteError(error));
  assert.ok(isFindRouteError(error, "TailLength"));
  assert.ok(!isFindRouteError(error, "EmptyMenu"));
  assert.ok(!isFindRouteError(new Error("TailLength")));
  if (isFindRouteError(error, "TailLength")) {
    assert.equal(error.detail.max, 7);
  }
});
