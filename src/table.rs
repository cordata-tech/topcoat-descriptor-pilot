//! The framework half: renders any descriptor, knows nothing about any domain.
//!
//! The one-way dependency lives here. `table.rs` must never mention `User`,
//! `Invoice`, or any other domain type. If it ever has to, the boundary failed
//! and that failure is the most interesting thing the pilot could produce —
//! write it down rather than working around it.

use crate::descriptor::{CellKind, Column};
use topcoat::{
    Result,
    view::{component, view},
};

fn cell_class(kind: CellKind) -> &'static str {
    match kind {
        CellKind::Text => "cell cell-text",
        CellKind::Number => "cell cell-number",
        CellKind::Date => "cell cell-date",
        CellKind::Badge => "cell cell-badge",
    }
}

#[component]
pub async fn table_page<T: 'static + Send + Sync>(
    title: &str,
    columns: &'static [Column<T>],
    rows: Vec<T>,
) -> Result {
    view! {
        <h1>(title)</h1>
        <table>
            <thead>
                <tr>
                    for col in columns {
                        <th>(col.header)</th>
                    }
                </tr>
            </thead>
            <tbody>
                for row in &rows {
                    <tr>
                        for col in columns {
                            <td class=(cell_class(col.kind))>((col.get)(row))</td>
                        }
                    </tr>
                }
            </tbody>
        </table>
    }
}
