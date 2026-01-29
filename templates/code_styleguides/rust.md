# Rust Code Style Guide

This document outlines the coding standards and best practices for Rust development in this project.

## General Principles
- **Safety First**: Leverage Rust's ownership and borrowing system to ensure memory safety. Avoid `unsafe` unless absolutely necessary and well-documented.
- **Idiomatic Rust**: Follow the patterns described in "The Rust Programming Language" and "Rust API Guidelines".
- **Performance**: Utilize zero-cost abstractions and efficient data structures.

## Naming Conventions
- **Modules/Crates**: `snake_case`
- **Types (Structs, Enums, Traits, Type Aliases)**: `PascalCase`
- **Functions/Methods/Variables**: `snake_case`
- **Constants/Statics**: `SCREAMING_SNAKE_CASE`
- **Type Parameters**: `UpperCamelCase` (usually a single letter like `T`)

## Formatting
- Use `rustfmt` for all formatting.
- Line length limit: 100 characters.
- Use 4 spaces for indentation.

## Documentation
- Use `///` for doc comments on public items.
- Use `//!` for module-level documentation.
- Include a `# Examples` section for complex functions.
- Document error conditions in a `# Errors` section.

## Error Handling
- Use `Result<T, E>` for recoverable errors.
- Use `panic!` only for unrecoverable errors or in tests/examples.
- Prefer the `?` operator for error propagation.
- Use `thiserror` for defining custom error types in libraries and `anyhow` for applications.

## Testing
- Place unit tests in a `tests` module at the bottom of the file with `#[cfg(test)]`.
- Place integration tests in the `tests/` directory.
- Use descriptive test names.
- Aim for high test coverage, especially for core logic.

## Tooling
- **Linter**: Use `clippy` and address all warnings.
- **Build System**: Use `cargo`.
- **Dependencies**: Keep dependencies minimal and up-to-date.
