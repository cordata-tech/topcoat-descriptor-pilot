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
                    descriptor: users::users(),
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
    view! {
        <!DOCTYPE html>
        <html>
            <head><title>"Invoices"</title>topcoat::dev::script()</head>
            <body>
                table::table_page(
                    descriptor: invoices::descriptor(invoices::Locale::En),
                    rows: invoices::rows(),
                )
            </body>
        </html>
    }
}

#[page("/de/invoices")]
async fn invoices_de() -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head><title>"Rechnungen"</title>topcoat::dev::script()</head>
            <body>
                table::table_page(
                    descriptor: invoices::descriptor(invoices::Locale::De),
                    rows: invoices::rows(),
                )
            </body>
        </html>
    }
}
