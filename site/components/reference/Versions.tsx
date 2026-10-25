import { InlineLink } from "./InlineLink.tsx";
import { Plate } from "../Plate.tsx";
import { Prose } from "../Prose.tsx";
import { SITE } from "../../lib/data/site.ts";
import type { ChangelogVersion } from "../../lib/markdown.ts";
import { sayDate, versionId } from "./text.ts";

/** One version: its name and date beside its changes, grouped under Added, Changed and Fixed. */
function Version({ version }: { version: ChangelogVersion }) {
  const unreleased = version.date === null;
  const id = versionId(version.version);
  return (
    <article aria-labelledby={id} className="grid gap-6 lg:grid-cols-[14rem_minmax(0,1fr)] lg:gap-10">
      <div className="lg:sticky lg:top-6 lg:self-start">
        <h2 id={id} className="m-0 scroll-mt-6 text-[2rem] leading-[1.05] font-[750]">
          {version.version}
        </h2>
        {unreleased ? (
          <p className="mt-2 mb-0 text-ink-2">
            Not yet released. These changes are in QuotaDial&rsquo;s code and not yet in a version you can
            download.
          </p>
        ) : (
          <>
            <p className="mt-2 mb-0 text-ink-2">
              <time dateTime={version.date ?? undefined}>{sayDate(version.date ?? "")}</time>
            </p>
            <p className="mt-3 mb-0">
              <InlineLink href={`${SITE.releases}/tag/v${version.version}`}>Release notes for {version.version} on GitHub</InlineLink>
            </p>
          </>
        )}
      </div>
      <div className="grid gap-8">
        {version.sections.map((section) => (
          <div key={section.heading}>
            <h3 className="m-0 text-[1.375rem] leading-[1.15] font-[700]">{section.heading}</h3>
            <Prose html={section.html} />
          </div>
        ))}
      </div>
    </article>
  );
}

/** The changelog's versions, newest first. A version not released yet sits on a plate of its own tint. */
export function Versions({ versions }: { versions: ChangelogVersion[] }) {
  return (
    <div className="grid gap-16 md:gap-20">
      {versions.map((version) =>
        version.date === null ? (
          <Plate key={version.version} tone="shallow">
            <Version version={version} />
          </Plate>
        ) : (
          <div key={version.version} className="border-t border-rule pt-10 md:pt-14">
            <Version version={version} />
          </div>
        ),
      )}
    </div>
  );
}
