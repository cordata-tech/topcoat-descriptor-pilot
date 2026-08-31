//! Screen one: the screen we build first and learn to trust.
//!
//! The zero-diff test needs a screen that already works before a second one is
//! added. This is it. Do not add the second screen until this one is trusted.

use crate::descriptor::{CellKind, Column, TableDescriptor};

pub struct User {
    pub name: &'static str,
    pub email: &'static str,
    pub seats: u32,
    pub joined: &'static str,
    pub status: &'static str,
}

pub const USERS: TableDescriptor<User> = TableDescriptor {
    title: "Users",
    columns: &[
        Column {
            header: "Name",
            kind: CellKind::Text,
            get: |u| u.name.to_string(),
        },
        Column {
            header: "Email",
            kind: CellKind::Text,
            get: |u| u.email.to_string(),
        },
        Column {
            header: "Seats",
            kind: CellKind::Number,
            get: |u| u.seats.to_string(),
        },
        Column {
            header: "Joined",
            kind: CellKind::Date,
            get: |u| u.joined.to_string(),
        },
        Column {
            header: "Status",
            kind: CellKind::Badge,
            get: |u| u.status.to_string(),
        },
    ],
};

/// Fixture data. Toasty replaces this in step 2 — see NOTES.md Q4.
pub fn rows() -> Vec<User> {
    vec![
        User {
            name: "Ada Lovelace",
            email: "ada@example.org",
            seats: 3,
            joined: "2026-01-14",
            status: "active",
        },
        User {
            name: "Grace Hopper",
            email: "grace@example.org",
            seats: 12,
            joined: "2026-03-02",
            status: "active",
        },
        User {
            name: "Karen Spärck",
            email: "karen@example.org",
            seats: 1,
            joined: "2026-07-30",
            status: "invited",
        },
    ]
}
