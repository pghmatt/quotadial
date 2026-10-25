/**
 * HTML rendered from the repository's own Markdown (lib/markdown.ts), set in the site's reading
 * style: 65 to 72 characters, lists with room, code and addresses in the code face.
 */
export function Prose({ html, className = "" }: { html: string; className?: string }) {
  return (
    <div
      className={`max-w-[68ch] text-[1.0625rem] leading-[1.6] [&_a]:underline [&_a]:underline-offset-[0.2em] [&_code]:rounded-chart [&_code]:bg-shallow [&_code]:px-1 [&_code]:font-code [&_code]:text-[0.875em] [&_li]:mt-2 [&_p]:my-4 [&_strong]:font-[700] [&_ul]:my-4 [&_ul]:list-disc [&_ul]:pl-6 ${className}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
