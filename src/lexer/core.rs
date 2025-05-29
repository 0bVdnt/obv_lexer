// --- Imports ---

// Import the `Regex` type from the `regex` crate.
// This type is used to define and match regular expressions against text.
use regex::Regex;

// Import the `lazy_static` macro from the `lazy_static` crate.
// This macro is used to ensure that complex static variables (like compiled regexes)
// are initialized only once, the first time they are accessed, in a thread-safe manner.
// This is more efficient than recompiling regexes every time they are needed.
use lazy_static::lazy_static;

// Import the `Token` enum from the sibling module `token.rs` within the `lexer` module.
// `super::` refers to the parent module of the current file (`core.rs`), which is `lexer` (defined by `lexer/mod.rs`).
// So, `super::token` refers to `lexer::token`.
use super::error::LexerError;

// Import the `LexerError` enum from the sibling module `error.rs`.
use super::token::Token;

// --- Regular Expression Definitions ---
// The `lazy_static!` block is used to define static `Regex` instances.
// Compiling regexes can be somewhat expensive, so doing it once at program
// startup (or first use) and storing them statically improves performance.
lazy_static! {
    // Regex for matching Identifiers.
    // - `\A`: Anchors the match to the beginning of the string slice being tested. This is crucial
    //   because we always want to match tokens from the current lexer position.
    // - `[a-zA-Z_]`: Matches a single ASCII letter (uppercase or lowercase) or an underscore.
    //   This defines the allowed starting characters for an identifier.
    // - `\w*`: Matches zero or more "word" characters. In many regex engines (including Rust's default),
    //   `\w` is equivalent to `[a-zA-Z0-9_]`. So, this part matches subsequent letters, digits, or underscores.
    // - `\b`: Matches a "word boundary". This ensures that the identifier doesn't immediately blend
    //   into another word character. For example, `int` in `int main` is a match because space is a boundary,
    //   but `int` in `intFoo` would match `intFoo` entirely (if `Foo` were valid word chars).
    //   More importantly, for something like `return2`, `return` would match because `2` forms a boundary.
    //   For `123bar`, `123` (if it were an identifier) would fail to match `123` alone if `CONSTANT_RE` didn't also use `\b`.
    // `.unwrap()`: `Regex::new` returns a `Result`. We `.unwrap()` here because if these fundamental
    //   regexes are invalid, the program cannot function, so panicking is acceptable at startup.
    static ref IDENTIFIER_RE: Regex = Regex::new(r"\A[a-zA-Z_]\w*\b").unwrap();

    // Regex for matching (Integer) Constants.
    // - `\A`: Anchors to the beginning of the slice.
    // - `[0-9]+`: Matches one or more ASCII digits (0 through 9).
    // - `\b`: Matches a word boundary. This prevents `123` from matching in `123foo` if `foo` starts
    //   with a word character, ensuring the constant is properly terminated.
    static ref CONSTANT_RE: Regex = Regex::new(r"\A[0-9]+\b").unwrap();

    // Regexes for simple punctuation tokens. These are very straightforward.
    // They match the literal character at the beginning of the slice.
    // `\(` and `\)`: Parentheses need to be escaped in regex because `(` and `)` have special meaning (for grouping).
    static ref OPEN_PAREN_RE: Regex = Regex::new(r"\A\(").unwrap();
    static ref CLOSE_PAREN_RE: Regex = Regex::new(r"\A\)").unwrap();
    // `{` and `}`: Braces also need escaping in many regex flavors for their grouping/quantifier meaning.
    static ref OPEN_BRACE_RE: Regex = Regex::new(r"\A\{").unwrap();
    static ref CLOSE_BRACE_RE: Regex = Regex::new(r"\A\}").unwrap();
    // `;`: Semicolon does not have a special regex meaning here, so it doesn't strictly need escaping,
    //   but escaping non-alphanumeric characters consistently is not harmful.
    static ref SEMICOLON_RE: Regex = Regex::new(r"\A;").unwrap();

    // Regexes for skipping non-token parts of the input.
    // - Whitespace:
    //   - `\A`: Anchor.
    //   - `\s+`: Matches one or more whitespace characters (spaces, tabs, newlines, etc.).
    static ref WHITESPACE_RE: Regex = Regex::new(r"\A\s+").unwrap();

    // - Single-line comments:
    //   - `\A//`: Matches the literal `//` at the beginning of the slice.
    //   - `.*`: Matches any character (except newline, by default for `.`) zero or more times.
    //     This consumes the rest of the line after `//`.
    static ref SINGLE_LINE_COMMENTS_RE: Regex = Regex::new(r"\A//.*").unwrap();

    // - Multi-line comments:
    //   - `\A`: Anchor.
    //   - `(?s)`: An inline flag that enables "DOTALL" mode (also called "single-line mode" in some engines).
    //     In this mode, the `.` metacharacter will match *any* character, including newline characters (`\n`).
    //     This is crucial for multi-line comments that span across newlines.
    //   - `/\*`: Matches the literal `/*` sequence. The `*` is escaped with `\` because `*` is a
    //     special regex quantifier (meaning "zero or more of the preceding item").
    //   - `.*?`: Matches any character (`.`, now including newlines due to `(?s)`) zero or more times (`*`),
    //     but as few times as possible (`?`). This makes the `*` "non-greedy". It's important here
    //     to ensure it stops at the *first* occurrence of `*/`, not the last one in case of
    //     multiple comments or nested-looking structures (though this regex doesn't handle true nesting).
    //   - `\*/`: Matches the literal `*/` sequence, terminating the comment. The `*` is escaped.
    static ref MULTI_LINE_COMMENTS_RE: Regex = Regex::new(r"\A(?s)/\*.*?\*/").unwrap();
}

// --- Keyword Definitions ---
// A static array that maps string representations of keywords to their corresponding `Token` enum variants.
// This is used after an identifier is matched to check if it's actually a keyword.
// - `[(&str, Token); 3]`: Defines an array of 3 elements. Each element is a tuple `(&str, Token)`.
//   - `&str`: A string slice representing the keyword text (e.g., "int").
//   - `Token`: The corresponding `Token` enum variant (e.g., `Token::KwInt`).
// The `Token` variants here (like `Token::KwInt`) are `clone()`d from their definitions because
// `Token` itself derives `Clone`. This ensures that the `KEYWORDS` array owns its `Token` values.
const KEYWORDS: [(&str, Token); 3] = [
    ("int", Token::KwInt),
    ("void", Token::KwVoid),
    ("return", Token::KwReturn),
];

// --- Lexer Struct Definition ---
// The `Lexer` struct is the main structure responsible for the tokenization process.
// It holds the state needed to scan through the input source code.
// - `'a`: This is a lifetime parameter. It indicates that the `Lexer` struct holds a reference
//   (`input: &'a str`) that lives for at least as long as the lifetime `'a`. This means
//   the `Lexer` instance cannot outlive the input string it is borrowing.
pub struct Lexer<'a> {
    // `input`: A string slice (`&'a str`) representing the source code to be tokenized.
    // It's a reference to the original input string, meaning the lexer doesn't own the string data itself.
    input: &'a str,

    // `position`: A `usize` representing the current byte offset (index) within the `input` string.
    // This tracks how much of the input has been processed (consumed into tokens or skipped).
    position: usize,
}

// --- Lexer Implementation ---
// `impl<'a> Lexer<'a>` block defines methods associated with the `Lexer` struct.
// The lifetime parameter `'a` from the struct definition is also used here.
impl<'a> Lexer<'a> {
    // `new` is a constructor function for creating a `Lexer` instance.
    // It's a common convention in Rust to name constructors `new`.
    // - `input: &'a str`: Takes a string slice (with lifetime 'a) as the source code.
    // - `-> Self`: The return type `Self` is an alias for `Lexer<'a>` within this impl block.
    pub fn new(input: &'a str) -> Self {
        // Initialize and return a new `Lexer` instance.
        // - `input`: The provided input string slice is stored.
        // - `position`: The current parsing position is initialized to `0` (the beginning of the input).
        Lexer { input, position: 0 }
    }

    // `skip_whitespace_and_comments` is a helper method responsible for advancing
    // the lexer's `position` past any whitespace characters or comments.
    // It repeatedly tries to match and consume skippable patterns from the current position.
    // - `&mut self`: Takes a mutable reference to the `Lexer` instance because it modifies
    //   the `self.position` field.
    // - `-> bool`: Returns `true` if any character (whitespace or comment) was actually skipped
    //   during this call, and `false` otherwise. This return value isn't strictly used
    //   by the caller (`next_token_internal`) in this version, but it can be useful for debugging
    //   or more complex skipping logic.
    fn skip_whitespaces_and_comments(&mut self) -> bool {
        // `skipped_something`: A flag to track if any skipping occurred in this call.
        let mut skipped_something = false;

        // `loop`: An infinite loop that continues as long as skippable items are found.
        // The loop breaks when no skippable pattern matches at the current position.

        loop {
            // `current_slice`: Get a string slice representing the remaining part of the input
            // from the current `self.position` to the end.
            let current_slice = &self.input[self.position..];

            // If `current_slice` is empty, it means control has reached the end of the input.
            // There's nothing left to skip, so break the loop.
            if current_slice.is_empty() {
                break;
            }

            // --- Try to match and skip WHITESPACE ---
            // `WHITESPACE_RE.find(current_slice)` attempts to find a match for the whitespace regex
            // at the beginning of `current_slice`.
            // `if let Some(mat) = ...`: If a match is found (`mat` will be a `regex::Match` object).
            if let Some(mat) = WHITESPACE_RE.find(current_slice) {
                // `mat.end()`: Returns the length (in bytes) of the matched whitespace.
                // Advance `self.position` by this length to move past the skipped whitespace.
                self.position += mat.end();
                // Set the flag indicating that something was skipped.
                skipped_something = true;
                // `continue`: Skip the rest of the current loop iteration and start the next one.
                // This is because after skipping whitespace, there might be a comment or more whitespace.
                continue;
            }

            // --- Try to match and skip SINGLE-LINE COMMENTS ---
            // If whitespace wasn't found, try matching a single-line comment.
            if let Some(mat) = SINGLE_LINE_COMMENTS_RE.find(current_slice) {
                // Advance `self.position` past the entire matched single-line comment.
                self.position += mat.end();
                // Continue to the next loop iteration to check for more skippables.
                skipped_something = true;
                continue;
            }

            // --- Try to match and skip MULTI-LINE COMMENTS ---
            // If neither whitespace nor a single-line comment was found, try a multi-line comment.
            if let Some(mat) = MULTI_LINE_COMMENTS_RE.find(current_slice) {
                // NOTE: (on MULTI_LINE_COMMENT_RE) `(?s)/\*.*?\*/`
                // The `(?s)` flag allows `.` to match newlines. `.*?` is non-greedy.
                // This regex handles simple, non-nested block comments.
                // If an unterminated comment `/* ... EOF` occurs, this regex (because of `.*?`)
                // might consume until the end of the file if `*/` is never found.
                // TODO: Add a check for unterminated multiline comment
                self.position += mat.end();
                skipped_something = true;
                continue;
            }
            // If none of the skippable patterns (whitespace, single-line comment, multi-line comment)
            // matched in this iteration of the loop, it means the character(s) at the current
            // `self.position` are not skippable and might be the start of an actual token.
            // So, break out of the `loop`.
            break;
        }
        // Return whether anything was skipped.
        skipped_something
    }

    // `next_token_internal` is the heart of the lexer. It attempts to identify and
    // return the next token from the input stream, starting from the current `self.position`.
    // - `&mut self`: Takes a mutable reference to `Lexer` to update `self.position`.
    // - `-> Option<Result<Token, LexerError>>`: The return type is nested:
    //   - `Option<...>`: `Some(...)` if a token is found or an error occurs. `None` if the
    //     end of the input is reached (after skipping whitespace/comments).
    //   - `Result<Token, LexerError>`: If `Some`, this `Result` indicates success or failure:
    //     - `Ok(Token)`: A token was successfully recognized.
    //     - `Err(LexerError)`: An error occurred during tokenization.
    fn next_token_internal(&mut self) -> Option<Result<Token, LexerError>> {
        // --- Phase 1: Skip leading whitespace and comments ---
        // This loop ensures that `self.position` is advanced past any skippable
        // characters before attempting to recognize an actual token.
        loop {
            // Call the helper method to skip whitespace and comments.
            // The boolean result of `skip_whitespace_and_comments` is ignored here (`let _ = ...`)
            // as we only care that the position is updated.
            let _ = self.skip_whitespaces_and_comments();

            // After attempting to skip, check if we've reached the end of the input.
            if self.position >= self.input.len() {
                // If `self.position` is at or beyond the input length, it means all remaining
                // characters were skippable, or the input was empty to begin with.
                // Return `None` to signal the end of token stream.
                return None;
            }
            // If control reaches here which means that `self.position` points to a
            // potential start of a token, so the control breaks out of the skipping loop.
            break;
        }

        // --- Phase 2: Attempt to match known token patterns ---
        // `current_slice`: Get the part of the input string from the current `self.position`.
        // All regex matches will be attempted against the beginning of this slice.
        let current_slice = &self.input[self.position..];

        // `start_pos_of_token`: Store the current position. This is useful for error reporting,
        // as it indicates where the problematic (or successful) token began.
        let start_position_of_the_token = self.position;

        // The order of these `if let Some(mat) = ...` blocks can be important,
        // especially if some token patterns could ambiguously match the same prefix.
        // Typically, longer or more specific matches (like keywords, which are handled
        // after general identifiers here) or frequently occurring simple tokens
        // might be checked first. For this set of tokens, the order is relatively robust.

        // --- 2.1: Match Punctuation Tokens ---
        // These are usually single-character tokens with fixed representations.
        if let Some(mat) = OPEN_PAREN_RE.find(current_slice) {
            self.position += mat.end(); // Advance position by the length of the matched token.
            return Some(Ok(Token::OpenParen)); // Return the recognized token.
        }

        if let Some(mat) = CLOSE_PAREN_RE.find(current_slice) {
            self.position += mat.end();
            return Some(Ok(Token::CloseParen));
        }

        if let Some(mat) = OPEN_BRACE_RE.find(current_slice) {
            self.position += mat.end();
            return Some(Ok(Token::OpenBrace));
        }

        if let Some(mat) = CLOSE_BRACE_RE.find(current_slice) {
            self.position += mat.end();
            return Some(Ok(Token::CloseBrace));
        }

        if let Some(mat) = SEMICOLON_RE.find(current_slice) {
            self.position += mat.end();
            return Some(Ok(Token::Semicolon));
        }

        // --- 2.2: Match Identifiers (which could also be Keywords) ---
        // Treating keywords like other identifiers.
        // First, finding the end of the token. Then, if it looks like an identifier,
        // check whether it matches any keywords."
        if let Some(mat) = IDENTIFIER_RE.find(current_slice) {
            let val = mat.as_str(); // Get the matched string slice (e.g., "main", "myVar").
            self.position += mat.end(); // Advance position.

            // Now, check if this identifier is one of the predefined keywords.
            // Iterate over the `KEYWORDS` array (defined earlier).
            for (keyword_str, token_variant) in KEYWORDS.iter() {
                // Compare the matched identifier string (`val`) with the keyword string (`keyword_str`).
                // `*keyword_str` dereferences `&str` to `str` for comparison with `val` (which is `str`).
                if *keyword_str == val {
                    // If it's a keyword, return the corresponding keyword `Token` variant.
                    // `token_variant.clone()` is used because `token_variant` is a reference
                    // from the `KEYWORDS` array, and we need an owned `Token` value.
                    // (As `Token` derives `Clone`).
                    return Some(Ok(token_variant.clone()));
                }
            }
            // If the matched string is not found in the `KEYWORDS` array,
            // then it's a regular user-defined identifier.
            // `val.to_string()` converts the `&str` slice into an owned `String`
            // to be stored in the `Token::Identifier` variant.
            return Some(Ok(Token::Identifier(val.to_string())));
        }

        // --- 2.3: Match Integer Constants ---
        if let Some(mat) = CONSTANT_RE.find(current_slice) {
            let val_str = mat.as_str(); // Get the matched string of digits (e.g., "123").
            self.position += mat.end(); // Advance position.

            // Attempt to parse the matched string of digits into an `i32` integer.
            // `value_str.parse::<i32>()` returns a `Result<i32, ParseIntError>`.
            match val_str.parse::<i32>() {
                // If parsing is successful (`Ok(val)`), return a `Token::Constant`.
                Ok(val) => return Some(Ok(Token::Constant(val))),
                // If parsing fails (e.g., the number is too large to fit in an `i32`),
                // it's an error.
                Err(_) => {
                    // Return an `InvalidInteger` lexer error.
                    // Store the original string value and its starting position.
                    return Some(Err(LexerError::InvalidInteger {
                        value: val_str.to_string(),
                        pos: start_position_of_the_token,
                    }));
                }
            }
        }

        // --- Phase 3: Handle Unrecognized Input (Error Reporting) ---
        // If execution reaches this point, it means that after skipping whitespace/comments,
        // the `current_slice` did not match any of the defined token regexes (punctuation,
        // identifier, constant).

        // The `IDENTIFIER_RE` and `CONSTANT_RE` use `\b` (word boundary).
        // - "123bar": `CONSTANT_RE` won't match "123" because `b` is not a boundary.
        //             `IDENTIFIER_RE` won't match because it starts with '1'.
        //             So, "123bar" will fall through to this error handling section.
        //             `current_slice` will be "123bar...".
        // - "$": This character doesn't start any known token. `current_slice` will be "$...".

        // Check if `current_slice` is not empty. (It shouldn't be if we passed the EOF check earlier).
        if !current_slice.is_empty() {
            // Try to get the first character of the problematic slice.
            // `chars().next()` correctly handles multi-byte UTF-8 characters.
            if let Some(first_char) = current_slice.chars().next() {
                // An unexpected character was found.
                // Return an `UnexpectedCharacter` error, providing the character and its position.
                // NOTE: We are NOT advancing `self.position` here. If `tokenize_all` stops on
                // the first error (which it does), the lexer stops at the exact error point.
                // If error recovery was implemented, we might advance `self.position` here
                // by `first_char.len_utf8()` to try and continue lexing.

                // self.position += first_char.len_utf8();
                return Some(Err(LexerError::UnexpectedCharacter {
                    char: first_char,
                    pos: start_position_of_the_token,
                }));
            }
        }

        // If `current_slice` was somehow empty here (which is unlikely given prior checks but
        // acts as a defensive fallback) or `chars().next()` returned `None` on a non-empty
        // slice (even more unlikely for valid UTF-8), then report a general `NoMatch` error.
        // This signifies that the lexer is "stuck" but cannot pinpoint a specific character.
        Some(Err(LexerError::NoMatch {
            pos: start_position_of_the_token,
        }))
    }

    // `tokenize_all` is the primary public method for using the lexer.
    // It consumes the entire input string (or up to the first error) and
    // returns a vector of all recognized tokens.
    // - `&mut self`: Takes a mutable reference because `next_token_internal` modifies `self.position`.
    // - `-> Result<Vec<Token>, LexerError>`:
    //   - `Ok(Vec<Token>)`: If lexing is successful for the entire input, returns a vector
    //     containing all the tokens in order.
    //   - `Err(LexerError)`: If any lexing error occurs, it stops immediately and returns
    //     the first error encountered.
    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, LexerError> {
        // `tokens`: Create an empty, mutable vector to store the recognized tokens.
        // `Vec::new()` is one way to create an empty vector.
        let mut tokens = Vec::new();
        // `while let Some(token_result) = self.next_token_internal()`:
        // This loop continues as long as `self.next_token_internal()` returns `Some(...)`.
        // When `next_token_internal` returns `None` (signifying end of input), the loop terminates.
        // `token_result` will be of type `Result<Token, LexerError>`.
        while let Some(token_result) = self.next_token_internal() {
            // `match token_result`: Pattern match on the `Result` returned by `next_token_internal`.
            match token_result {
                // If `token_result` is `Ok(token)`, it means a token was successfully recognized.
                Ok(token) => {
                    // Add the successfully recognized `token` to the `tokens` vector.
                    tokens.push(token);
                }
                // If `token_result` is `Err(e)`, it means a lexing error occurred.
                Err(e) => {
                    // If an error is encountered, stop tokenizing immediately and
                    // return the error. The `?` operator could also be used here if
                    // `next_token_internal` returned `Result<Option<Token>, LexerError>`,
                    // but the current structure requires an explicit return.
                    return Err(e);
                }
            }
        }
        // If the loop completes without returning an `Err`, it means the entire input
        // was processed successfully (or was empty).
        // Return the vector of collected tokens wrapped in `Ok`.
        Ok(tokens)
    }
} // End of `impl<'a> Lexer<'a>` block
