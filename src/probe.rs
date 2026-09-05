//! Scratch file for the Q2 probes — see Q2-PROBES.md.
//!
//! Deliberately not wired into any route. Uncomment one probe at a time,
//! run `cargo build 2>&1 | head -40`, record what it says, re-comment it.
//!
//! Nothing here is meant to compile.

#![allow(dead_code, unused_imports)]

use topcoat::{
    view::{component, view},
    Result,
};

// ── P1 — a value that isn't Send ────────────────────────────────────────
// use std::rc::Rc;
//
// #[component]
// pub async fn probe() -> Result {
//     let name = Rc::new(String::from("Ada"));
//     view! { <p>(name)</p> }
// }

// ── P2 — a borrow that outlives the view ────────────────────────────────
// #[component]
// pub async fn probe() -> Result {
//     let v = {
//         let local = String::from("temporary");
//         view! { <p>(&local)</p> }
//     };
//     v
// }

// ── P3 — a $() expression that cannot cross to JavaScript ───────────────
// async fn server_only() -> String { String::from("from the database") }
//
// #[component]
// pub async fn probe() -> Result {
//     view! {
//         <button @click=$(server_only())>"click"</button>
//     }
// }

// ── P4 — malformed markup ───────────────────────────────────────────────
// #[component]
// pub async fn probe() -> Result {
//     view! { <div><p>"unclosed div"</div> }
// }

// ── P5 — a type that cannot render ──────────────────────────────────────
// struct NotDisplayable;
//
// #[component]
// pub async fn probe() -> Result {
//     view! { <p>(NotDisplayable)</p> }
// }
