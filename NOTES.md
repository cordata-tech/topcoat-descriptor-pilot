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

**Who wrote what**, because the post depends on it: the scaffold, the
hand-written `invoices` screen and the zero-diff harness were written by
Claude under direction. The Q1 findings below are therefore *its* experience
of the framework, not László's — the post must say so, or the refactor and
the notes that come out of it have to be his. Do not blur this.

---

## Q0 — Does the descriptor survive the move to Rust?

Not one of the four promised questions; it is the thesis of the pilot. See
`README.md`.

**Preliminary: yes, and the descriptor stayed data.** `TableDescriptor<T>` is a
`const` holding a `&'static [Column<T>]`, each column a `fn(&T) -> String`
accessor plus a closed `CellKind` enum. `view!` iterates it with an ordinary
`for`, and `table.rs` never names a domain type. First screen renders
server-side and correctly.

**Screen two now exists, hand-written — `src/invoices.rs`.** Deliberately
harder than `users`, which is all Text/Number/Date/Badge and would prove
nothing. Invoices carries three things `CellKind` and `fn(&T) -> String`
cannot currently express:

1. a currency amount formatted for a locale — `4 850,00 Kč`, cs-CZ grouping
   and decimal mark. **An `fn` pointer cannot capture a locale.**
2. a column computed from two fields (`days_late`, rendered as `—` when zero)
3. a cell that is a link, not text

Baselines captured for both routes. **The refactor onto the descriptor is the
experiment and has not been done.**

**Untested and the real test:** whether that refactor reproduces
`baseline/invoices.html` byte for byte, and leaves `users` untouched. Nothing is proven until a
different descriptor is added and `users` renders byte-identically. Do not
record a verdict here before then.

**Open, and the thing most likely to break it:** `fn` pointers cannot capture.
The moment a real screen needs a cell that closes over state — a permission
check, a locale, a formatter — this becomes `Box<dyn Fn>` and the descriptor
stops being `const` data. That moment is the finding, not a detour. Write down
what forced it.

## The zero-diff harness

`scripts/zero-diff.sh capture` before a refactor, `check` after. Because
Topcoat renders on the server, the zero-diff claim is a text comparison rather
than an eyeballed visual one — `diff` returns nothing or the abstraction is
wrong. That is a real advantage of this version of the experiment over the
React original, and worth saying in the post.

**The harness lied once before it was trusted, which is the point of proving a
test discriminates rather than assuming it.** Perturbing a column header
correctly produced `CHANGED` (exit 1). Reverting it with `mv` then *still*
produced `CHANGED` — because `mv` restores the original mtime, leaving the
source older than the compiled binary, so cargo skipped the rebuild and the
server kept serving the previous build. Nothing reported a skipped rebuild.
`touch` fixed it; the script now forces it every run, which costs ~1.4s.

Same shape as the traps this project exists to write about: a check that is
confidently wrong because a step upstream silently did not happen.

## Q1 — Developer experience end to end

- **`#[component]` requires the returned future to be `Send`.** A generic
  component therefore needs bounds the React version never had:

  ```
  error[E0277]: `T` cannot be sent between threads safely
  error[E0277]: `T` cannot be shared between threads safely
  ```

  Fixed by `T: 'static + Send + Sync`. Worth noting fairly: the compiler
  suggested exactly that, and the suggestion was correct.

- **A `#[page]` function cannot share a name with a module in scope.** Naming
  the invoices route `invoices()` alongside `mod invoices` produced **five**
  errors, none of which says "rename the function":

  ```
  error[E0428]: the name `invoices` is defined multiple times
  error[E0573]: expected type, found module `invoices`
  error[E0425]: cannot find function `handler` in module `invoices`
  error: the `Self` constructor can only be used with tuple or unit structs
  error[E0277]: the trait bound `invoices: Page` is not satisfied
  ```

  The macro generates an item named after the function, so the collision is
  real — but the diagnostics describe the *expansion*, not the cause. This is
  the first thing found that a newcomer would lose time to. Renaming to
  `invoices_list` fixed it instantly.

- **Worked first attempt, worth recording as well as the friction:** `if/else`
  inline in `view!` for the computed column, `format!` inside an `href`
  attribute, and Czech text through the whole path without an encoding step.

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
