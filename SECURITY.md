# Security policy

## Reporting a vulnerability

Report it privately, through this repository's
[private vulnerability reporting](https://github.com/pghmatt/quotadial/security/advisories/new),
or by email to support@quotadial.com with "Security" in the subject. Please do not open a public
issue or pull request for it.

Say what you found, how to make it happen, and what it lets someone do. You will hear back within
3 working days. A vulnerability that exposes a sign-in, a token or what QuotaDial keeps, or that
lets another user, a website or an update act as the user, is fixed within 14 days where it can
be, in a release whose notes credit you unless you would rather they did not. Anything else is
fixed in the next release.

## Supported versions

Fixes go into the newest release only. The app updates itself when you choose Update, after
checking the update's signature ([`docs/INSTALL.md`](docs/INSTALL.md#updates)); the `quotadial`
command updates the way it was installed.

## How QuotaDial protects you

[`docs/SECURITY.md`](docs/SECURITY.md) is the threat model: what QuotaDial reads, sends and keeps,
and what an attacker on the same machine, on the same network or behind a website can and cannot
do. [`docs/PRIVACY.md`](docs/PRIVACY.md) lists every address it connects to, and nothing else.
