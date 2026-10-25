import { Plate } from "../Plate.tsx";

/** What the website itself does, beside the privacy text: the text covers the app, this covers the pages. */
export function WebsiteNote({ className = "" }: { className?: string }) {
  return (
    <Plate tone="shallow" className={className}>
      <h2 className="m-0 text-[1.75rem] leading-[1.1] font-[750]">This website</h2>
      <p className="mt-3 mb-0 max-w-[42ch]">
        quotadial.com sets no cookies, runs no analytics and loads no scripts from other sites. Its
        fonts and pictures are served from the same place as its pages.
      </p>
    </Plate>
  );
}
