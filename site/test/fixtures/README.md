# Lemon Squeezy fixtures

`validate-active.json` is the example response of `POST /v1/licenses/validate` from Lemon Squeezy's
documentation, as published at
https://docs.lemonsqueezy.com/api/license-api/validate-license-key on 2026-10-24, unchanged. The
tests derive the other answers from it by changing one field each, such as `license_key.status`,
and the error answers from the documented shape: a 4xx status with `valid` false and an `error`.

When QuotaDial's store exists, a response recorded in test mode, with its customer's name and email
address replaced, joins it here (#51).
