//! MCP (Model Context Protocol) server implementation.
//!
//! This module implements the server side of the MCP wire protocol used by
//! AI coding assistants (Claude Code, `OpenCode`, etc.) to interact with
//! Mercury Cortex.  It provides JSON-RPC 2.0 transport over stdio, session
//! management, method routing, and handler registration.
//!
//! The server types (handlers, router, server loop) are wired into the
//! `mercury-cortex mcp serve` CLI command and served over stdio.

pub mod context;
pub mod error;
pub mod handler;
pub mod helpers;
pub mod models;
pub mod session;
pub mod tools;
pub mod transport;

pub use error::{McpError, McpResult};
