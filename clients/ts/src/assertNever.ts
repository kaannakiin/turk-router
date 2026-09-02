export function assertNever(value: never): never {
  throw new Error(`unreachable: ${String(value)}`);
}
