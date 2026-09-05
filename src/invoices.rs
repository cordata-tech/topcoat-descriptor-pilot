//! Screen two, declared as a descriptor.
//!
//! Refactored from the hand-written version (git history) after the framework
//! gained `CellKind::Link`. The zero-diff test decides whether this is the
//! same screen.
//!
//! What could NOT be expressed and had to change the framework: the link cell.
//! What could: the computed column, and the money — but read the note on
//! `LOCALE` before believing that second one.

use crate::descriptor::{CellKind, Column, TableDescriptor};

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

/// THE HONEST PART.
///
/// A `Column`'s accessor is `fn(&T) -> String`, which cannot capture. The
/// locale comes from the *route*, so an accessor cannot see it — and the only
/// way to keep the descriptor a `const` is to reach for a global.
///
/// This is not a fix. It is the smallest thing that compiles, and it is worse
/// than the problem: two descriptors that differ only by locale cannot exist
/// at once, and the value a cell renders now depends on when it is read. It is
/// here so the zero-diff test can run against something, and so the cost is
/// visible in code rather than described in prose.
///
/// The real answer is `Box<dyn Fn(&T) -> String + Send + Sync>`, at which point
/// the descriptor stops being `const` data — which was flagged on day one as
/// the thing most likely to break the thesis. It did.
static LOCALE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn set_locale(l: Locale) {
    LOCALE.store(matches!(l, Locale::De) as u8, std::sync::atomic::Ordering::Relaxed);
}

fn locale() -> Locale {
    if LOCALE.load(std::sync::atomic::Ordering::Relaxed) == 1 { Locale::De } else { Locale::En }
}

pub fn descriptor(l: Locale) -> TableDescriptor<Invoice> {
    let de = matches!(l, Locale::De);
    TableDescriptor {
        title: if de { "Rechnungen" } else { "Invoices" },
        columns: if de { &COLUMNS_DE } else { &COLUMNS_EN },
    }
}

macro_rules! columns {
    ($name:ident, $number:expr, $client:expr, $amount:expr, $due:expr, $status:expr, $late:expr) => {
        static $name: [Column<Invoice>; 6] = [
            Column { header: $number, kind: CellKind::Link { href: |i| format!("/invoices/{}", i.number) }, get: |i| i.number.to_string() },
            Column { header: $client, kind: CellKind::Text,   get: |i| i.client.to_string() },
            Column { header: $amount, kind: CellKind::Number, get: |i| money(locale(), i.amount_cents) },
            Column { header: $due,    kind: CellKind::Date,   get: |i| i.due.to_string() },
            Column { header: $status, kind: CellKind::Badge,  get: |i| i.status.to_string() },
            Column { header: $late,   kind: CellKind::Number, get: |i| late_label(locale(), i.days_late) },
        ];
    };
}

columns!(COLUMNS_EN, "Number", "Client", "Amount", "Due", "Status", "Late");
columns!(COLUMNS_DE, "Nummer", "Kunde", "Betrag", "Fällig", "Status", "Verzug");
