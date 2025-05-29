// Import the `Serialize` trait from the `serde` crate.
// This trait is used to enable the conversion of our `Token` enum
// into formats like JSON, which is useful for IPC (Inter-Process Communication)
// or for saving/loading token streams.
use serde::Serialize;

// Define the `Token` enumeration.
// An enum is a custom type that can be one of several possible variants.
// Here, each variant represents a distinct type of token found in the source code.
//
// `#[derive(...)]` is an attribute that tells the Rust compiler to automatically
// generate implementations for certain traits.
// - `Debug`: Allows instances of `Token` to be printed using the `{:?}` formatter,
//   which is very helpful for debugging purposes.
// - `PartialEq`: Allows instances of `Token` to be compared for equality using `==` and `!=`.
//   This is essential for writing assertions in unit tests.
// - `Clone`: Allows creating a deep copy of a `Token` instance. This is needed because
//   we store `Token` variants (which are `Copy` types like `KwInt`) in the `KEYWORDS`
//   array, and when we retrieve them, we need an owned copy. Variants with owned
//   data like `Identifier(String)` also benefit from `Clone` if copies are needed.
// - `Serialize`: Enables this enum to be serialized by `serde` into formats like JSON.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Token {
    // --- Keyword Tokens ---
    // These variants represent reserved keywords in the language.
    // They do not carry any extra data because the token type itself is sufficient information.
    KwInt,    // Represents the "int" keyword.
    KwVoid,   // Represents the "void" keyword.
    KwReturn, // Represents the "return" keyword.

    // --- Identifier Token ---
    // Represents a user-defined name (e.g., variable name, function name).
    // It holds a `String` which is the actual name of the identifier.
    // Example: For `main`, this token would be `Identifier("main".to_string())`. // Identifiers (ex. main, foo, bar, return)
    Identifier(String),

    // --- Constant Token ---
    // Represents an integer literal found in the source code.
    // It holds an `i32` (a 32-bit signed integer) which is the numerical value of the constant.
    // Example: For `123`, this token would be `Constant(123)`.
    Constant(i32),

    // --- Punctuation/Symbol Tokens ---
    // These variants represent single characters or sequences of characters
    // that have special meaning in the language's syntax.
    OpenParen,  // Represents an opening parenthesis: `(`.
    CloseParen, // Represents a closing parenthesis: `)`.
    OpenBrace,  // Represents an opening curly brace: `{`.
    CloseBrace, // Represents a closing curly brace: `}`.
    Semicolon,  // Represents a semicolon: `;`.
}
