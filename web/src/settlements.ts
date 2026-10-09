// Several settlements in the observer (ADR-0018): the new-world dialog's neighbouring groups.

/** Most founding groups a world is made with, the first included (ADR-0018 §6). */
export const MAX_FOUNDING_GROUPS = 3;

/**
 * The sizes of the further founding groups typed as `text` ("40, 30"; blank for none), or what is
 * wrong with them. Sizes are checked against the content's band limits when they are known.
 */
export function neighbourSizes(
  text: string,
  limits: { bandSizeMin: number; bandSizeMax: number } | null,
): number[] | string {
  const parts = text
    .split(/[,\s]+/)
    .map((p) => p.trim())
    .filter((p) => p.length > 0);
  if (parts.length > MAX_FOUNDING_GROUPS - 1) {
    return `At most ${MAX_FOUNDING_GROUPS - 1} neighbouring groups.`;
  }
  const sizes: number[] = [];
  for (const p of parts) {
    if (!/^\d+$/.test(p)) {
      return "Each neighbouring group's size must be a whole number.";
    }
    const n = Number(p);
    if (limits && (n < limits.bandSizeMin || n > limits.bandSizeMax)) {
      return `Each neighbouring group must have ${limits.bandSizeMin} to ${limits.bandSizeMax} people.`;
    }
    sizes.push(n);
  }
  return sizes;
}
