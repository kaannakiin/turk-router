export type FindRouteErrorDetail =
  | { readonly kind: "UnknownHopKind"; readonly raw: number }
  | { readonly kind: "NoRouteMints" }
  | { readonly kind: "TooManyRouteMints"; readonly given: number; readonly max: number }
  | { readonly kind: "EmptyMenu" }
  | { readonly kind: "TooManyMenuPools"; readonly given: number; readonly max: number }
  | {
      readonly kind: "MenuAccountBudgetExceeded";
      readonly declared: number;
      readonly budget: number;
    }
  | {
      readonly kind: "TailLength";
      readonly given: number;
      readonly min: number;
      readonly max: number;
    }
  | {
      readonly kind: "IntegerOutOfRange";
      readonly field: "maxWalkSteps" | "minProfitBaseUnits";
      readonly given: number | bigint;
      readonly min: bigint;
      readonly max: bigint;
    };

export type FindRouteErrorKind = FindRouteErrorDetail["kind"];

/**
 * Why an instruction could not be built. Each variant names an input the program would refuse
 * before reaching any named error of its own, so this package refuses it first, by name.
 * `IntegerOutOfRange` is the one variant the Rust client lacks: its `u8`/`u64` parameters make a
 * value outside the range unrepresentable, while a JavaScript caller can pass anything.
 */
export class FindRouteError extends Error {
  override readonly name = "FindRouteError" as const;
  readonly detail: FindRouteErrorDetail;

  constructor(detail: FindRouteErrorDetail) {
    super(describe(detail));
    this.detail = detail;
  }
}

export function isFindRouteError<K extends FindRouteErrorKind>(
  value: unknown,
  kind?: K,
): value is FindRouteError & { readonly detail: Extract<FindRouteErrorDetail, { kind: K }> } {
  return value instanceof FindRouteError && (kind === undefined || value.detail.kind === kind);
}

function describe(detail: FindRouteErrorDetail): string {
  switch (detail.kind) {
    case "UnknownHopKind":
      return `hop kind ${detail.raw} is not one the menu accepts`;
    case "NoRouteMints":
      return "a route needs at least one route mint";
    case "TooManyRouteMints":
      return `${detail.given} route mints given, the wire carries at most ${detail.max}`;
    case "EmptyMenu":
      return "a menu needs at least one pool";
    case "TooManyMenuPools":
      return `${detail.given} menu pools given, the wire carries at most ${detail.max}`;
    case "MenuAccountBudgetExceeded":
      return `the menu declares ${detail.declared} accounts, the program budgets ${detail.budget}`;
    case "TailLength":
      return `${detail.given} tail accounts given, the venue accepts ${detail.min}..=${detail.max}`;
    case "IntegerOutOfRange":
      return `${detail.field} is ${String(detail.given)}, the wire carries ${String(detail.min)}..=${String(detail.max)}`;
  }
}
