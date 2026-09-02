//! Screen two, written BY HAND on purpose.
//!
//! This is the "screen you already trust" half of the zero-diff test. It is
//! deliberately NOT built on the descriptor: the point is to write the markup
//! we actually want, capture it, and then see whether the skeleton can
//! reproduce it byte for byte.
//!
//! It is also deliberately harder than `users`. That screen is all Text /
//! Number / Date / Badge, which `CellKind` already covers, so refactoring it
//! would prove nothing. This one carries three things the descriptor cannot
//! currently express:
//!
//!   1. a currency amount formatted for a locale (cs-CZ grouping, `Kč` suffix)
//!   2. a column computed from two fields (`days_late`, from due date + status)
//!   3. a cell that is a link rather than text
//!
//! If the refactor forces a change to `descriptor.rs` or `table.rs`, that is
//! the result of the experiment. Record what forced it in NOTES.md before
//! changing anything.

use topcoat::{Result, view::{component, view}};

pub struct Invoice {
    pub number: &'static str,
    pub client: &'static str,
    pub amount_haleru: i64,
    pub due: &'static str,
    pub status: &'static str,
    pub days_late: i32,
}

pub fn rows() -> Vec<Invoice> {
    vec![
        Invoice { number: "2026-0041", client: "Městská knihovna", amount_haleru: 4_850_00, due: "2026-08-15", status: "paid",    days_late: 0 },
        Invoice { number: "2026-0042", client: "Krajský úřad",     amount_haleru: 12_400_50, due: "2026-08-28", status: "sent",    days_late: 5 },
        Invoice { number: "2026-0043", client: "Charita Brno",     amount_haleru: 990_00,    due: "2026-09-10", status: "draft",   days_late: 0 },
    ]
}

/// cs-CZ money formatting: space as the thousands separator, comma as the
/// decimal mark, currency after the number. Written out rather than pulled
/// from a crate so the refactor has something concrete to fail to express —
/// a `fn(&T) -> String` accessor cannot capture a locale.
fn koruna(haleru: i64) -> String {
    let whole = haleru / 100;
    let cents = (haleru % 100).abs();
    let mut digits = whole.abs().to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        grouped = format!(" {tail}{grouped}");
    }
    grouped = format!("{digits}{grouped}");
    if whole < 0 {
        grouped.insert(0, '-');
    }
    format!("{grouped},{cents:02} Kč")
}

#[component]
pub async fn invoices_page(rows: Vec<Invoice>) -> Result {
    view! {
        <h1>"Invoices"</h1>
        <table>
            <thead>
                <tr>
                    <th>"Number"</th>
                    <th>"Client"</th>
                    <th>"Amount"</th>
                    <th>"Due"</th>
                    <th>"Status"</th>
                    <th>"Late"</th>
                </tr>
            </thead>
            <tbody>
                for inv in &rows {
                    <tr>
                        <td class="cell cell-text">
                            <a href=(format!("/invoices/{}", inv.number))>(inv.number)</a>
                        </td>
                        <td class="cell cell-text">(inv.client)</td>
                        <td class="cell cell-number">(koruna(inv.amount_haleru))</td>
                        <td class="cell cell-date">(inv.due)</td>
                        <td class="cell cell-badge">(inv.status)</td>
                        <td class="cell cell-number">
                            if inv.days_late > 0 {
                                (format!("{} days", inv.days_late))
                            } else {
                                "—"
                            }
                        </td>
                    </tr>
                }
            </tbody>
        </table>
    }
}
