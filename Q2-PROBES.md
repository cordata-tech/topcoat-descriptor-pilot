# Q2 — what the macro says when it rejects you

The Topcoat intro post promised to report "error messages when the macro
rejects an expression". Nothing has been fed to `view!` that it refused yet,
so this is unanswered.

**The question is not whether it errors.** It's whether the error points at
*your* line or at macro-expanded code you never wrote. A macro that fails
clearly is a different framework to live with than one that fails at the
expansion — and that difference is exactly what a reader piloting it wants to
know.

You don't need to write working code here. Every probe below is meant to fail.

---

## How to run one

1. Open `src/probe.rs` (created for this, currently a stub).
2. Uncomment **one** probe at a time — they interfere if stacked.
3. Run:

   ```bash
   cargo build 2>&1 | head -40
   ```

4. Record in `NOTES.md` under **Q2**, using the table at the bottom of this
   file. Copy the *first* error verbatim, then answer three things:

   - **Where does it point?** A line in `src/probe.rs`, or somewhere inside
     `view!`/`topcoat-macros`?
   - **Does it name the real cause**, or a symptom of the expansion?
   - **Would it have told you what to do** if you didn't already know?

5. Re-comment it and move to the next.

**Record the boring answers too.** "Pointed at the right line, said exactly
what was wrong" is a finding, and a good one. The post is worthless if it only
lists complaints.

---

## The probes

### P1 — a value that isn't `Send`

`#[component]` requires the returned future to be `Send`. `Rc` isn't.

```rust
use std::rc::Rc;

#[component]
pub async fn probe() -> Result {
    let name = Rc::new(String::from("Ada"));
    view! { <p>(name)</p> }
}
```

*Why it matters:* this is the same class as the `T: Send + Sync` bound already
recorded in Q1, but hit from the value side rather than the type-parameter
side. Does it read the same?

### P2 — a borrow that outlives the view

```rust
#[component]
pub async fn probe() -> Result {
    let view = {
        let local = String::from("temporary");
        view! { <p>(&local)</p> }
    };
    view
}
```

*Why it matters:* lifetime errors are where Rust is at its least friendly, and
a macro in the middle can make the span point anywhere.

### P3 — a `$()` expression that can't cross to JavaScript

`$(...)` is type-checked Rust that Topcoat also compiles to JS. Give it
something that only exists on the server.

```rust
async fn server_only() -> String { String::from("from the database") }

#[component]
pub async fn probe() -> Result {
    view! {
        <button @click=$(server_only())>"click"</button>
    }
}
```

*Why it matters:* **this is the most interesting probe.** The `$()` trick is
Topcoat's headline feature, and its whole risk is a boundary the type system
has to police. If this error is good, the feature is safe to use. If it points
into generated JS glue, that's the seam the post is about.

### P4 — malformed markup

```rust
#[component]
pub async fn probe() -> Result {
    view! { <div><p>"unclosed div"</div> }
}
```

*Why it matters:* the cheapest, most common mistake. Anyone will hit it on
day one.

### P5 — a type that can't render

```rust
struct NotDisplayable;

#[component]
pub async fn probe() -> Result {
    view! { <p>(NotDisplayable)</p> }
}
```

*Why it matters:* does it say "this needs `Display`", or does it surface as a
trait bound on some internal `IntoView`-style type the reader has never seen?

---

## Record it like this

| Probe | Points at | Names the cause? | Verdict |
|---|---|---|---|
| P1 non-`Send` | | | |
| P2 borrow escapes | | | |
| P3 `$()` server-only | | | |
| P4 malformed markup | | | |
| P5 not displayable | | | |

Then one sentence for the post: **is this a macro you'd be comfortable putting
a junior on?** That's the question a reader is actually asking.

---

## If you get stuck

- `cargo build 2>&1 | head -40` — the first error is the one that matters;
  the rest are usually cascade.
- Errors mentioning `topcoat_macros` or a `#[doc(hidden)]` type are pointing
  at expansion rather than your code. **That itself is the finding** — note it
  and move on.
- Nothing here can break the pilot. `src/probe.rs` isn't wired into any route,
  and `git checkout src/probe.rs` resets it.
