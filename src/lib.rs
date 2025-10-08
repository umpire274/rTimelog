//! rTimelogger — a simple command-line time logging tool.
//!
//! This crate provides the core library used by the `rtimelog` CLI. It contains
//! modules for command-line parsing, configuration, database access, event
//! modeling, and the business logic driving the application.
//!
//! The main crate documentation is included from the repository `README.md` so
//! that docs.rs and `cargo doc` present the same introduction and examples as
//! the project README.
#![allow(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

pub mod cli;
pub mod config;
pub mod db;
pub mod events;
pub mod export;
pub mod logic;
pub mod utils;
