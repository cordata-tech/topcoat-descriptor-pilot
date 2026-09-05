//! Screen two, hand-written — the "before" half of the zero-diff test.
//!
//! Deliberately harder than `users`, which is all Text/Number/Date/Badge and
//! would prove nothing on refactor. This screen carries three things the
//! descriptor cannot currently express:
//!
//!   1. a money amount formatted for a locale that comes from the REQUEST,
//!      not from the row — `4.850,00 €` in German, `$4,850.00` in English
//!   2. a column computed from two fields
//!   3. a cell that is a link, not text
//!
//! (1) is the one that matters. If the locale lived on the row, a plain `fn`
//! accessor could reach it. It doesn't — it comes from the URL, exactly as it
//! does on cordata.tech itself — so the accessor would have to *capture* it,
//! and `fn(&T) -> String` cannot capture.

use topcoat::{Result, view::{component, view}};

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
///
/// Written out rather than pulled from a crate so the refactor has something
/// concrete to fail to express. Note the separators swap *and* the symbol
/// moves side — this is not a formatting flag, it is a different shape.
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

#[component]
pub async fn invoices_page(locale: Locale, rows: Vec<Invoice>) -> Result {
    let (h_number, h_client, h_amount, h_due, h_status, h_late) = match locale {
        Locale::De => ("Nummer", "Kunde", "Betrag", "Fällig", "Status", "Verzug"),
        Locale::En => ("Number", "Client", "Amount", "Due", "Status", "Late"),
    };
    view! {
        <h1>(if locale == Locale::De { "Rechnungen" } else { "Invoices" })</h1>
        <table>
            <thead>
                <tr>
                    <th>(h_number)</th>
                    <th>(h_client)</th>
                    <th>(h_amount)</th>
                    <th>(h_due)</th>
                    <th>(h_status)</th>
                    <th>(h_late)</th>
                </tr>
            </thead>
            <tbody>
                for inv in &rows {
                    <tr>
                        <td class="cell cell-text">
                            <a href=(format!("/invoices/{}", inv.number))>(inv.number)</a>
                        </td>
                        <td class="cell cell-text">(inv.client)</td>
                        <td class="cell cell-number">(money(locale, inv.amount_cents))</td>
                        <td class="cell cell-date">(inv.due)</td>
                        <td class="cell cell-badge">(inv.status)</td>
                        <td class="cell cell-number">(late_label(locale, inv.days_late))</td>
                    </tr>
                }
            </tbody>
        </table>
    }
}
