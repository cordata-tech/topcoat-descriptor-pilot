//! Topcoat + Toasty pilot — does the descriptor boundary survive the move to Rust?
//!
//! See README.md for what this is testing and NOTES.md for what it found.

mod descriptor;
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
