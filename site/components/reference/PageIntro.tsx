import type { ReactNode } from "react";

/**
 * The top of a reference page: the one H1, a short answer beneath it and the page's actions. The
 * same size and spacing on /privacy, /support and /changelog, so the three read as one set.
 */
export function PageIntro({
  title,
  answer,
  children,
}: {
  title: string;
  answer: ReactNode;
  /** The actions, primary first; none on a page that has no action of its own. */
  children?: ReactNode;
}) {
  return (
    <div>
      <h1 className="m-0 text-[2.75rem] leading-[1.02] font-[800] tracking-[-0.01em] md:text-[4rem]">{title}</h1>
      <p className="mt-5 mb-0 max-w-[40ch] text-[1.25rem] leading-[1.45] text-ink-2">{answer}</p>
      {children && <div className="mt-8 flex flex-wrap items-center gap-x-6 gap-y-4">{children}</div>}
    </div>
  );
}
