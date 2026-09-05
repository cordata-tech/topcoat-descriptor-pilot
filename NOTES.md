# Findings

The four questions `topcoat-full-stack-rust` (2026-08-02) promised to answer,
plus the one this pilot exists to test.

| | | Status |
|---|---|---|
| **[Q0](#q0--does-the-descriptor-survive-the-move-to-rust)** | Does the descriptor survive the move to Rust? | **answered** — declarative survived, `const` did not |
| **[Q1](#q1--developer-experience-end-to-end)** | Developer experience end to end | partial — and **not** László's experience, see below |
| **[Q2](#q2--error-messages-when-the-macro-rejects-an-expression)** | Error messages when the macro rejects an expression | **answered** — spans good, vocabulary opaque |
| **[Q3](#q3--does-the-fast-rebuild-loop-hold-up)** | Does the fast-rebuild loop hold up? | **answered** — yes, ~1.6ms per `view!` block |
| **[Q4](#q4--where-the-trade-offs-land)** | Where the trade-offs land | **answered** — descriptor absorbed a real DB unchanged |

Also here: [the zero-diff harness](#the-zero-diff-harness) and
[notes for the CI/CD post](#for-the-cicd-post-second-post-not-the-same-one),
which is a separate piece and must not be merged into this one.

**Write findings here while building,
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
Claude under direction. The Q1 findings below are therefore _its_ experience
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

### Result of the zero-diff test — attempt 1, nothing in the framework changed

```
ZERO DIFF  users     (/)
CHANGED    invoices  (/invoices)   — one column
```

**`users` is byte-identical.** Adding a second screen did not move the first.
The one-way dependency held: `table.rs` still names no domain type.

**`invoices` reproduced five of its six columns exactly**, including the two
expected to break it:

- the **locale-formatted currency** (`4 850,00 Kč`) survived — but only
  because cs-CZ is hardcoded inside `koruna`, so the closure captures nothing
  and coerces to `fn`. Make the locale a parameter and it stops compiling.
  The constraint is real, it just was not exercised yet.
- the **computed column** (`days_late` → `5 days` / `—`) survived outright.

**Exactly one thing failed: the link cell.**

```
- <td class="cell cell-text"><a href="/invoices/2026-0041">2026-0041</a></td>
+ <td class="cell cell-text">&lt;a href="/invoices/2026-0041"&gt;2026-0041&lt;/a&gt;</td>
```

`view!` escapes an interpolated `String`, so returning markup from an accessor
renders as visible text. **That is correct framework behaviour** — it is the
XSS defence — not a bug to route around.

### What this actually says, which is not "the abstraction failed"

The boundary held for every cell that is a **value** and broke at the first
cell that is **structure**. `fn(&T) -> String` can express anything you can
compute; it cannot express anything you can nest.

Fixing it means adding a variant to `CellKind` and a branch in `table.rs` —
a change to the _framework_ side, not the domain side. **That is the one-way
dependency working as designed**, not failing: a domain cannot smuggle in new
presentation, so a new cell shape is a deliberate, single, framework-wide
decision.

Worth stating plainly in the post, because it inverts the React result. There
a `ColumnDef` held a render function, so any domain could return arbitrary
JSX — more flexible, and **no boundary at all**. The Rust version has the
stricter boundary; the price is that new presentation costs a framework
change, and the benefit is that the framework knows every shape a cell can
take. Which of those you want is the actual decision, and it is not obvious.

### Attempt 2 — `CellKind::Link` added, and the locale test run

```
ZERO DIFF  users        (/)
ZERO DIFF  invoices-en  (/invoices)
ZERO DIFF  invoices-de  (/de/invoices)
```

**Do not read that as a pass.** Two separate things happened and only one of
them is good.

**The link: clean.** `CellKind` gained a `Link { href: fn(&T) -> String }`
variant and `table.rs` gained one branch. Blast radius was two framework files
and zero domain knowledge — `table.rs` still names no domain type. The
framework learned a **shape**, which is the one-way dependency behaving
exactly as designed. Cost noted honestly: making the enum generic over `T`
also meant hand-writing `Clone`/`Copy`, because the derive would have demanded
`T: Copy` for a variant that only holds a function pointer.

**The locale: the thesis broke, as predicted on day one.**

`fn(&T) -> String` cannot capture, and the locale comes from the *route*, not
the row — so an accessor cannot see it. The only way to keep the descriptor a
`const` was **a global**:

```rust
static LOCALE: AtomicU8 = AtomicU8::new(0);   // set per request, read per cell
```

That is not a fix, it is the smallest thing that compiles, and it is worse
than the problem it solves: two descriptors differing only by locale cannot
exist at once, and what a cell renders now depends on *when* it is read. It is
committed deliberately, with this note, so the cost is visible in code rather
than described in prose.

**So the green result above is a test pinning the wrong thing** — the same
shape as Trap 2 in the trace_id post. Zero diff was achieved by making the
program worse.

### The actual answer to Q0

**A screen descriptor survives the move to Rust as `const` data for exactly as
long as every cell is a pure function of the row.** Add ambient context —
locale, permissions, tenant, an injected formatter — and the accessor must
become `Box<dyn Fn(&T) -> String + Send + Sync>`, at which point the
descriptor is no longer `const`, no longer trivially `Copy`, and no longer the
cheap thing the original post was recommending.

React never hit this because a `ColumnDef` held a closure from the start. It
captured whatever it liked and nobody noticed, because in JavaScript a
function and a value are the same kind of thing. The Rust version makes the
distinction explicit: **a descriptor of `fn` pointers is a genuinely
different, stricter artefact than a descriptor of closures**, and the strictness
buys the one-way dependency the original post wanted. Whether the boundary is
worth what it costs is the honest question, and it is not obvious.

### Attempt 3 — `Box<dyn Fn>`, global deleted. The result inverts the premise.

```
ZERO DIFF  users        (/)
ZERO DIFF  invoices-en  (/invoices)
ZERO DIFF  invoices-de  (/de/invoices)
```

Same three screens, byte-identical, **and this time without the global.** The
accessors capture the locale directly.

**The measurement, which is the surprise:**

| | `fn` pointers + global | `Box<dyn Fn>` | |
|---|---:|---:|---|
| `descriptor.rs` | 66 | 88 | +22 |
| `table.rs` | 58 | 58 | — |
| `users.rs` | 72 | 54 | −18 |
| `invoices.rs` | 119 | 88 | −31 |
| **total** | **315** | **288** | **−27** |

**The "expensive" version is smaller.** The framework grew 22 lines — a
documented `Accessor<T>` alias and a `col(...)` constructor — and the domains
shrank 49, because the `const` version needed a macro to generate two static
column arrays per locale, plus a global to smuggle the locale in. All of that
is gone.

### What was actually traded

**Lost:** `Copy` on `Column` and `CellKind`; `const`/`static` descriptors;
`&'static [Column<T>]`; zero allocation. A descriptor is now built per request
with one `Box` per column.

**Gained:** the locale is captured rather than smuggled; two locales coexist;
a cell's value no longer depends on when it is read; and the domain files got
substantially shorter.

**Kept, which was the actual question:** `table.rs` still names no domain type,
and a descriptor still *reads* as data at the call site —

```rust
col(if de { "Betrag" } else { "Amount" }, CellKind::Number,
    move |i: &Invoice| money(l, i.amount_cents)),
```

That is not meaningfully further from `Column { header, kind, get }` than the
original was. **Declarative survived; `const` did not.**

### The conclusion, and it is not the one I expected

The premise going in — inherited from the React post — was that the descriptor
should be *data*, and the Rust version's `fn` pointers looked like a stricter,
purer form of the same idea. They are stricter. They are not purer, and past
the first cell that needs ambient context they are not cheaper either: keeping
`const` cost a macro, two duplicated arrays and a global, which is more
machinery than the boxed closure it was avoiding.

So the honest finding is that **`const`-ness was never the property that
mattered.** What made the original abstraction work was the one-way dependency
— domains hand the framework descriptions, the framework knows nothing about
domains — and that held throughout, unchanged, across all three attempts. The
thing that broke was an implementation detail everyone (me included) mistook
for the idea.

Which is a better post than either "it worked" or "it didn't". Nothing is proven until a
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
correctly produced `CHANGED` (exit 1). Reverting it with `mv` then _still_
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
  real — but the diagnostics describe the _expansion_, not the cause. This is
  the first thing found that a newcomer would lose time to. Renaming to
  `invoices_list` fixed it instantly.

- **Worked first attempt, worth recording as well as the friction:** `if/else`
  inline in `view!` for the computed column, `format!` inside an `href`
  attribute, and Czech text through the whole path without an encoding step.

- **Untested:** `topcoat ui`, `topcoat fmt`, the CLI generally. Not installed.

## Q2 — Error messages when the macro rejects an expression

**Probed 2026-09-05 by László** (five deliberate failures, one at a time — see
`Q2-PROBES.md`). Raw compiler output at the bottom of this section.

| Probe | Points at | Names the cause? | Verdict |
|---|---|---|---|
| P1 `Rc` | your `view!` line | **no — wrong cause** | probe was mis-designed, see below |
| P2 borrow escapes | — | — | **compiled. No error at all** |
| P3 `$()` server-only | the exact `$(…)` span | partly — in framework vocabulary | the seam |
| P4 malformed markup | the exact closing tag | **yes, plainly** | the best of the five |
| P5 not displayable | your line **and** the struct definition | yes | good |

### The headline: spans are consistently good

**Every error points at the source line, not into macro-expanded code.** That
was the question this probe existed to answer, and the answer is favourable.
P4 and P5 go further — P4 names the exact mismatched tag, P5 adds a second
span pointing at the offending struct's definition.

### P4 is a purpose-built diagnostic and it shows

```
error: closing tag `div` does not match opening tag `p`
 --> src/probe.rs:47:37
```

No trait bounds, no expansion note, plain English, exact column. Someone wrote
that error on purpose. It is the most common mistake anyone will make and it
is the best-handled.

### Where it thins out: framework vocabulary in trait bounds

P1, P3 and P5 all surface as unsatisfied trait bounds on **types a reader has
never heard of** — `NodeViewParts`, `Surrogated` — followed by a `help:` list
of implementors ending "and 57 others". Actionable if you read Rust fluently;
it tells you *that* the value is unacceptable, never the **rule** for what is
acceptable.

P3 is the one that matters, since `$()` is the headline feature:

```
error[E0277]: the trait bound `fn() -> impl Future<Output = String>
              {server_only}: Surrogated` is not satisfied
 --> src/probe.rs:42:26
    | <button @click=$(server_only())>"click"</button>
    |                 ^^^^^^^^^^^ the trait `Surrogated` is not implemented
```

The span is exactly right — it underlines the offending call inside `$()`, not
the whole view. But nothing says *why an async fn cannot cross to JavaScript*,
which is the actual rule, and `Surrogated` is not a word in the README. The
boundary is policed correctly and explained poorly.

### P2 did not fail, which is a finding

A borrow taken in an inner block and interpolated into `view!` **compiled
cleanly**. The expected lifetime error never happened, so `view!` is not
holding the borrow past the block. Good news, and worth saying: this is the
class of error people fear most from a Rust macro, and it did not appear.

### P1 was a bad probe — mine, not Topcoat's

The intent was to test a non-`Send` value. What it actually tested was
renderability: `Rc<String>` fails on `NodeViewParts` before `Send` is ever
considered, making P1 a duplicate of P5. A real `Send` probe has to hold the
`Rc` **across an await** inside the component. Recorded rather than quietly
dropped, because the promised post reports on error messages and a probe that
measured the wrong thing would have produced a wrong claim.

### Would you put a junior on this macro?

Yes, with one caveat: spans are good enough that they will always land in the
right place, but they will need telling *once* what `NodeViewParts` and
`Surrogated` mean, because the errors never say. That is a documentation gap,
not a design one.

---

### Raw output

P1

Compiling cordata-topcoat-pilot v0.1.0 (/Users/lhadhazy/dev/cordata-topcoat-pilot)
error[E0277]: the trait bound `Rc<String>: NodeViewParts` is not satisfied
--> src/probe.rs:21:5
|
21 | view! { <p>(name)</p> }
| ^^^^^^^^^^^^^^^^^^^^^^^ the trait `NodeViewParts` is not implemented for `Rc<String>`
|
= help: the following other types implement trait `NodeViewParts`:
&&'b T
&String
&bool
&char
&f32
&f64
&i128
&i16
and 57 others
note: required by a bound in `topcoat::view::internal::Builder::<'_, '_, '_>::node`
--> /Users/lhadhazy/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/topcoat-view-0.6.2/src/internal.rs:184:40
|
184 | pub fn node(&mut self, value: impl NodeViewParts) {
| ^^^^^^^^^^^^^ required by this bound in `Builder::<'_, '_, '_>::node`
= note: this error originates in the macro `view` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `cordata-topcoat-pilot` (bin "cordata-topcoat-pilot") due to 1 previous error

P2

Compiling cordata-topcoat-pilot v0.1.0 (/Users/lhadhazy/dev/cordata-topcoat-pilot)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.12s

￼

P3

Compiling cordata-topcoat-pilot v0.1.0 (/Users/lhadhazy/dev/cordata-topcoat-pilot)
error[E0277]: the trait bound `fn() -> impl Future<Output = String> {server_only}: Surrogated` is not satisfied
--> src/probe.rs:42:26
|
41 | / view! {
42 | | <button @click=$(server_only())>"click"</button>
| | ^^^^^^^^^^^ the trait `Surrogated` is not implemented for fn item `fn() -> impl Future<Output = String> {server_onl
y}`
43 | | }
| |**\_**- required by a bound introduced by this call
|
= help: the following other types implement trait `Surrogated`:
&'**lifetime Option<T>
&'**lifetime Result<T, E>
&'**lifetime String
&'**lifetime bool
&'**lifetime f64
&'**lifetime mut Option<T>
&'**lifetime mut Result<T, E>
&'**lifetime mut String
and 25 others

error[E0277]: the trait bound `fn() -> impl Future<Output = String> {server_only}: Surrogated` is not satisfied
--> src/probe.rs:41:5
|
41 | / view! {
42 | | <button @click=$(server_only())>"click"</button>
43 | | }
| |**\_**^ the trait `Surrogated` is not implemented for fn item `fn() -> impl Future<Output = String> {server_only}`
|
= help: the following other types implement trait `Surrogated`:
&'**lifetime Option<T>
&'**lifetime Result<T, E>
&'**lifetime String
&'**lifetime bool
&'**lifetime f64
&'**lifetime mut Option<T>
&'**lifetime mut Result<T, E>
&'**lifetime mut String
and 25 others
= note: this error originates in the macro `::topcoat::runtime::expr` (in Nightly builds, run with -Z macro-backtrace for more info)

￼

P4

Compiling cordata-topcoat-pilot v0.1.0 (/Users/lhadhazy/dev/cordata-topcoat-pilot)
error: closing tag `div` does not match opening tag `p`
--> src/probe.rs:47:37
|
47 | view! { <div><p>"unclosed div"</div> }
| ^^^

error: could not compile `cordata-topcoat-pilot` (bin "cordata-topcoat-pilot") due to 1 previous error

￼

P5

Compiling cordata-topcoat-pilot v0.1.0 (/Users/lhadhazy/dev/cordata-topcoat-pilot)
error[E0277]: the trait bound `NotDisplayable: NodeViewParts` is not satisfied
--> src/probe.rs:55:5
|
55 | view! { <p>(NotDisplayable)</p> }
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
|
help: the trait `NodeViewParts` is not implemented for `NotDisplayable`
--> src/probe.rs:51:1
|
51 | struct NotDisplayable;
| ^^^^^^^^^^^^^^^^^^^^^
= help: the following other types implement trait `NodeViewParts`:
&&'b T
&String
&bool
&char
&f32
&f64
&i128
&i16
and 57 others
note: required by a bound in `topcoat::view::internal::Builder::<'_, '_, '_>::node`
--> /Users/lhadhazy/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/topcoat-view-0.6.2/src/internal.rs:184:40
|
184 | pub fn node(&mut self, value: impl NodeViewParts) {
| ^^^^^^^^^^^^^ required by this bound in `Builder::<'_, '_, '_>::node`
= note: this error originates in the macro `view` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `cordata-topcoat-pilot` (bin "cordata-topcoat-pilot") due to 1 previous error

￼

## Q3 — Does the fast-rebuild loop hold up?

**Yes, and `view!` expansion is not the bottleneck.** Measured 2026-09-05 with
`scripts/measure-rebuild.sh` — touch one file, rebuild, seven runs, take the
median. macOS arm64, `rustc` 1.97.1, debug profile.

**Correction to the first number recorded here.** The earlier 1.38s was a
single cold-ish run. The warm median is **0.47s**; the first run after a pause
is consistently ~1.3s and then it settles. One measurement was not a
measurement.

### Does it scale with template size?

The real question is whether the macro dominates as an app grows. Generated
N components each containing a `view!` block
(`scripts/gen-views-for-measurement.py`) and re-measured:

| `view!` blocks in crate | median rebuild |
|---:|---:|
| 6 | 0.47s |
| 31 | 0.52s |
| 81 | 0.57s |
| 156 | 0.71s |

**Roughly linear, and shallow — about 1.6ms per `view!` block.** Extrapolating
naively, 500 components would land near 1.3s, which is still inside the range
where the loop feels immediate. Nothing here suggests expansion cost becomes
the thing you notice.

Caveats worth keeping attached to those numbers: the generated components are
simple (one loop, three elements, no `$(…)` reactivity, no `#[shard]`), it is
one crate on one machine, and cargo recompiles the whole crate on any change
so this is total crate time rather than per-file. A real app with heavier
templates and client expressions could look different — but the slope, not the
absolute, is what this measures, and the slope is gentle.

**Untested:** the dev-server reload loop. `topcoat::dev::script()` is wired but
the browser-refresh half has never been exercised, so "fast rebuild" here means
`cargo build`, not the round trip a developer actually feels.

## Q4 — Where the trade-offs land

**Toasty wired in 2026-09-05.** `Invoice` is now a `#[derive(toasty::Model)]`
backed by in-memory SQLite, seeded with the same three rows and read back with
`Invoice::all().exec(&mut db)`. Same three screens, still byte-identical.

### The descriptor absorbed a real database without changing

**Not one accessor changed.** What moved was underneath them:

| | fixture | Toasty model |
|---|---|---|
| string fields | `&'static str` | `String` |
| `days_late` | `i32` | `i64` — Toasty's integer width |
| `rows()` | sync | **`async`** |
| descriptor | — | **unchanged** |

Every accessor already returned `String`, so the widening was invisible to
them. And `rows()` becoming `async` did not reach the descriptor either,
because the rows are loaded *before* the table is rendered — the page awaits,
then hands a `Vec<T>` to a component whose accessors stay synchronous.

**That is the trade-off landing in a good place**, and it is the third time
the same thing has held: the one-way dependency survived a new cell shape, a
new source of ambient context, and now a swap of the entire data layer.

### The limit, named rather than glossed

This worked because **every column is a scalar already on the row.** It says
nothing about `Deferred` relations, which is Toasty's lazy-loading shape — a
column showing "invoices for this client" would need the accessor to hit the
database, and `Fn(&T) -> String` is **synchronous**. There is no version of
that signature which awaits.

So the honest boundary of the Q4 answer: a descriptor of sync accessors is
fine over a database as long as the query is done before rendering starts. The
first column that needs to load something is the one that breaks it, and
**that is untested** — no relation was added.

### Friction, for the record

- `collect(&mut db)` is not a terminal. The terminals are `exec()`,
  `first().exec()`, and `get()`. The error — *"`InvoiceQuery` is not an
  iterator"* — was clear enough to fix in one attempt.
- `#[derive(toasty::Model)]` needs `uuid` as a direct dependency for the
  `#[key] #[auto]` field; it is not re-exported.
- Toasty's integer columns are `i64`. An `i32` field is a type error at the
  call site, not at the model.

### Gotcha that nearly became a false finding

The first `cargo build` after enabling `toasty/sqlite` failed to resolve:
`libsqlite3-sys ^0.38.0` "candidate versions found which didn't match: 0.36.0,
0.35.0 …". That reads exactly like *the sqlite driver of the published Toasty
is unbuildable*, which would have been a significant claim about the pairing
the intro post named.

**It was a stale local cargo index.** `libsqlite3-sys` 0.38.2 has been on
crates.io the whole time; refreshing the index fixed it with no version change.
Checking crates.io before writing it down is the only reason it is not in this
file as a finding about Toasty.

## For the CI/CD post (second post, not the same one)

Kept separate deliberately — do not merge these into the descriptor post.

- Upstream roadmap has **"Docs for how to deploy Topcoat" unchecked**, so
  deployment is unsolved upstream rather than merely undocumented by us.
- No Node in the pipeline is the claim worth testing against a real workflow.
- What is the deployable artefact, and what does the asset bundler need at
  build time versus run time?
