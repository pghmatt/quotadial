# Licence keys

QuotaDial Free needs no key. A licence key adds QuotaDial Pro (D-099). This page is for whatever
issues keys: the format, how QuotaDial checks one, and the tool that signs them. `docs/CLI.md` says
how a user adds one.

## What QuotaDial checks

A key is a licence signed with QuotaDial's Ed25519 key. QuotaDial carries the public half
(`QUOTADIAL` in `crates/core/src/licence.rs`) and checks a key against it on the machine, with no
network call, every time it reads the key. So:

- a key works offline, for ever, unless the licence it holds has a last day;
- nothing about a key is checked anywhere else, and a key cannot be turned off once issued;
- anyone holding the private half can issue keys, so it never enters this repository.

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
cargo run -p quotadial-issue -- keygen                          # make the signing key, once
cargo run -p quotadial-issue -- public                          # print its public half
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
- `sign` refuses a signing key whose public half QuotaDial does not carry, since its keys would not
  check out.

QuotaDial's signing key was made on Matt's Mac on 2026-10-24.

## Changing the signing key

To replace the key, make a new one, add its public half to `trusted()` in
`crates/core/src/licence.rs` beside the old one, and release. Keys signed with either then check
out. Removing the old public half later makes every key it signed stop working, so it stays as long
as those keys should.
