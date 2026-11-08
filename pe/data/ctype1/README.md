# Bounded Windows CTYPE1 data

These 256 measured WORD classifications describe the Unicode repertoire obtained
by decoding all byte values through native Windows code page 1252. They implement
a finite supported NLS repertoire, not arbitrary Unicode classification.

Source: original MIT observer mode 13 in pe/fixtures/pe32-startup/guard.c,
unchanged PE SHA-256 `a65ecd30e93d9a25328809cf39de79856bbc8e90ab461a50d8dd90bed92e30e4`.
Executed natively by GitHub Actions run 42460883696 at revision
9e852ec1e4e7cac4844794e0f105c5c1767c516e, 2026-11-08:
https://github.com/daeken/Alkyhol/actions/runs/42460883696

The full unmodified native report is frozen in
pe/reference/pe32-startup-windows-v1.json (SHA-256
`e872d29c86d34d392d8481941e593d38c5cfab7e472fd802c6071d556dda8015`).
The independent published byte-to-Unicode mapping is pinned in
pe/data/windows-1252/README.md. The runtime table pairs each measured native
CTYPE1 result with that mapping's Unicode code unit. Each row is `unit flags`.

The runtime applies these data to any supported sequence, including duplicates
and different orders; other code units and CTYPE2/3 fail explicitly. These are
original probe observations of public API behavior, with no game bytes or data.
