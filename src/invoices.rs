//! Screen two, now declared as a descriptor.
//!
//! ATTEMPT 1 — refactor onto the existing skeleton, changing nothing in
//! `descriptor.rs` or `table.rs`. Whatever this cannot express is the result.

use crate::descriptor::{CellKind, Column, TableDescriptor};

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

pub const INVOICES: TableDescriptor<Invoice> = TableDescriptor {
    title: "Invoices",
    columns: &[
        // The link column. There is no way to say "this cell is an anchor",
        // so the closest the descriptor can get is the markup as a string.
        Column { header: "Number", kind: CellKind::Text, get: |i| format!("<a href=\"/invoices/{}\">{}</a>", i.number, i.number) },
        Column { header: "Client", kind: CellKind::Text,   get: |i| i.client.to_string() },
        // Locale formatting survives ONLY because cs-CZ is hardcoded inside
        // `koruna`. A non-capturing closure coerces to `fn`; the moment the
        // locale is a parameter, this stops compiling.
        Column { header: "Amount", kind: CellKind::Number, get: |i| koruna(i.amount_haleru) },
        Column { header: "Due",    kind: CellKind::Date,   get: |i| i.due.to_string() },
        Column { header: "Status", kind: CellKind::Badge,  get: |i| i.status.to_string() },
        // Computed from another field — this one the descriptor handles fine.
        Column { header: "Late",   kind: CellKind::Number, get: |i| if i.days_late > 0 { format!("{} days", i.days_late) } else { "—".to_string() } },
    ],
};
