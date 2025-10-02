# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **`DynamoDbStore` struct** - New primary API for interacting with DynamoDB
  - Reuses AWS client across operations for massive performance improvement
  - Thread-safe and Clone-able
  - Three constructors: `new()`, `from_config()`, `from_client()`
- Custom error types using `thiserror` for better error handling
  - `Error` enum with `AwsSdk` and `Validation` variants
  - `Result<T>` type alias for convenience
- Input validation for table names and items/keys
  - Validates table names are not empty
  - Validates items and keys are not empty
- Comprehensive rustdoc documentation for all public APIs
- LICENSE file (MIT)
- CHANGELOG.md following Keep a Changelog format
- `thiserror` dependency for ergonomic error handling

### Changed
- **BREAKING**: Error type changed from `aws_sdk_dynamodb::Error` to `clean_dynamodb_store::Error`
- **BREAKING**: Return type of `put_item` and `delete_item` now uses custom `Result` type
- Updated to Rust 2024 edition
- Updated aws-config from 1.1.9 to 1.8.6
- Updated aws-sdk-dynamodb from 1.20.0 to 1.93.0
- Enhanced Cargo.toml metadata with repository, homepage, categories, and documentation URLs
- Refactored `put_item` and `delete_item` functions to use `DynamoDbStore` internally
- Updated documentation to recommend `DynamoDbStore` for better performance

### Performance
- 🚀 **Significant performance improvement** by reusing AWS DynamoDB client
- Follows AWS SDK best practices for client usage
- Eliminates per-operation client creation overhead

## [0.0.2] - 2025-10-02

### Changed
- Fix keywords length in Cargo.toml
- Update README documentation
- Add missing cargo metadata

## [0.0.1] - Initial Release

### Added
- `put_item` function for inserting/updating items in DynamoDB tables
- `delete_item` function for deleting items from DynamoDB tables
- Basic async API using tokio
- AWS SDK integration with credential loading from environment

[Unreleased]: https://github.com/vvivan/clean_dynamodb_store/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/vvivan/clean_dynamodb_store/releases/tag/v0.0.2
[0.0.1]: https://github.com/vvivan/clean_dynamodb_store/releases/tag/v0.0.1
