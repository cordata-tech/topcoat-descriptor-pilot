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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Text,
    Number,
    Date,
    Badge,
}

/// One column: a header, how to draw it, and how to get it out of a row.
///
/// `fn(&T) -> String` rather than a closure is deliberate — it keeps the
/// descriptor a `const`-able plain value. If a real screen needs captured
/// state this has to become a `Box<dyn Fn>`, and that moment is worth writing
/// down: it is the point where the descriptor stops being data.
pub struct Column<T: 'static> {
    pub header: &'static str,
    pub kind: CellKind,
    pub get: fn(&T) -> String,
}

/// A screen, declared.
pub struct TableDescriptor<T: 'static> {
    pub title: &'static str,
    pub columns: &'static [Column<T>],
}
