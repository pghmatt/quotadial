/**
 * QuotaDial's licence keys, as the key page signs them (D-099, #54). A key is `QD1.`, the licence
 * as base64url JSON, a dot, and the base64url Ed25519 signature of everything before the second
 * dot, exactly as `crates/core/src/licence.rs` writes and checks them. The JSON has its fields in
 * alphabetical order and no spaces, as `quotadial-issue` writes it, so both sign one licence to
 * one key, byte for byte.
 */

/** A licence for QuotaDial Pro. */
export interface Licence {
  /** Such as `ls-1234`. */
  id: string;
  /** The day it was issued, `YYYY-MM-DD`. */
  issued: string;
  /** Its last day, `YYYY-MM-DD`, for a licence that ends. */
  until?: string;
}

const PREFIX = "QD1";

/** The DER before an Ed25519 private key's 32 bytes in a PKCS #8 file (RFC 8410). */
const PKCS8_ED25519 = Uint8Array.of(
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
);

/** `bytes` as base64url without padding. */
export function base64url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

/** The JSON a licence is signed as: alphabetical, with no spaces. */
export function written(licence: Licence): string {
  const fields: Record<string, string> = {
    id: licence.id,
    issued: licence.issued,
    plan: "pro",
  };
  if (licence.until) {
    fields.until = licence.until;
  }
  return JSON.stringify(fields);
}

/** The signing key in the text of a PKCS #8 PEM file, as `quotadial-issue keygen` writes it. */
export async function signingKey(pem: string): Promise<CryptoKey> {
  const body = pem
    .split(/\r?\n/)
    .filter((line) => !line.startsWith("-----"))
    .join("")
    .trim();
  const der = Uint8Array.from(atob(body), (character) => character.charCodeAt(0));
  const ed25519 =
    der.length === PKCS8_ED25519.length + 32 &&
    PKCS8_ED25519.every((byte, index) => der[index] === byte);
  if (!ed25519) {
    throw new Error("The issuing key is not an Ed25519 private key in PKCS #8.");
  }
  return crypto.subtle.importKey("pkcs8", der, { name: "Ed25519" }, false, ["sign"]);
}

/** The licence key for `licence`, signed with `key`. */
export async function sign(licence: Licence, key: CryptoKey): Promise<string> {
  const encoder = new TextEncoder();
  const signed = `${PREFIX}.${base64url(encoder.encode(written(licence)))}`;
  const signature = await crypto.subtle.sign({ name: "Ed25519" }, key, encoder.encode(signed));
  return `${signed}.${base64url(new Uint8Array(signature))}`;
}
