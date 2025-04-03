# Innosystem Rust Coding Conventions

This document outlines the coding conventions and guidelines for the Innosystem project. These standards ensure consistency, maintainability, and compatibility with the Rust 2024 edition.

## General Guidelines

1. **Edition**: All Rust packages should use edition 2024.
2. **Formatting**: Use `rustfmt` for consistent code formatting.
3. **Linting**: Run `clippy` regularly to catch common mistakes and anti-patterns.
4. **Documentation**: All public APIs should have appropriate documentation comments.
5. **Error Handling**: Prefer using the Result monad and avoid panics in production code.

## Type Safety

1. **Explicit Types**: Prefer explicit type annotations for function parameters and returns.
2. **Option<T>**: Always use Option<T> for optional values rather than nullable types.
3. **Result<T, E>**: Use Result for operations that can fail, with appropriate error types.
4. **Type Conversion**: Prefer explicit conversions over implicit ones. Use `.into()`, `.try_into()`, or constructor methods.

## API Handlers (Axum)

1. **Debug Handler**: Always use `#[axum::debug_handler]` attribute on handler functions to get better error messages.
2. **Return Types**: Handler functions should return `Result<impl IntoResponse, StatusCode>` for consistent error handling.
3. **Authorization Headers**: When extracting tokens, clone the value instead of borrowing to avoid lifetime issues:
   ```rust
   let token = match auth_header {
       Some(auth) => auth.token().to_string(), // Important: Clone to avoid lifetime issues
       None => return Err(StatusCode::UNAUTHORIZED),
   };
   ```
4. **Type Conversions in Responses**: Always handle type conversions explicitly when constructing response objects:
   - For Option<String> to String: Use `.unwrap_or_default()`
   - For String to Option<String>: Use `Some(value)`
   - For enum string representation: Use `format!("{:?}", enum_value)` or implement proper Display traits

## Model Guidelines

1. **Field Consistency**: Ensure model field types match between request/response objects and database models.
2. **Enums**: Always implement Display, Debug, and FromStr for enums used in APIs.
3. **Serialization**: All API models should derive Serialize and Deserialize.
4. **Validation**: Perform validation at the API boundary before passing data to repositories.

## Error Handling

1. **Error Types**: Use appropriate error types:
   - Use StatusCode for web API errors
   - Use thiserror for defining custom error types
   - Use anyhow for application errors without custom types
2. **Error Conversion**: Implement From<OtherError> trait for custom error types to enable the `?` operator.
3. **Error Messages**: Include descriptive error messages to aid debugging.

## Testing

1. **Unit Tests**: Every module should have associated unit tests.
2. **Integration Tests**: API endpoints should have integration tests.
3. **Mocking**: Use test doubles for external dependencies.
4. **Test Coverage**: Aim for high test coverage, especially for business logic.

## Performance Considerations

1. **Async/Await**: Use async/await for I/O-bound operations, but be mindful of the overhead.
2. **Memory Management**: Minimize cloning of large data structures.
3. **Pooling**: Use connection pooling for database and Redis connections.
4. **Batch Operations**: Prefer batch operations over multiple individual operations when possible.

## Security Guidelines

1. **Input Validation**: Always validate user input before processing.
2. **Authentication**: Properly validate API keys and tokens.
3. **Authorization**: Check permissions before performing sensitive operations.
4. **Sensitive Data**: Never log sensitive data like API keys or personal information.

This document is a living standard and will be updated as the project evolves.
