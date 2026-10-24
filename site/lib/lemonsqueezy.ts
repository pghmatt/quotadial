/**
 * Lemon Squeezy's License API, as the key page asks it (#54): `POST /v1/licenses/validate` with the
 * key from a receipt. It needs no API key; it answers JSON, with a 4xx status and `valid` false for
 * a key it does not know.
 */

/** Where Lemon Squeezy validates a key, over TLS. */
export const VALIDATE_URL = "https://api.lemonsqueezy.com/v1/licenses/validate";

/** The part of a validation answer the key page reads. */
export interface Validation {
  valid: boolean;
  license_key: {
    id: number;
    status: string;
    created_at: string;
    expires_at: string | null;
  } | null;
  meta: {
    store_id: number;
    product_id: number;
  } | null;
}

/** How the page fetches, which tests replace with recorded answers. */
export type Fetch = (url: string, init: RequestInit) => Promise<Response>;

/** Why Lemon Squeezy gave no answer the page can use. */
export type Unanswered = "unreachable" | "unreadable";

/** Asks Lemon Squeezy about `orderKey`. A 4xx answer is still an answer: it says the key is not valid. */
export async function validate(orderKey: string, fetcher: Fetch): Promise<Validation | Unanswered> {
  let response: Response;
  try {
    response = await fetcher(VALIDATE_URL, {
      method: "POST",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({ license_key: orderKey }).toString(),
      redirect: "error",
    });
  } catch {
    return "unreachable";
  }
  if (response.status >= 500) {
    return "unreachable";
  }
  try {
    const answer: unknown = await response.json();
    return readValidation(answer) ?? "unreadable";
  } catch {
    return "unreadable";
  }
}

/** `answer` as a validation, if it has the shape Lemon Squeezy documents. */
function readValidation(answer: unknown): Validation | null {
  if (typeof answer !== "object" || answer === null) {
    return null;
  }
  const { valid, license_key: key, meta } = answer as Record<string, unknown>;
  if (typeof valid !== "boolean") {
    return null;
  }
  const record = (value: unknown) =>
    typeof value === "object" && value !== null ? (value as Record<string, unknown>) : null;
  const k = record(key);
  const m = record(meta);
  return {
    valid,
    license_key:
      k &&
      typeof k.id === "number" &&
      typeof k.status === "string" &&
      typeof k.created_at === "string" &&
      (typeof k.expires_at === "string" || k.expires_at === null)
        ? {
            id: k.id,
            status: k.status,
            created_at: k.created_at,
            expires_at: k.expires_at as string | null,
          }
        : null,
    meta:
      m && typeof m.store_id === "number" && typeof m.product_id === "number"
        ? { store_id: m.store_id, product_id: m.product_id }
        : null,
  };
}
