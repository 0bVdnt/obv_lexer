// --- 1. Declare the `lexer` module ---
// The `mod lexer;` statement tells the Rust compiler to look for and include
// the `lexer` module. Based on Rust's module discovery rules, it will find
// `src/lexer/mod.rs` (because a directory `src/lexer/` exists) and treat
// that file as the root of the `lexer` module.
mod lexer;

// --- 2. Import necessary items ---
// `use` statements bring specific items from modules into the current scope,
// allowing them to be used without full qualification.

// Import `Lexer`, `Token`, and `LexerError` from our `lexer` module.
// These were re-exported in `src/lexer/mod.rs`, making them directly
// accessible under the `lexer` namespace.
use lexer::{Lexer, LexerError, Token};

// Import from the standard library (`std`):
// `env` module for interacting with the execution environment,
// specifically for accessing command-line arguments (`env::args`).
// `fs` module for file system operations, specifically `read_to_string`.
// `io` module for input/output operations.
// `self` imports the `io` module itself (for `io::Result`, `io::stderr`).
// `Write` trait is imported for methods like `write_all` on `stderr`.
use std::{
    env, fs,
    io::{self, Write},
};

// Import from third-party crates (defined in `Cargo.toml`):
use serde::Serialize; // The `Serialize` trait from `serde` is needed for the
                      // `CompilerOutput` enum to derive it, enabling JSON serialization.
use serde_json; // The `serde_json` crate provides functions for serializing
                // Rust data structures to JSON strings and vice-versa.

// --- 3. Define `CompilerOutput` Enum ---
// This enum is a utility for structuring the program's final output,
// especially when serializing to JSON. It allows us to represent either
// a successful outcome (a list of tokens) or an error in a single, unified type.
//
// `#[derive(serde::Serialize)]`: This attribute automatically generates the code
// needed to serialize `CompilerOutput` instances into formats supported by `serde` (like JSON).
#[derive(Serialize)]
// `#[serde(untagged)]`: This is a `serde` attribute that affects how enums are serialized.
// "Untagged" means that when serializing, `serde` will not add an extra field to the JSON
// to indicate which variant of the enum it is. Instead, it will try to serialize the
// data *inside* the variant directly.
// - If `CompilerOutput::Success(tokens)`, the JSON will be an array `[...]` (the tokens).
// - If `CompilerOutput::Error(error)`, the JSON will be an object `{...}` (the error details).
// This relies on the serialized forms of `Vec<Token>` and `LexerError` being distinct.
// #[serde(untagged)]
enum CompilerOutput {
    Success(Vec<Token>), // Variant for successful lexing, holding the vector of tokens.
    Error(LexerError),   // Variant for a lexing error, holding the `LexerError` instance.
}

// `main` is the entry point function for the Rust application.
// `-> io::Result<()>`: The return type indicates that `main` can return an I/O error
// (`std::io::Error`). `Ok(())` signifies success with no specific value, while `Err(io_error)`
// would signify an I/O failure. This allows using the `?` operator for I/O operations within `main`.
fn main() -> io::Result<()> {
    // --- 4. Handle Command-Line Arguments and Read Source Code ---
    // `env::args()`: Returns an iterator over the command-line arguments passed to the program.
    // The first argument (`args[0]`) is typically the path to the executable itself.
    // `.collect()`: Collects the arguments from the iterator into a `Vec<String>`.
    let args: Vec<String> = env::args().collect();

    // `source_code`: This variable will hold the source code string to be lexed.
    let source_code = if args.len() > 1 {
        // If there is more than one argument, it means a file path was likely provided
        // as the second argument (`args[1]`).
        // Get a reference to the file path string.
        let file_path = &args[1];
        // `fs::read_to_string(file_path)`: Attempts to read the entire content of the
        // specified file into a `String`. This operation can fail (e.g., file not found,
        // no permission), so it returns an `io::Result<String>`.
        // The `?` operator is used here: if `read_to_string` returns an `Err(io_error)`,
        // the `?` operator will immediately return that `Err(io_error)` from the `main` function.
        // If it's `Ok(content)`, `content` is assigned to `source_code`.
        fs::read_to_string(file_path)?
    } else {
        // If no file path argument is provided, use a default hardcoded string for demonstration.
        // `eprintln!`: Prints to standard error (`stderr`). This is good for informational
        // messages or errors that shouldn't be part of the primary output (which goes to `stdout`).
        eprintln!("No source file provided. Use default example code.");
        "int main () { return 0; }".to_string() // Convert `&str` to `String`
    };
    // Print the source code being processed to `stderr` for user visibility.
    eprintln!("--- Source Code ---");
    eprintln!("{}", source_code);
    eprintln!("-------------------");

    // --- 5. Instantiate and Run the Lexer ---
    // Create a new `Lexer` instance, passing a reference to the `source_code`.
    // `lexer_instance` needs to be mutable (`mut`) because `tokenize_all` (which calls
    // `next_token_internal`) modifies the lexer's internal `position`.
    let mut lexer_instance = Lexer::new(&source_code);

    // Call `tokenize_all()` on the lexer instance. This attempts to convert the
    // entire `source_code` into a sequence of tokens.
    // `match lexer_instance.tokenize_all()`: Handle the `Result` returned by `tokenize_all`.
    let output = match lexer_instance.tokenize_all() {
        // If `tokenize_all` returns `Ok(tokens)`, lexing was successful.
        // Wrap the `tokens` vector in the `CompilerOutput::Success` variant.
        Ok(tokens) => CompilerOutput::Success(tokens),
        // If `tokenize_all` returns `Err(e)`, a lexing error occurred.
        // Wrap the `LexerError` instance `e` in the `CompilerOutput::Error` variant.
        Err(e) => CompilerOutput::Error(e),
    };

    // --- 6. Serialize Output to JSON and Print to Standard Output (`stdout`) ---
    // `serde_json::to_string_pretty(&output)`: Attempts to serialize the `output`
    // (which is a `CompilerOutput` enum instance) into a JSON string.
    // `to_string_pretty` formats the JSON with indentation for human readability.
    // This operation can also fail (though rarely, e.g., if a type cannot be serialized),
    // so it returns a `Result<String, serde_json::Error>`.
    match serde_json::to_string_pretty(&output) {
        // If serialization is successful (`Ok(json_string)`):
        Ok(json_string) => {
            // `println!("{}", json_string)`: Print the resulting JSON string to standard output.
            // This is the primary way this lexer communicates its results to other tools or scripts.
            println!("{}", json_string);
        }
        // If JSON serialization itself fails (`Err(e)`):
        Err(e) => {
            // This is an internal error of the lexer program, not a lexing error of the source code.
            // Construct an error message.
            let error_msg = format!(
                "Internal Error: Failed to serialize lexer output to JSON: {}",
                e
            );
            // Write the error message to standard error.
            // `io::stderr()`: Gets a handle to the standard error stream.
            // `.write_all(error_msg.as_bytes())?`: Writes the byte representation of the message.
            // The `?` here will propagate any `io::Error` from `write_all`.
            io::stderr().write_all(error_msg.as_bytes())?;
            io::stderr().write_all(b"\n")?;
            // Write a newline for better formatting.
            // `std::process::exit(1)`: Terminate the program immediately with a non-zero exit code (1),
            // which conventionally indicates failure.
            std::process::exit(1);
        }
    }

    // --- 7. Set Program Exit Code Based on Lexing Outcome ---
    // Even if JSON serialization was successful, we need to set the program's exit code
    // to reflect whether the *lexing* of the source code was successful.
    // This is important for scripting and build tools that check exit codes.
    if let CompilerOutput::Error(_) = output {
        // If the `output` was the `Error` variant (meaning a `LexerError` occurred),
        // exit the program with a status code of 1 to indicate failure.
        std::process::exit(1);
    }
    // If the program reaches this point, it means:
    // 1. Source code was read (or default was used).
    // 2. Lexing resulted in `CompilerOutput::Success` (no `LexerError`).
    // 3. JSON serialization was successful.
    // So, the program execution was successful overall.
    // Returning `Ok(())` from `main` results in an exit code of 0 (success).
    Ok(())
}
