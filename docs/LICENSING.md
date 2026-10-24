# Licence keys

QuotaDial Free needs no key. A licence key adds QuotaDial Pro (D-099). This page is for whatever
issues keys: the format, how QuotaDial checks one, the tool that signs them, and the key page that
turns a paid Lemon Squeezy order into one (D-102). `docs/CLI.md` says how a user adds a key.

## What QuotaDial checks

A key is a licence signed with one of QuotaDial's two Ed25519 keys, whose public halves QuotaDial
carries in `crates/core/src/licence.rs`:

- `QUOTADIAL`, the root key. Its private half stays offline, on Matt's Mac and in his password
  manager, and signs only by hand.
- `ISSUER`, the issuing key. Its private half is the key page's secret, online, so it is the one
  most likely to leak, and it can be replaced (see If the issuing key leaks).

QuotaDial checks a key against both on the machine, with no network call, every time it reads the
key. So:

- a key works offline, for ever, unless the licence it holds has a last day;
- nothing about a key is checked anywhere else, and a key cannot be turned off once issued;
- anyone holding a private half can issue keys, so neither ever enters this repository.

A key that does not check out, or whose licence has ended, makes the copy QuotaDial Free, with one
sentence saying why. It never stops QuotaDial.

## The format

```
QD1.<licence>.<signature>
```

- `QD1` names the format. A reader takes it in any case, and a later format is `QD2` and so on: a
  version that meets one it does not know asks to be updated.
- `<licence>` is the licence as JSON, encoded as base64url without padding.
- `<signature>` is the 64-byte Ed25519 signature (RFC 8032) of the ASCII text `QD1.<licence>`,
  encoded as base64url without padding.

The licence's fields:

| Field | Meaning |
| --- | --- |
| `id` | The licence's id, such as the order it came from: 1 to 64 printable ASCII characters, no spaces. Shown to the user |
| `plan` | `pro`. A plan a version does not know asks for an update |
| `issued` | The day the licence was issued, `YYYY-MM-DD` |
| `until` | Its last day, `YYYY-MM-DD`, for a licence that ends. It holds through that day, by the earlier of the user's local and the UTC date. Left out for one that does not end |

QuotaDial checks the signature over the bytes as written, so the JSON's field order and spacing are
the issuer's choice, and a field it does not know is kept in the signature and otherwise left
alone. `quotadial-issue` writes the fields in alphabetical order with no spaces.

A reader leaves out spaces, line breaks and invisible characters (zero-width spaces, joiners, soft
hyphens and byte-order marks) before anything else, since a key is often copied from an email that
wrapped it. It keeps the key on one line, with `QD1` in capitals.

## The test key

`TEST_SEED` in `crates/core/src/licence.rs` is a signing key whose private half is public on
purpose, so tests can sign licences. Only a debug build trusts it: a release build trusts only
QuotaDial's key, which CI checks by running `only_a_debug_build_trusts_the_test_key` in a release
build. These two keys are signed with it, and `crates/core` checks that QuotaDial signs them byte
for byte the same, so an issuer in another language can check its output against them:

```
QD1.eyJpZCI6InRlc3QtMDAwMSIsImlzc3VlZCI6IjIwMjYtMTAtMjQiLCJwbGFuIjoicHJvIn0.TCnQdA2XJTPnuGVV280SOgfy_BH4maU7Nj8s11TSw6nWbpPT_icIfL-nf155_ZQXnAnvGsKDQ14V3X-72YchDw
```

is `{"id":"test-0001","issued":"2026-10-24","plan":"pro"}`, and

```
QD1.eyJpZCI6InRlc3QtMDAwMiIsImlzc3VlZCI6IjIwMjYtMTAtMjQiLCJwbGFuIjoicHJvIiwidW50aWwiOiIyMDI3LTEwLTI0In0.tlTkpHCVT6JLE_iVJEanK4PgzTLBxrwz9IApnRFCHLPO87jnrYkVKrNZvnEAWKvfK_NG_az9TK3xkY9fa3k3Dw
```

is `{"id":"test-0002","issued":"2026-10-24","plan":"pro","until":"2027-10-24"}`.

## Signing keys with `quotadial-issue`

`crates/issue` is a small tool for whoever issues keys. No release carries it.

```sh
cargo run -p quotadial-issue -- keygen                          # make the root key, once
cargo run -p quotadial-issue -- keygen --key-file "$HOME/Library/Application Support/quotadial-issue/issuing-key.pem"
cargo run -p quotadial-issue -- public                          # print the root key's public half
cargo run -p quotadial-issue -- sign --id order-1234            # a key that does not end
cargo run -p quotadial-issue -- sign --id order-1234 --until 2027-10-24
cargo run -p quotadial-issue -- sign --test --id test-1         # with the test key
cargo run -p quotadial-issue -- check QD1....                   # what a key holds, and who signed it
```

- `keygen` writes the private half to `quotadial-issue/signing-key.pem` in the system's config
  folder (`~/Library/Application Support` on macOS), or to `--key-file`. The file is PKCS #8, which
  OpenSSL and WebCrypto both read, and readable only by its owner. It is never replaced and never
  printed. Keep a copy somewhere safe, such as a password manager: without it no new keys can be
  made, though every key already issued keeps working.
- `public` prints the public half as base64 and as the bytes `crates/core` keeps.
- `sign` and `public` take `--key-file` for the issuing key, as `keygen` does. `sign` refuses a
  signing key whose public half QuotaDial does not carry, since its keys would not check out, and
  `public` and `check` name which of QuotaDial's keys it is.

Both keys were made on Matt's Mac on 2026-10-24, in `signing-key.pem` (the root) and
`issuing-key.pem` beside it.

## The key page

Lemon Squeezy sells QuotaDial Pro and makes its own licence keys, UUIDs such as
`38b1460a-5104-4067-a91d-77b872934d51`, which cannot be QuotaDial's. The key page turns one into a
QuotaDial key (#54):

1. Lemon Squeezy's receipt email has a button, "Get your QuotaDial key", linking to
   `https://quotadial.com/key#licence=[license_key]`. Lemon Squeezy fills in the key. It goes in the
   fragment, which a browser never sends with the page, so it stays out of request logs and
   history sync.
2. The page sends it to its function, `POST /api/key` (`site/functions/api/key.ts`), as
   `{ "licence": "<key>" }`.
3. The function asks Lemon Squeezy's License API, `POST https://api.lemonsqueezy.com/v1/licenses/validate`,
   over TLS, following no redirect. It signs only when every check passes, and otherwise answers
   one sentence that says what to do:
   - `valid` is true;
   - `license_key.status` is `inactive` or `active`. Lemon Squeezy calls a key `inactive` when it
     is "valid but has no activations", as every new key is, and QuotaDial never activates one;
     `expired` and `disabled` are refused;
   - `meta.store_id` and `meta.product_id` are QuotaDial's, from the page's environment.
4. The licence is `ls-<license_key.id>`, issued on `created_at`'s day, until `expires_at`'s day
   when there is one. Signing is deterministic, so the same receipt always gives the same key, and
   the page can be visited again to get it back. Nothing is logged or kept, and the answer is
   never cached.

The function needs, in Cloudflare Pages:

| Name | Kind | What |
| --- | --- | --- |
| `LICENCE_ISSUING_KEY` | Secret | The issuing key's PKCS #8 file, as `wrangler pages secret put LICENCE_ISSUING_KEY --project-name quotadial < "$HOME/Library/Application Support/quotadial-issue/issuing-key.pem"` sends it. Never the root key |
| `LS_STORE_ID` | Variable | QuotaDial's store id at Lemon Squeezy |
| `LS_PRODUCT_ID` | Variable | QuotaDial Pro's product id |

Without them it answers that it isn't set up yet. A Cloudflare rate-limiting rule on `/api/key`,
about 10 requests a minute from one address, keeps it from being a free oracle for Lemon Squeezy's
validate endpoint. Its tests (`site/test`, run in CI) use Lemon Squeezy's documented answer and the
test key, and sign the two keys above byte for byte as QuotaDial does.

A key pasted into QuotaDial that is a Lemon Squeezy key says where QuotaDial's key is.

## If the issuing key leaks

Whoever holds the issuing key's private half can make keys that check out. Then:

1. Make a new issuing key with `keygen --key-file` into a new file.
2. Put its public half in `ISSUER`, in place of the old one, and release.
3. Put its private half in the key page's secret in place of the old one.

Keys the old issuing key signed stop working in the new release, and each honest buyer gets a new
key by opening their receipt's link again, since the page signs the same licence again with the new
key. The root key is untouched, and so is every key it signed by hand.

## Changing the root key

To replace the root key, make a new one, add its public half to `trusted()` in
`crates/core/src/licence.rs` beside the old one, and release. Keys signed with either then check
out. Removing the old public half later makes every key it signed stop working, so it stays as long
as those keys should.
