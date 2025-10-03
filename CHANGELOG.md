# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Batch Write Operations** - Efficiently write large numbers of items
  - `batch_put<T>()` - Batch write with type-safe structs
  - `batch_put_items()` - Batch write with low-level HashMap API
  - `BatchWriteResult` - Detailed success/failure statistics
  - Automatic chunking into batches of 25 (DynamoDB's limit)
  - Exponential backoff retry for unprocessed items (up to 3 retries)
  - Available on both `DynamoDbStore` and `TableBoundStore`
  - Handles throttling and provides detailed error reporting
- **Table-Scoped API** - Repository pattern support with `TableBoundStore`
  - `DynamoDbStore::for_table(name)` - Create table-bound stores
  - `TableBoundStore` - Eliminates need to pass table name on every operation
  - Ideal for implementing repository pattern (one repository per entity/table)
  - Perfect for clean architecture and domain-driven design
  - All operations available: `put()`, `delete()`, `get()`, `put_item()`, `delete_item()`, `batch_put()`, `batch_put_items()`
- **Type-safe API** - High-level methods using serde for ergonomic DynamoDB operations
  - `put<T: Serialize>()` - Insert/update items using Rust structs
  - `delete<K: Serialize>()` - Delete items using key structs
  - `get<K: Serialize, T: DeserializeOwned>()` - Retrieve and deserialize items
  - Full serde support: flattening, enums, custom serialization
  - Automatic conversion between Rust types and DynamoDB AttributeValue format
- **`DynamoDbStore` struct** - Primary API for interacting with DynamoDB
  - Reuses AWS client across operations for massive performance improvement
  - Thread-safe and Clone-able
  - Three constructors: `new()`, `from_config()`, `from_client()`
  - Dual API: high-level type-safe methods + low-level HashMap methods
- Custom error types using `thiserror` for better error handling
  - `Error` enum with `AwsSdk` and `Validation` variants
  - `Result<T>` type alias for convenience
- Input validation for table names and items/keys
  - Validates table names are not empty
  - Validates items and keys are not empty
- Comprehensive rustdoc documentation for all public APIs
- LICENSE file (MIT)
- CHANGELOG.md following Keep a Changelog format
- Dependencies:
  - `serde` 1.0 - Serialization framework
  - `serde_dynamo` 4.x with aws-sdk-dynamodb+1 feature
  - `thiserror` 2.0 - Ergonomic error handling

### Changed
- **BREAKING**: Error type changed from `aws_sdk_dynamodb::Error` to `clean_dynamodb_store::Error`
- Updated to Rust 2024 edition
- Updated aws-config from 1.1.9 to 1.8.6
- Updated aws-sdk-dynamodb from 1.20.0 to 1.93.0
- Enhanced Cargo.toml metadata with repository, homepage, categories, and documentation URLs
- Updated documentation with AWS Lambda usage examples

### Removed
- **BREAKING**: Removed `put_item()` free function - use `DynamoDbStore::put_item()` instead
- **BREAKING**: Removed `delete_item()` free function - use `DynamoDbStore::delete_item()` instead

**Migration Guide:**
```rust
// Before (0.0.2):
put_item("table", item).await?;
delete_item("table", key).await?;

// After (0.1.0+):
let store = DynamoDbStore::new().await?;
store.put_item("table", item).await?;
store.delete_item("table", key).await?;
```

**Rationale**: Free functions created a new DynamoDB client for each operation, which is
terrible for performance in AWS Lambda and other long-running applications. The `DynamoDbStore`
API follows AWS SDK best practices by reusing the client, providing 10-100x better performance.

### Performance
- 🚀 **Significant performance improvement** by reusing AWS DynamoDB client
- Follows AWS SDK best practices for client usage
- Eliminates per-operation client creation overhead

## [0.0.2] - 2024-04-01

### Changed
- Fix keywords length in Cargo.toml
- Update README documentation
- Add missing cargo metadata

## [0.0.1] - 2024-04-01

### Added
- `put_item` function for inserting/updating items in DynamoDB tables
- `delete_item` function for deleting items from DynamoDB tables
- Basic async API using tokio
- AWS SDK integration with credential loading from environment

[Unreleased]: https://github.com/vvivan/clean_dynamodb_store/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/vvivan/clean_dynamodb_store/releases/tag/v0.0.2
[0.0.1]: https://github.com/vvivan/clean_dynamodb_store/releases/tag/v0.0.1
