//! # Zekurix Server
//!
//! **Zero-Knowledge Hierarchical Collaboration Platform - Server Component**
//!
//! <div style="background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 0.75em 1em; margin: 1em 0;">
//! <strong>⚠ Warning:</strong> This project is in an early prototype stage.<br/>
//! Core functionality is still under development and many planned features are not yet implemented.
//! </div>
//!
//! ## Project Scope
//!
//! Zekurix Server is one component of the broader Zekurix ecosystem.
//!
//! The project is intended to evolve into a larger ecosystem that may include:
//! - Zekurix Server
//! - Cross-platform SDK
//! - Web client
//! - Mobile clients
//! - Desktop clients
//!
//! This repository contains only the backend server component.
//!
//! ## Planned Features
//!
//! - Zero-knowledge encryption architecture
//! - Hierarchical permission model
//! - Group-based collaboration
//! - Vector clock-based synchronization
//! - Deterministic deletion through key shredding
#[doc(hidden)]
pub mod app;
#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod database;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod health;
#[doc(hidden)]
pub mod openapi;
#[doc(hidden)]
pub mod router;
#[doc(hidden)]
pub mod secrets;
#[doc(hidden)]
pub mod settings;
#[doc(hidden)]
pub mod telemetry;
#[doc(hidden)]
pub mod user;

#[doc(hidden)]
pub use app::Application;
