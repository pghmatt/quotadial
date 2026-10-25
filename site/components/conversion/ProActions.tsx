import { ButtonLink } from "../Button.tsx";

/**
 * Pro's primary action. With a checkout link (Matt's to add, QUOTADIAL_CHECKOUT_URL at build time) it
 * is "Buy QuotaDial Pro" to Lemon Squeezy, with Free beside it. Without one it says buying opens
 * soon and offers QuotaDial Free, so no button leads nowhere.
 */
export function ProActions({ checkout }: { checkout: string | null }) {
  const free = {
    href: "/download",
    "data-event": "download_click",
    "data-event-system": "unknown",
    "data-event-from": "pro",
  } as const;
  if (checkout) {
    return (
      <>
        <div className="mt-7 flex flex-wrap items-center gap-x-4 gap-y-4">
          <ButtonLink href={checkout} data-primary-action="" data-event="checkout_click" data-event-from="pro">
            Buy QuotaDial Pro
          </ButtonLink>
          <ButtonLink variant="secondary" {...free}>
            Download QuotaDial Free
          </ButtonLink>
        </div>
        <p className="m-0 mt-5 max-w-[40ch] text-ink-2">
          Lemon Squeezy takes the payment and shows the price and the tax at its checkout.
        </p>
      </>
    );
  }
  return (
    <>
      <p className="m-0 mt-6 max-w-[40ch] font-display text-[1.375rem] leading-[1.2] font-[650]">
        Buying QuotaDial Pro opens soon.
      </p>
      <div className="mt-5 flex flex-wrap items-center gap-x-6 gap-y-4">
        <ButtonLink data-primary-action="" {...free}>
          Download QuotaDial Free
        </ButtonLink>
        <ButtonLink variant="text" href="#how-buying-works">
          See how buying will work
        </ButtonLink>
      </div>
    </>
  );
}
