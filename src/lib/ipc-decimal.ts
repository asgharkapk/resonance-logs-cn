export type IpcDecimal = string | number | bigint;

const DECIMAL_INTEGER = /^[+-]?\d+$/;
const RATIO_SCALE = 1_000_000_000n;
const RATIO_SCALE_NUMBER = 1_000_000_000;

/** Normalize an integer-valued IPC field without losing decimal-string precision. */
export function ipcBigInt(value: unknown): bigint {
  if (typeof value === "bigint") return value;

  if (typeof value === "number") {
    if (!Number.isFinite(value)) return 0n;
    return BigInt(Math.trunc(value));
  }

  if (typeof value !== "string") return 0n;
  const normalized = value.trim();
  if (!DECIMAL_INTEGER.test(normalized)) return 0n;

  try {
    return BigInt(normalized);
  } catch {
    return 0n;
  }
}

/** Convert only at a display or layout boundary. Invalid values become zero. */
export function ipcNumber(value: unknown): number {
  const normalized = ipcBigInt(value);
  const converted = Number(normalized);
  if (Number.isFinite(converted)) return converted;
  return normalized < 0n ? -Number.MAX_VALUE : Number.MAX_VALUE;
}

export function ipcIsZero(value: unknown): boolean {
  return ipcBigInt(value) === 0n;
}

export function ipcCompare(left: unknown, right: unknown): -1 | 0 | 1 {
  const a = ipcBigInt(left);
  const b = ipcBigInt(right);
  return a < b ? -1 : a > b ? 1 : 0;
}

export function ipcAdd(left: unknown, right: unknown): bigint {
  return ipcBigInt(left) + ipcBigInt(right);
}

export function ipcSum(values: Iterable<unknown>): bigint {
  let total = 0n;
  for (const value of values) total += ipcBigInt(value);
  return total;
}

/**
 * Compute `(numerator / denominator) * multiplier` with fixed-point BigInt
 * arithmetic. This avoids `Infinity / Infinity` for very large decimal strings.
 */
export function ipcRatio(
  numerator: unknown,
  denominator: unknown,
  multiplier: unknown = 1,
): number {
  const numeratorValue = ipcBigInt(numerator);
  const denominatorValue = ipcBigInt(denominator);
  const multiplierValue = ipcBigInt(multiplier);
  if (
    denominatorValue === 0n ||
    numeratorValue === 0n ||
    multiplierValue === 0n
  ) {
    return 0;
  }

  const negative =
    (numeratorValue < 0n !== denominatorValue < 0n) !== multiplierValue < 0n;
  const absoluteNumerator =
    (numeratorValue < 0n ? -numeratorValue : numeratorValue) *
    (multiplierValue < 0n ? -multiplierValue : multiplierValue);
  const absoluteDenominator =
    denominatorValue < 0n ? -denominatorValue : denominatorValue;
  const scaled =
    (absoluteNumerator * RATIO_SCALE + absoluteDenominator / 2n) /
    absoluteDenominator;
  const converted = Number(scaled) / RATIO_SCALE_NUMBER;
  const finite = Number.isFinite(converted) ? converted : Number.MAX_VALUE;
  return negative ? -finite : finite;
}
