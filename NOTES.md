# Findings

The four questions `topcoat-full-stack-rust` (2026-08-02) promised to answer,
plus the one this pilot exists to test. **Write findings here while building,
not afterwards.** A reconstructed impression is worth nothing; the point of the
pilot is that it happened.

Rules for this file:

- Record what was observed, with the command or the error text. No summaries of
  things that were not run.
- Mark anything not yet tested as **untested**, not as absent.
- A finding that flatters Topcoat and one that does not are worth the same.

Versions: `topcoat` 0.6.2, `toasty` 0.10.0, `rustc` 1.97.1, macOS arm64.

---

## Q0 — Does the descriptor survive the move to Rust?

Not one of the four promised questions; it is the thesis of the pilot. See
`README.md`.

**Preliminary: yes, and the descriptor stayed data.** `TableDescriptor<T>` is a
`const` holding a `&'static [Column<T>]`, each column a `fn(&T) -> String`
accessor plus a closed `CellKind` enum. `view!` iterates it with an ordinary
`for`, and `table.rs` never names a domain type. First screen renders
server-side and correctly.

**Untested and the real test:** the second screen. Nothing is proven until a
different descriptor is added and `users` renders byte-identically. Do not
record a verdict here before then.

**Open, and the thing most likely to break it:** `fn` pointers cannot capture.
The moment a real screen needs a cell that closes over state — a permission
check, a locale, a formatter — this becomes `Box<dyn Fn>` and the descriptor
stops being `const` data. That moment is the finding, not a detour. Write down
what forced it.

## Q1 — Developer experience end to end

- **`#[component]` requires the returned future to be `Send`.** A generic
  component therefore needs bounds the React version never had:

  ```
  error[E0277]: `T` cannot be sent between threads safely
  error[E0277]: `T` cannot be shared between threads safely
  ```

  Fixed by `T: 'static + Send + Sync`. Worth noting fairly: the compiler
  suggested exactly that, and the suggestion was correct.

- **Untested:** `topcoat ui`, `topcoat fmt`, the CLI generally. Not installed.

## Q2 — Error messages when the macro rejects an expression

**Untested.** Nothing has been fed to `view!` that it refused yet. This needs
deliberate probing rather than waiting for an accident: try a non-`Send` value,
a borrow that outlives the view, a `$(...)` expression that cannot cross to
JavaScript. The interesting question is whether the error points at the source
line or at macro-expanded code.

## Q3 — Does the fast-rebuild loop hold up?

First numbers, this project only — small, three modules, one dependency tree:

| Change | Time |
|---|---|
| touch `table.rs`, `cargo build` | **1.38 s** |
| `cargo clean -p` + rebuild (crate only, deps cached) | **1.80 s** |

Honest framing: this is a scaffold, not an app. The number that matters is what
this looks like at twenty components, and whether the `view!` macro expansion
cost grows with template size. Re-measure and add rows as the pilot grows —
a single early number proves nothing except that it is not pathological at
this size.

**Untested:** the dev-server reload loop (`topcoat::dev::script()` is wired but
the browser-reload behaviour has not been exercised), and a cold build from an
empty cargo registry.

## Q4 — Where the trade-offs land

Nothing earned yet. Toasty is a declared dependency but unused — screen one runs
on fixture data in `users::rows()`. Persistence is step two and the trade-offs
question cannot be answered before then.

---

## For the CI/CD post (second post, not the same one)

Kept separate deliberately — do not merge these into the descriptor post.

- Upstream roadmap has **"Docs for how to deploy Topcoat" unchecked**, so
  deployment is unsolved upstream rather than merely undocumented by us.
- No Node in the pipeline is the claim worth testing against a real workflow.
- What is the deployable artefact, and what does the asset bundler need at
  build time versus run time?
