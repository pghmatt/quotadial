import { CopyButton } from "./CopyButton.tsx";

/** Commands or output, in JetBrains Mono, with a button that copies them. */
export function CodeBlock({ code, label = "Copy", copy = true }: { code: string; label?: string; copy?: boolean }) {
  return (
    <div className="rounded-chart bg-mid p-4 text-ink">
      <pre className="m-0 overflow-x-auto font-code text-[0.9375rem] leading-[1.6]">
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
