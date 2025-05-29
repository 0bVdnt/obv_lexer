// Import the `fmt` module from the standard library (`std`).
// This module provides functionality for formatted output, which
// is used to implement the `Display` trait for custom error types.
use std::fmt;

// Import the `Serialize` trait from the `serde` crate.
// This allows `LexerError` enum to be converted into formats like JSON,
// which is useful if there is a need to communicate errors to other
// programs or log them in a structured way.
use serde::Serialize;

// Definition of the `LexerError` enumeration.
// This enum represents the different kinds of errors that can occur during
// the lexical analysis (tokenization) phase.
//
// `#[derive(...)]` is an attribute for automatic trait implementations:
// - `Debug`: Allows instances of `LexerError` to be printed with `{:?}` for debugging.
// - `PartialEq`: Allows comparing `LexerError` instances, useful for testing error conditions.
// - `Serialize`: Enables serialization of `LexerError` instances into formats like JSON.
#[derive(Debug, PartialEq, Serialize)]
pub enum LexerError {
    // Variant representing an error where an unexpected character is encountered.
    // This means a character was found that cannot start any known token pattern.
    //
    // `#[serde(rename = "unexpected_character")]` is a `serde` attribute.
    // It specifies that when this variant is serialized to JSON (or other formats),
    // the field name for this variant should be "unexpected_character" instead of
    // the default "UnexpectedCharacter". This is done to maintain the writing style
    // here, snake_case.
    //
    // This variant uses named fields (`char` and `pos`) for clarity in both Rust code
    // and the serialized output (i.e., JSON).
    #[serde(rename = "unexpected_character")]
    UnexpectedCharacter {
        char: char, // The actual unexpected character that was encountered.
        pos: usize, // The byte offset (position) in the input string where the character was found
    },

    // Variant representing an error where a sequence of digits was found that
    // looked like an integer constant, but it could not be parsed into a valid
    // integer (example case: it was too large for an `i32`)
    #[serde(rename = "invalid_integer")]
    InvalidInteger {
        value: String, // The string representation of the malformed integer.
        pos: usize,    // The starting position of this malformed integer in the input.
    },

    // Variant representing a situation where, at the current position in the input,
    // no defined token pattern (regex) could be matched. This is a more general
    // error than `UnexpectedCharacter` if the lexer can't even identify a single
    // problematic character and is simply "stuck."
    #[serde(rename = "no_match")]
    NoMatch {
        pos: usize, // The position in the input string where no token rule could be applied.
    },
}

// Implementation of the `std::fmt::Display` trait for `LexerError`.
// The `Display` trait is used to provide a user-friendly, human-readable
// string representation of a type. This is what gets printed when using
// the `{}` formatting placeholder (e.g., `println!("Error: {}", my_error);`)
impl fmt::Display for LexerError {
    // The `fmt` method takes a mutable reference to a `Formatter` and writes
    // the string representation into it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `match` is used to handle each variant of the `LexerError` enum differently.
        match self {
            // If the error is `UnexpectedCharacter`, format a specific message.
            // `char` and `pos` are destructured from the `UnexpectedCharacter` variant.
            LexerError::UnexpectedCharacter { char, pos } => {
                // `write!` is a macro similar to `println!`, but it writes to the
                // provided `Formatter` (`f`) instead of standard output.
                write!(f, "Unexpected character '{}' at position {}", char, pos)
            }

            // If the error is `InvalidInteger`, format its specific message.
            LexerError::InvalidInteger { value, pos } => {
                write!(
                    f,
                    "Invalid integer constant '{}' at position {}",
                    value, pos
                )
            }
            // If the error is `NoMatch`, format its specific message.
            LexerError::NoMatch { pos } => {
                write!(f, "No token matched at position {}", pos)
            }
        }
    }
}

// Implemention of the `std::error::Error` trait for `LexerError`.
// The `Error` trait is the base trait for all error types in Rust.
// Implementing it allows `LexerError` to be used with Rust's standard error
// handling mechanisms, such as the `?` operator, and to be composed with
// other error types.
// An empty implementation (`{}`) is often sufficient if the error type
// doesn't need to provide a "source" for the error (i.e., it's not wrapping another error).
impl std::error::Error for LexerError {}
