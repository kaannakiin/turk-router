# Security

## Reporting a vulnerability

Report privately through GitHub's advisory form on this repository, under Security then Report a
vulnerability. Please do not open a public issue for anything that affects funds.

Include what you need to reproduce it: the instruction bytes, the account list, and the wire epoch
from `wire/wire-manifest.json`.

## What this package can and cannot do to you

It builds one instruction and returns it. It never signs, never sends, never reads the chain, and
never touches a key. A bug here can produce an instruction that the program rejects, or one that
routes through pools you did not intend to name. It cannot move funds on its own, because it never
holds the authority to.

Check what you sign. The instruction this package returns names your token accounts and the pools
you chose; both are visible in the account list before you sign anything.

## Scope

Vulnerabilities in this package's encoding are in scope. The deployed program is a separate surface
with its own review, and its source is not published here.
