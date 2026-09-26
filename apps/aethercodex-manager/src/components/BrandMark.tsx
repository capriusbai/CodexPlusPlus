import markUrl from "@brand/aethercodex-mark.svg";

type BrandMarkProps = {
  /** Rendered size in px. The mark is square by contract. */
  size?: number;
  className?: string;
};

/**
 * The AetherCodex application mark.
 *
 * The image comes from the single brand master in `assets/brand/`, so swapping
 * that file updates the UI and every platform icon at once. Kept square and
 * unrounded per ARCHAI-CIVI-001.
 */
export function BrandMark({ size = 36, className }: BrandMarkProps) {
  return (
    <img
      src={markUrl}
      width={size}
      height={size}
      className={className ? `brand-mark ${className}` : "brand-mark"}
      alt="AetherCodex"
      draggable={false}
    />
  );
}
