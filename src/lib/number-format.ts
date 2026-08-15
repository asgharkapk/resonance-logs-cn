/**
 * @file Shared magnitude abbreviation used wherever a large number has to fit
 * in a narrow column, axis label or tooltip. Extracted from
 * `components/abbreviated-number.svelte` so non-component code (chart axis
 * formatters in particular) can reuse the exact same rounding and suffixes -
 * a table cell and the curve's Y axis must not disagree about what "1.2m" is.
 */

export type AbbreviationStyle = "western" | "cn";

/** Splits `n` into its abbreviated mantissa and unit suffix, e.g.
 * `[ "1.2", "m" ]`. The suffix is returned separately so callers can style it
 * (smaller/muted) independently of the digits. */
export function abbreviateNumberSplit(
  n: number,
  dp: number,
  style: AbbreviationStyle,
): [string, string] {
  if (style === "cn") {
    if (n >= 1e8) return [(n / 1e8).toFixed(dp), "亿"];
    if (n >= 1e4) return [(n / 1e4).toFixed(dp), "w"];
    return [n.toFixed(0), ""];
  }

  if (n >= 1e3 && n < 1e6) return [(n / 1e3).toFixed(dp), "k"];
  if (n >= 1e6 && n < 1e9) return [(n / 1e6).toFixed(dp), "m"];
  if (n >= 1e9 && n < 1e12) return [(n / 1e9).toFixed(dp), "b"];
  if (n >= 1e12) return [(n / 1e12).toFixed(dp), "t"];
  return [n.toFixed(0), ""];
}

/** Convenience wrapper for plain-text contexts (axis labels, tooltips) that
 * cannot style the suffix separately. */
export function abbreviateNumber(
  n: number,
  dp: number,
  style: AbbreviationStyle,
): string {
  const [mantissa, suffix] = abbreviateNumberSplit(n, dp, style);
  return `${mantissa}${suffix}`;
}
