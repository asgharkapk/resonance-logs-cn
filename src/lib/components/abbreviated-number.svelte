<script lang="ts">
  /**
   * @file This component displays a number in an abbreviated format.
   */
  import { formatNumber } from "$lib/i18n/index.svelte";
  import {
    abbreviateNumberSplit,
    type AbbreviationStyle,
  } from "$lib/number-format";

  let {
    num = 0,
    decimalPlaces = 1,
    abbreviationStyle = "western",
    suffixFontSize,
    suffixColor,
  }: {
    num: number;
    decimalPlaces?: number;
    abbreviationStyle?: AbbreviationStyle;
    suffixFontSize?: number | undefined;
    suffixColor?: string | undefined;
  } = $props();

  let abbreviatedNumberTuple = $derived(
    abbreviateNumberSplit(num, decimalPlaces, abbreviationStyle),
  );
  let fullNumberString = $derived(formatNumber(num));

  let suffixStyle = $derived(
    [
      suffixFontSize ? `font-size: ${suffixFontSize}px` : "",
      suffixColor ? `color: ${suffixColor}` : "",
    ]
      .filter(Boolean)
      .join("; "),
  );
</script>

<span
  title={fullNumberString}
  class="inline-flex items-baseline gap-0.5 whitespace-nowrap"
>
  {abbreviatedNumberTuple[0]}<span
    class="text-tiny text-muted-foreground"
    style={suffixStyle || undefined}>{abbreviatedNumberTuple[1]}</span
  >
</span>
