//! Screen two, declared as a descriptor.
//!
//! Refactored from the hand-written version (git history) after the framework
//! gained `CellKind::Link`. The zero-diff test decides whether this is the
//! same screen.
//!
//! The global that made the locale work is gone. The accessors are boxed
//! closures now, so they capture the locale directly — see `Accessor` in
//! `descriptor.rs` for what that cost.

use crate::descriptor::{col, CellKind, TableDescriptor};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    De,
    En,
}

pub struct Invoice {
    pub number: &'static str,
    pub client: &'static str,
    pub amount_cents: i64,
    pub due: &'static str,
    pub status: &'static str,
    pub days_late: i32,
}

pub fn rows() -> Vec<Invoice> {
    vec![
        Invoice { number: "2026-0041", client: "Stadtbibliothek München", amount_cents: 4_850_00,  due: "2026-08-15", status: "paid",  days_late: 0 },
        Invoice { number: "2026-0042", client: "Landratsamt Starnberg",   amount_cents: 12_400_50, due: "2026-08-28", status: "sent",  days_late: 5 },
        Invoice { number: "2026-0043", client: "Caritas Berlin",          amount_cents: 990_00,    due: "2026-09-10", status: "draft", days_late: 0 },
    ]
}

/// de-DE: `4.850,00 €`  ·  en-US: `$4,850.00`
pub fn money(locale: Locale, cents: i64) -> String {
    let (thousands, decimal) = match locale {
        Locale::De => ('.', ','),
        Locale::En => (',', '.'),
    };
    let whole = cents / 100;
    let rest = (cents % 100).abs();

    let mut digits = whole.abs().to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        grouped = format!("{thousands}{tail}{grouped}");
    }
    grouped = format!("{digits}{grouped}");
    if whole < 0 {
        grouped.insert(0, '-');
    }

    match locale {
        Locale::De => format!("{grouped}{decimal}{rest:02} €"),
        Locale::En => format!("${grouped}{decimal}{rest:02}"),
    }
}

fn late_label(locale: Locale, days: i32) -> String {
    if days <= 0 {
        return "—".to_string();
    }
    match locale {
        Locale::De => format!("{days} Tage"),
        Locale::En => format!("{days} days"),
    }
}

pub fn descriptor(l: Locale) -> TableDescriptor<Invoice> {
    let de = matches!(l, Locale::De);
    TableDescriptor {
        title: if de { "Rechnungen" } else { "Invoices" },
        columns: vec![
            // The locale is captured here — the thing an `fn` pointer could not do.
            col(if de { "Nummer" } else { "Number" },
                CellKind::Link { href: Box::new(|i: &Invoice| format!("/invoices/{}", i.number)) },
                |i: &Invoice| i.number.to_string()),
            col(if de { "Kunde" } else { "Client" },  CellKind::Text,   |i: &Invoice| i.client.to_string()),
            col(if de { "Betrag" } else { "Amount" }, CellKind::Number, move |i: &Invoice| money(l, i.amount_cents)),
            col(if de { "Fällig" } else { "Due" },    CellKind::Date,   |i: &Invoice| i.due.to_string()),
            col("Status",                             CellKind::Badge,  |i: &Invoice| i.status.to_string()),
            col(if de { "Verzug" } else { "Late" },   CellKind::Number, move |i: &Invoice| late_label(l, i.days_late)),
        ],
    }
}
