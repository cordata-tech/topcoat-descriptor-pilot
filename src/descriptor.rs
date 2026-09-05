//! The descriptor types.
//!
//! This is the whole experiment. `one-skeleton-many-screens` argued that an
//! admin screen should be *declared as typed data* and rendered by a framework
//! that knows nothing about the domain — and that the test of the boundary is a
//! zero-diff refactor: adding a second screen must cost a descriptor and
//! nothing else.
//!
//! In React a column descriptor can hold a render function, because JSX is a
//! runtime value. Here `view!` is a macro, so the open question is whether a
//! descriptor can stay *data* at all, or whether the macro drags it back into
//! being code. The shape below is the first attempt at keeping it data:
//! a plain `fn` pointer for the accessor (no `Box`, no lifetimes) and a closed
//! enum for how a cell is drawn.
//!
//! That enum is strictly more constrained than React's `render: (row) => Node`.
//! Whether that constraint is a cost or the point is what the pilot is for —
//! do not resolve it here, record it in NOTES.md once there is evidence.

/// How a cell is drawn. Closed on purpose: a domain cannot add a variant, which
/// is the one-way dependency the original post argued for.
///
/// `Link` was added 2026-09-05, after the zero-diff test failed on exactly one
/// column. `fn(&T) -> String` can express any cell that is a **value** and no
/// cell that is **structure** — an interpolated `String` is escaped by `view!`,
/// correctly, so markup returned from an accessor renders as visible text.
///
/// Note what the fix did and did not require. The framework learned a new
/// *shape*; it learned nothing about invoices. That is the one-way dependency
/// working rather than failing — and it is the cost side of the trade, because
/// a domain that needs a new shape cannot add one itself.
pub enum CellKind<T: 'static> {
    Text,
    Number,
    Date,
    Badge,
    /// An anchor. The href is computed from the row exactly as the label is.
    Link { href: Accessor<T> },
}

/// **This type is the whole finding.**
///
/// It was `fn(&T) -> String` — a plain function pointer, `Copy`, `const`-able,
/// no allocation. That works for exactly as long as every cell is a pure
/// function of the row.
///
/// It is now a boxed closure, because the locale comes from the request and an
/// `fn` pointer cannot capture. What that cost, precisely:
///
/// - `Column` and `CellKind` are no longer `Copy`, so they are no longer
///   `const`, so a descriptor is no longer a `static` — it is built per
///   request, with one allocation per column.
/// - `TableDescriptor.columns` went from `&'static [Column<T>]` to `Vec`.
/// - `Send + Sync` had to be spelled out, because `#[component]` requires the
///   future to be `Send` and a bare `dyn Fn` is neither.
///
/// What it bought: the global is gone, two locales can coexist, and a cell's
/// value no longer depends on when it is read.
pub type Accessor<T> = Box<dyn Fn(&T) -> String + Send + Sync>;

/// One column: a header, how to draw it, and how to get it out of a row.
///
/// `fn(&T) -> String` rather than a closure is deliberate — it keeps the
/// descriptor a `const`-able plain value. If a real screen needs captured
/// state this has to become a `Box<dyn Fn>`, and that moment is worth writing
/// down: it is the point where the descriptor stops being data.
pub struct Column<T: 'static> {
    pub header: &'static str,
    pub kind: CellKind<T>,
    pub get: Accessor<T>,
}

/// A screen, declared.
pub struct TableDescriptor<T: 'static> {
    pub title: &'static str,
    pub columns: Vec<Column<T>>,
}

/// Sugar so a descriptor still *reads* as data at the call site, which is the
/// thing worth preserving. `text("Client", |i| ...)` is not much further from
/// the original than `Column { header, kind, get }` was.
pub fn col<T: 'static>(
    header: &'static str,
    kind: CellKind<T>,
    get: impl Fn(&T) -> String + Send + Sync + 'static,
) -> Column<T> {
    Column { header, kind, get: Box::new(get) }
}
