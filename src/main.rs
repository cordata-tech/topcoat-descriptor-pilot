//! Topcoat + Toasty pilot — does the descriptor boundary survive the move to Rust?
//!
//! See README.md for what this is testing and NOTES.md for what it found.

mod descriptor;
mod invoices;
mod probe;
mod table;
mod users;

use topcoat::{
    Result,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::view,
};

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}

#[page("/")]
async fn home() -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Topcoat descriptor pilot"</title>
                topcoat::dev::script()
            </head>
            <body>
                table::table_page(
                    title: users::USERS.title,
                    columns: users::USERS.columns,
                    rows: users::rows(),
                )
            </body>
        </html>
    }
}

// Screen two, hand-written — see src/invoices.rs. Two routes, because the
// locale comes from the URL exactly as it does on cordata.tech. That is what
// the descriptor has to reproduce.
#[page("/invoices")]
async fn invoices_en() -> Result {
    invoices::set_locale(invoices::Locale::En);
    view! {
        <!DOCTYPE html>
        <html>
            <head><title>"Invoices"</title>topcoat::dev::script()</head>
            <body>
                table::table_page(
                    title: invoices::descriptor(invoices::Locale::En).title,
                    columns: invoices::descriptor(invoices::Locale::En).columns,
                    rows: invoices::rows(),
                )
            </body>
        </html>
    }
}

#[page("/de/invoices")]
async fn invoices_de() -> Result {
    invoices::set_locale(invoices::Locale::De);
    view! {
        <!DOCTYPE html>
        <html>
            <head><title>"Rechnungen"</title>topcoat::dev::script()</head>
            <body>
                table::table_page(
                    title: invoices::descriptor(invoices::Locale::De).title,
                    columns: invoices::descriptor(invoices::Locale::De).columns,
                    rows: invoices::rows(),
                )
            </body>
        </html>
    }
}
