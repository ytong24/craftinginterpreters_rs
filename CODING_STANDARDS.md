# Coding Standards

## Part 1: General Rust Style

### Type System

- Use precise, domain-appropriate types. `PathBuf` for paths, enums over booleans, newtypes where they add clarity. Never use `String` or `i32` when a more specific type exists.
- Match types to their consumers — e.g. `exit_code()` returns `i32` because `std::process::exit` takes `i32`. Don't use a narrower type that forces casts at every call site.
- Return types must accurately describe what can go wrong. `io::Result<()>` if only I/O can fail. Don't use `anyhow::Result` to erase type information.

### Data & Ownership

- **Borrow, don't allocate.** When data is a slice of an existing owned string, use `&'src str` instead of `String`. If the source lives long enough, downstream types should borrow from it, not clone. Prefer `&str` slices over `char` when the data naturally lives as part of a larger string — `char` is a copy that breaks the borrowing chain.
- **No redundant storage.** Don't carry data in enum variants when it's trivially derivable from another field. Only store data when it's a fundamentally different representation (e.g. `Number(f64)` vs the lexeme text).
- **Compute once, pass explicitly.** If a value is needed in multiple places, compute it once, store it in a local, and pass it where needed. Don't re-derive the same value by calling the same method again — even if cheap, it obscures data flow and creates a dependency that's easy to break silently.

### File & Code Organization

- `main` goes at the **bottom** of the file. Define building blocks first (types, low-level helpers), then higher-level functions, then `main` last.
- Imports: std first, then external crates, then `crate::` imports, each group separated by a blank line.

### Error Handling

- **Only `main` controls process exit.** Helper functions propagate errors upward; they never call `std::process::exit`.
- **Functions should have a single, clear error type.** If a function's return type would need to unify unrelated error kinds, that's a sign the function is doing too much.
- **Don't invent wrapper types just to propagate errors.** If a function does two things that fail differently, consider whether the function should exist at all.

### Dependencies

- Don't add dependencies unless they earn their place. Prefer manual impls when they give more control (e.g. manual `Display` over `thiserror` when exact format strings matter).
- If a dependency is no longer used, remove it immediately.

### Visibility

- **Default to minimal visibility.** Think carefully about every `pub` — limit scope to avoid unnecessary information leakage.
- **But don't add boilerplate for its own sake.** If a struct is a plain data type with no invariants to protect, `pub` fields are better than private fields with getter/setter/constructor boilerplate. The justification must be explicit: no invariants, no encapsulation benefit.
- In a binary crate, `pub` and `pub(crate)` are functionally identical — prefer `pub` to avoid visual noise when there are no external consumers.

### Comments

- **Explain the *why* and the *risk*, not the *what*.** Don't describe what the code does — describe why it's written this way and what would go wrong otherwise.
- **Use concrete examples with specific values.** "Underflows for multi-line strings" is vague. `"ab\ncd" — self.column is 3 but the token spans 8 bytes, so 3 - 8 overflows` is convincing.
- **Justify non-obvious safety assumptions.** When code relies on an invariant (like `peek()` always returning a single UTF-8 char or `""`), explain why the assumption holds and why it can't be violated.
- **Don't delete existing useful comments** when adding new ones nearby.
- **Don't over-describe.** If the code is self-evident, no comment needed. Only comment where the logic isn't obvious or where a future reader might question a design choice.
- **Document design choices in code.** When a decision was made between alternatives (e.g., dedicated enums vs reusing an existing type), the rationale belongs as a comment at the definition site.

### General

- No over-engineering. Don't create abstractions for one-time operations. Three similar lines are better than a premature helper.
- Code should be precise, not clever. Favor clarity and correctness.
- **Prefer idiomatic Rust.** Use Rust's own idioms (`matches!`, `if let`, `?` operator, iterator methods) over patterns ported from other languages. When the language provides a built-in way, use it.
- **Correct by construction.** At pipeline boundaries (scanner → parser → interpreter), the producer validates and encodes invariants in the types so consumers can trust the structure. The AST is a semantic representation, not a syntactic reshuffling — e.g., operators use dedicated enums (not raw tokens), literals store parsed values, grouping drops parentheses.

## Part 2: Project-Specific (Lox Interpreter)

### Error Types

- `Result<(), LoxError>` for functions that can produce Lox compile or runtime errors.
- Compile errors collect multiple issues before halting (exit 65); runtime errors halt immediately (exit 70).
- `Display` impls must match the test suite's expected stderr formats exactly — this is why we use manual impls over `thiserror`.

### Token Design

- `TokenKind` is a flat enum. Only `Number(f64)` carries a payload — `Identifier` and `Str` derive their values from the `Token::lexeme` field.
- `Token` borrows from the source string (`&'src str`) — zero allocations during scanning.

### Architecture

- `main` reads the file and calls `run` directly — no `run_file` wrapper. Two unrelated error kinds (I/O and Lox) are handled naturally in `main` without a unifying type.
- REPL mode: Lox errors are printed inline and the loop continues; only I/O errors propagate.
