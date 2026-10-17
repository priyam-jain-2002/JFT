# Synthetic NSE CM fixtures

Only synthetic, non-licensed golden fixtures belong here.

These records were authored for repository tests. They are not copied from NSE data and must not be treated as representative market evidence.

SHA-256 covers the complete tracked file, including its single trailing LF byte.

| Fixture | Product / record | Payload + LF | SHA-256 |
|---|---|---:|---|
| `trim_order_entry.dat` | Current CM Trim Order | 68 + 1 bytes | `85a1f60822ac5f4b6b328576faccec15d2527fbb776a9949c07c1b29087a9197` |
| `trim_order_modify.dat` | Current CM Trim Order modify | 68 + 1 bytes | `a6b951a722cb85ab351497e7d20ee2b9673d501bc91fa718d836a7b602142aa9` |
| `trim_order_cancel.dat` | Current CM Trim Order cancel | 68 + 1 bytes | `50b10492773f4e93d74240b99916767bcb34ec8d194b772fff136192c7c9379d` |
| `trim_trade.dat` | Current CM Trim Trade | 82 + 1 bytes | `90faddba0142a3d12c20ad1c960cedb955f0c2817740cba11e2e5a163f3b03f4` |
| `full_order_entry.dat` | Current CM Full Order | 91 + 1 bytes | `779315b32d835e43ccee2405821babe3d5af5077c0163db4fcb7f02212781e7e` |
| `full_trade.dat` | Current CM Full Trade | 103 + 1 bytes | `b917c60f11eb9074edef52d0e9d10af3d0cc867980867f2edb845658c74a97f9` |

Recompute from the repository root with:

```sh
shasum -a 256 tests/fixtures/nse_cm/synthetic_nonlicensed/*.dat
wc -c tests/fixtures/nse_cm/synthetic_nonlicensed/*.dat
```

Future fixtures should cover remaining sessions and lifecycle transitions, validated jiffy boundaries, same-jiffy groups, malformed records, and replay anomalies only after each semantic case is reviewed. Any fixture edit must update this inventory in the same commit and explain why its golden identity changed.
