---
tags: soft3, foculus, radio, launch, audit
date: 2026-09-24
---
# Row 21: soft3 cannot yet depend on foculus's settle radio

[Launch tracker](../../cyber/launch.md) row 21 names its remaining gap as
"radio wired as an actual dependency of the soft3 node." `foculus`'s
`net` feature already ships a working transport for this,
[`radio_settle::SettleRadio`](../../foculus/src/radio_settle.rs) — a real
iroh QUIC endpoint that publishes and receives
[`gossip::SettleMsg`](../../foculus/src/gossip.rs), with its own passing
two-endpoint test (`radio_settle::tests::two_endpoints_exchange_self_acc`).
Adding it to soft3 looked like a small wiring step: a `radio` feature
turning on `foculus/net`, plus a thin `soft3::radio::start_settle_radio`
wrapper.

It does not build. From a clean worktree off `origin/main`
(revision `5272f6c`), enabling the dependency chain that this wiring needs:

```
$ RUSTC_BOOTSTRAP=1 cargo check --features net   # in foculus, isolated
...
error[E0277]: `?` couldn't convert the error to `ed25519::pkcs8::Error`
   --> ed25519-dalek-3.0.0-pre.1/src/signing.rs:714:57
error[E0308]: mismatched types
   --> ed25519-dalek-3.0.0-pre.1/src/signing.rs:717:28
error: could not compile `ed25519-dalek` (lib) due to 2 previous errors
```

This is the same root cause [foculus#31](https://github.com/cyberia-to/foculus/pull/31)
already documents from foculus's own side: `iroh`/`iroh-base` 0.96.1 (upstream,
crates.io) pin `ed25519-dalek = "=3.0.0-pre.1"` exactly, and that pre-release
has a real bug in its own `pkcs8` feature. `cargo update -p ed25519-dalek
--precise 3.0.0` cannot escape the pin (`iroh-base` names it as the blocker);
`iroh` 1.2 resolves to the fixed `ed25519-dalek` but renames the
`address-lookup-mdns` feature `SettleRadio` depends on, which foculus#31 named
as a real API migration, not attempted there.

This audit confirms the same wall from the consumer side: any crate that
turns on `foculus/net` — not just foculus itself — inherits this break,
because `ed25519-dalek` fails to compile before feature-specific code is
ever reached. Wiring soft3 to `foculus::radio_settle::SettleRadio` is
correctly scoped as a small step once the dependency compiles, but it
cannot land, green or draft-and-red, before foculus#31's `iroh` 0.96 → 1.2
migration lands in foculus itself.

Remains: once foculus#31's migration merges, add a `radio` feature to
`soft3/crate/Cargo.toml` (`radio = ["foculus/net"]`) and a
`soft3::radio::start_settle_radio(data_dir, port)` wrapper over
`SettleRadio::start`, with a two-endpoint `start_memory` gossip test
mirroring `radio_settle`'s own — that shape was written and reverted here
once the build broke (not committed, since committing it would ship a
feature no one can `cargo check` green); the next worker on this row can
carry it forward once the migration lands.
