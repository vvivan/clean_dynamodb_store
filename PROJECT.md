# Project Requirements and Roadmap

## Project Vision

`clean_dynamodb_store` is a Rust library that provides a clean, simple abstraction over AWS DynamoDB operations, following clean architecture principles. The goal is to make DynamoDB interactions straightforward while maintaining flexibility and type safety.

## Current State (v0.0.2)

### Implemented Features
- ✅ `put_item` - Insert or update items in a DynamoDB table
- ✅ `delete_item` - Delete items from a DynamoDB table
- ✅ Basic async API using `tokio`
- ✅ AWS SDK integration with credential loading from environment

### In Progress
- 🚧 `bulk_put_item` - Write multiple items in a single operation (using batch_write_item)

### Current Limitations
- Client is instantiated per operation (no connection pooling)
- No tests
- Limited error handling and error types
- Only 2 basic operations supported
- No batch operations
- No query or scan capabilities

## Main Flow Requirements

### Core Operations Flow
1. User calls library function with table name and data
2. Library loads AWS config from environment
3. Library creates DynamoDB client
4. Library executes operation via AWS SDK
5. Library returns result or error to user

### AWS Integration Requirements
- Must support standard AWS credential chain (env vars, config files, IAM roles)
- Must work with any AWS region configured by user
- Must handle AWS SDK errors gracefully

## Planned Updates

### Phase 1: Complete CRUD Operations
- [ ] Add `get_item` - Retrieve a single item by primary key
- [ ] Add `update_item` - Update specific attributes of an item
- [ ] Add `query` - Query items using partition key and optional sort key
- [ ] Add `scan` - Scan entire table with optional filters

### Phase 2: Performance & Architecture Improvements
- [ ] Implement client reuse pattern (avoid creating new client per operation)
- [ ] Add configuration options for client behavior
- [ ] Consider builder pattern for complex operations
- [ ] Add connection pooling support

### Phase 3: Batch Operations
- [ ] Add `bulk_put_item` - Write multiple items in a single request (using batch_write_item) **[IN PROGRESS]**
- [ ] Add `batch_write_item` - Write or delete multiple items in a single request
- [ ] Add `batch_get_item` - Retrieve multiple items in a single request
- [ ] Handle batch operation limitations (25 items max per request)

### Phase 4: Error Handling & Types
- [ ] Create custom error types wrapping AWS SDK errors
- [ ] Add detailed error messages for common failures
- [ ] Implement retry logic for throttled requests
- [ ] Add validation for required fields

### Phase 5: Testing & Quality
- [ ] Add unit tests with mocked DynamoDB client
- [ ] Add integration tests (require local DynamoDB or LocalStack)
- [ ] Add example code demonstrating common use cases
- [ ] Add benchmarks for performance testing

### Phase 6: Advanced Features
- [ ] Support for DynamoDB Streams
- [ ] Transaction support (`transact_write_items`, `transact_get_items`)
- [ ] Conditional operations (conditional writes, optimistic locking)
- [ ] Support for Global Secondary Indexes (GSI) and Local Secondary Indexes (LSI)
- [ ] Pagination support for query and scan operations
- [ ] Expression builder for filter/condition expressions

## Design Principles

1. **Simplicity First**: API should be intuitive for common use cases
2. **Clean Architecture**: Maintain separation of concerns and testability
3. **Type Safety**: Leverage Rust's type system for compile-time guarantees
4. **Async-First**: All operations are async using `tokio`
5. **Minimal Dependencies**: Only include necessary dependencies
6. **AWS SDK Compatibility**: Stay compatible with official AWS SDK patterns

## Development Guidelines

### Commit Messages
This project follows [Conventional Commits](https://www.conventionalcommits.org/) guidelines:

- **feat**: New features (e.g., `feat: add bulk_put_item function`)
- **fix**: Bug fixes (e.g., `fix: handle empty item list in bulk operations`)
- **chore**: Maintenance tasks (e.g., `chore: update dependencies`)
- **docs**: Documentation changes (e.g., `docs: add usage examples`)
- **refactor**: Code refactoring (e.g., `refactor: extract client creation logic`)
- **test**: Adding or updating tests (e.g., `test: add unit tests for put_item`)
- **perf**: Performance improvements (e.g., `perf: implement client connection pooling`)

## Questions to Resolve

- Should we provide a `Store` struct that holds the client, or keep functional API?
- How to balance simplicity vs. flexibility in the API design?
- Should we abstract away `AttributeValue` or expose it directly?
- What level of opinionated defaults should we provide?

## Notes

- Library targets Rust 2021 edition
- Uses AWS SDK v1.x (latest stable)
- MIT licensed
