import { CodeBlock } from "../CodeBlock.tsx";
import { doctorSample } from "./text.ts";

/** What `quotadial doctor` prints, as the command guide shows it, with the words that make it safe to quote. */
export function DoctorSample() {
  return (
    <figure className="m-0">
      <CodeBlock code={doctorSample()} copy={false} wrap={false} name="quotadial doctor output, example accounts" />
      <figcaption className="mt-4 max-w-[52ch] font-table text-[0.9375rem] text-ink-2 italic">
        What the command prints, from the command guide: example accounts, each address cut to its
        first letters and ending, and for each account what QuotaDial reads it from, but never a sign-in itself.
      </figcaption>
    </figure>
  );
}
