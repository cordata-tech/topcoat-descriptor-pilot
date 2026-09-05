# topcoat-descriptor-pilot

A small admin panel in [Topcoat](https://github.com/tokio-rs/topcoat) + Toasty,
built to answer one question and settle a promise.

## The question

[*One skeleton, many screens*](https://cordata.tech/en/blog/one-skeleton-many-screens)
argued that admin screens should be declared as typed data, rendered by a
framework that knows nothing about the domain — and it stated the test as
stack-independent:

> draw the boundary as a **one-way dependency** — domains depend inward on a
> framework that knows nothing about them — and validate it with a **zero-diff
> refactor** of a screen you already trust.

That was React and TypeScript, where a descriptor is a runtime value and a
column can hold a render function. Topcoat's `view!` is a macro. So:

**Can a screen descriptor stay *data* in Rust, or does the macro drag it back
into being code — and does the zero-diff test still pass?**

This repo is the experiment, not a demo. It can come out negative; a negative
result is the more interesting post.

## The test

1. Build one screen and learn to trust it. ✅ `src/users.rs`
2. Add a second screen fed by a different descriptor.
3. **If screen one changes at all, the abstraction is wrong.**

Step 2 is deliberately not done yet. A zero-diff test means nothing if both
screens were written at once.

## Layout

```
src/descriptor.rs   the descriptor types — the boundary itself
src/table.rs        the framework half; must never name a domain type
src/users.rs        screen one: a domain, declared
src/main.rs         router + page
```

The rule that makes the experiment meaningful: **`table.rs` may not mention
`User`, `Invoice`, or any other domain type.** If it ever has to, that is the
result — record it in `NOTES.md` rather than working around it.

## Running

```bash
cargo run           # then http://localhost:3000
```

## Findings

In [`NOTES.md`](./NOTES.md), written while building rather than reconstructed
afterwards. It records what has *not* been tested as carefully as what has.

## Why this exists

The Topcoat intro post promised a follow-up reporting where the rough edges
actually show. That post cannot be written from the announcement, and it
cannot be written from imagination — so this gets built first. Tracked on
[cordata-tech/platform#34](https://github.com/cordata-tech/platform/issues/34).

Versions pinned at the start: `topcoat` 0.6.2, `toasty` 0.10.0.
