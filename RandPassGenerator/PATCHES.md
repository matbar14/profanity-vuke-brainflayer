# Local patches to upstream RandPassGenerator

Vendored from https://github.com/nsacyber/RandPassGenerator
(US Government work, CC0 1.0 — see `LICENSE.md`).

Upstream commit at time of vendoring: `614eb1ed9d130dfa96bf372775445bac2449da92` (master, 2023-12-22).

## `-nokeyid`: raw key stream output

**File:** `src/gov/nsa/ia/pass/RandPassGenerator.java`

Upstream `-k` mode prints each generated key followed by a bookkeeping
block:

```
<64 hex chars>
Key ID:
<hash of the key>
```

This is useless — worse, harmful — when the keys are piped into another
tool: the `Key ID:` lines corrupt the key stream. This fork adds:

- CLI option `-nokeyid` — suppress the `Key ID:` lines;
- `setSuppressKeyIDs(boolean)` — the same programmatically.

With `-nokeyid` and `-str 256`, `-k` mode emits exactly what brainflayer
expects for `-x -t priv`: one 64-character lowercase hex private key per
line:

```
java -jar RandPassGenerator/build/jar/PassGenerator.jar -k 100000 -str 256 -nokeyid \
  | ./brainflayer-MultiBlooms/brainflayer -x -t priv -v -b bitcoin.blf
```

Everything else is upstream code, unmodified.

## `build_noant.py`: build without Apache Ant

Upstream builds with `ant jar`. `build_noant.py` reproduces the same
result (`build/jar/PassGenerator.jar`, classes + `wordlist.txt` at the
jar root + `Main-Class` manifest) using plain `javac` + Python's zipfile
for environments without Ant. It prefers `javac`; if absent, it can use
the Eclipse batch compiler via `ECJ_JAR=/path/to/ecj.jar`.
