/**
 * A capture of the real QuotaDial window (apps/tray/scripts/check-window.mjs with the site's example
 * accounts), in the theme the visitor's system uses. Never an interface drawn to look like the app.
 */
export function AppShot({
  name,
  alt,
  width,
  height,
  priority = false,
  className = "",
}: {
  /** The capture's base name in public/images/app, such as "accounts-pro". */
  name: string;
  /** What the window shows; "example accounts" is added. */
  alt: string;
  width: number;
  height: number;
  priority?: boolean;
  className?: string;
}) {
  return (
    <picture className={className}>
      <source srcSet={`/images/app/${name}-dark.png`} media="(prefers-color-scheme: dark)" />
      <img
        src={`/images/app/${name}-light.png`}
        alt={`${alt}, example accounts`}
        width={width}
        height={height}
        loading={priority ? "eager" : "lazy"}
        fetchPriority={priority ? "high" : undefined}
        decoding="async"
        className="block h-auto w-full"
      />
    </picture>
  );
}
