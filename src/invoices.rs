//! Screen two, declared as a descriptor.
//!
//! Refactored from the hand-written version (git history) after the framework
//! gained `CellKind::Link`. The zero-diff test decides whether this is the
//! same screen.
//!
//! The global that made the locale work is gone. The accessors are boxed
//! closures now, so they capture the locale directly — see `Accessor` in
//! `descriptor.rs` for what that cost.

use crate::descriptor::{CellKind, TableDescriptor, col};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    De,
    En,
}

/// Now a Toasty model rather than a plain struct. Note what that did to the
/// field types: `&'static str` became `String`, because rows come from a
/// database rather than a literal. Every descriptor accessor already returned
/// `String`, so none of them changed — the boundary absorbed it.
#[derive(Debug, toasty::Model)]
pub struct Invoice {
    #[key]
    #[auto]
    pub id: uuid::Uuid,
    pub number: String,
    pub client: String,
    pub amount_cents: i64,
    pub due: String,
    pub status: String,
    pub days_late: i64,
}

/// Seeds an in-memory SQLite database and reads the rows back out.
///
/// Same three invoices as the fixture it replaces — the zero-diff test has to
/// keep passing, so the data must be identical. What changed is where it comes
/// from and, crucially, that getting it is now `async`.
pub async fn rows() -> Vec<Invoice> {
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::invoices::Invoice))
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    db.push_schema().await.expect("schema");

    for (number, client, cents, due, status, late) in [
        (
            "2026-0041",
            "Stadtbibliothek München",
            485000i64,
            "2026-08-15",
            "paid",
            0i64,
        ),
        (
            "2026-0042",
            "Landratsamt Starnberg",
            1240050,
            "2026-08-28",
            "sent",
            5,
        ),
        (
            "2026-0043",
            "Caritas Berlin",
            99000,
            "2026-09-10",
            "draft",
            0,
        ),
    ] {
        toasty::create!(Invoice {
            number: number,
            client: client,
            amount_cents: cents,
            due: due,
            status: status,
            days_late: late,
        })
        .exec(&mut db)
        .await
        .expect("seed");
    }

    Invoice::all().exec(&mut db).await.expect("query")
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

fn late_label(locale: Locale, days: i64) -> String {
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
            col(
                if de { "Nummer" } else { "Number" },
                CellKind::Link {
                    href: Box::new(|i: &Invoice| format!("/invoices/{}", i.number)),
                },
                |i: &Invoice| i.number.to_string(),
            ),
            col(
                if de { "Kunde" } else { "Client" },
                CellKind::Text,
                |i: &Invoice| i.client.to_string(),
            ),
            col(
                if de { "Betrag" } else { "Amount" },
                CellKind::Number,
                move |i: &Invoice| money(l, i.amount_cents),
            ),
            col(
                if de { "Fällig" } else { "Due" },
                CellKind::Date,
                |i: &Invoice| i.due.to_string(),
            ),
            col("Status", CellKind::Badge, |i: &Invoice| {
                i.status.to_string()
            }),
            col(
                if de { "Verzug" } else { "Late" },
                CellKind::Number,
                move |i: &Invoice| late_label(l, i.days_late),
            ),
        ],
    }
}
