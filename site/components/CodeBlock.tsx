import { CopyButton } from "./CopyButton.tsx";

/**
 * Commands or output, in JetBrains Mono, with a button that copies them. Long lines wrap, so a
 * phone shows the whole command; `wrap={false}` keeps columns of output aligned and scrolls them
 * sideways instead, as a region the keyboard can reach.
 */
export function CodeBlock({
  code,
  label = "Copy",
  copy = true,
  wrap = true,
  name,
}: {
  code: string;
  label?: string;
  copy?: boolean;
  wrap?: boolean;
  /** What a screen reader calls the block when it scrolls, such as "quotadial status output". */
  name?: string;
}) {
  return (
    <div className="rounded-chart bg-mid p-4 text-ink">
      <pre
        className={`m-0 font-code text-[0.9375rem] leading-[1.6] ${wrap ? "whitespace-pre-wrap [overflow-wrap:anywhere]" : "overflow-x-auto"}`}
        tabIndex={wrap ? undefined : 0}
        role={wrap ? undefined : "region"}
        aria-label={wrap ? undefined : name}
      >
        <code>{code}</code>
      </pre>
      {copy && (
        <div className="mt-3">
          <CopyButton text={code} label={label} />
        </div>
      )}
    </div>
  );
}
