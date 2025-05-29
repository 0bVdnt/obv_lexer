// This file serves as the root of the `lexer` module.
// When another part of the crate (like `src/main.rs`) declares `mod lexer;`,
// Rust looks for `src/lexer.rs` or `src/lexer/mod.rs`. Since this file exists,
// it becomes the entry point for the `lexer` module.

// --- 1. Declare Sub-modules ---
// The `mod` keyword followed by a name declares a submodule.
// Rust will look for files named `token.rs`, `error.rs`, and `core.rs`
// (or directories `token/mod.rs`, etc.) within the same directory as this `mod.rs` file
// (i.e., within `src/lexer/`).
// These lines effectively bring the contents of those files into the `lexer` module's scope,
// under their respective submodule names (e.g., `lexer::token`, `lexer::error`).
mod core; // Declares the `core` submodule, sourcing from `src/lexer/core.rs`.
mod error; // Declares the `error` submodule, sourcing from `src/lexer/error.rs`.
mod token; // Declares the `token` submodule, sourcing from `src/lexer/token.rs`.

// --- 2. Re-export Public Items ---
// The `pub use` keyword is used to re-export items from the submodules,
// making them directly accessible from outside the `lexer` module as if they were
// defined directly within `lexer`.
// For example, after `pub use token::Token;`, code outside this module can write
// `use my_crate::lexer::Token;` instead of the more verbose `use my_crate::lexer::token::Token;`.
// This creates a cleaner public API for the `lexer` module.

// Re-export the `Token` enum from the `token` submodule.
pub use core::Lexer;

// Re-export the `LexerError` enum from the `error` submodule.
pub use error::LexerError;

// Re-export the `Lexer` struct from the `core` submodule.
// This makes the main lexer functionality available.
pub use token::Token;
