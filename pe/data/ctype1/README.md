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

## Neutral-locale case mapping

case-map.txt records `unit lower upper` for LCMapStringW locale 0, flags
0x100/0x200 only. Each single-unit result is one UTF-16 code unit; concatenation
agrees with the independently observed complete 256-unit batch in both directions.
The original MIT nls.c mode 3 ran natively at revision
9a51997571b6a68911fa3a6cf86bd4c8ad1330a8 in run 42462059740:
https://github.com/daeken/Alkyhol/actions/runs/42462059740
Its unchanged PE SHA-256 is `b5ce1b858ae3bc4d88542c709c2d2b5815b435f2f4bd381e36309741db2a814f`.
Full unmodified native capture: pe/reference/pe32-lcmap-windows-v1.json, SHA-256
`e22977b5e30c49163aeeb8577f88d7ebc4bb6cc0919809c729c5f025b959635a`.

These are measured bounded mappings, not claims about other locales, Unicode
repertoires, sort keys or linguistic/context-sensitive flags. In particular the
observed uppercase sharp-s remains sharp-s; the florin uppercases to U+0191.
