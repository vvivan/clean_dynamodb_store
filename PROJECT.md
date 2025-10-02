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

### Versioning
This project follows [Semantic Versioning](https://semver.org/) (SemVer):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): New functionality in a backwards-compatible manner
- **PATCH** version (0.0.X): Backwards-compatible bug fixes

Pre-1.0.0 versions (0.x.x) may introduce breaking changes in minor versions.

## Release Checklist

Before publishing a new version to crates.io, ensure all items are completed:

### Pre-Release Verification
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] Documentation builds without warnings: `cargo doc --no-deps`
- [ ] Run `cargo publish --dry-run` to verify package contents
- [ ] Check `.crate` file size is under 10MB limit

### Documentation & Metadata
- [ ] Update version number in `Cargo.toml` following SemVer
- [ ] Update `CHANGELOG.md` with all changes for this version
- [ ] Ensure all public APIs have rustdoc comments with examples
- [ ] Verify `README.md` is up to date
- [ ] Confirm `Cargo.toml` metadata is accurate (description, keywords, categories, repository, homepage)

### Security & Dependencies
- [ ] Run `cargo audit` to check for vulnerable dependencies
- [ ] Review and update dependencies if needed
- [ ] Ensure no sensitive information in code or commits

### Quality Assurance
- [ ] All planned features for this version are implemented
- [ ] No known critical bugs
- [ ] Breaking changes are documented in CHANGELOG
- [ ] Migration guide provided for breaking changes (if applicable)

### Publishing
- [ ] Create git tag for version: `git tag v0.X.X`
- [ ] Push tag to GitHub: `git push origin v0.X.X`
- [ ] Publish to crates.io: `cargo publish`
- [ ] Create GitHub release with changelog notes
- [ ] Announce release (if significant)

### Post-Release
- [ ] Verify package on crates.io
- [ ] Check docs.rs built successfully
- [ ] Update version to next development version (optional)
- [ ] Close related GitHub issues/milestones

## Publishing Guidelines

### First-Time Publishing Setup
1. Create account on [crates.io](https://crates.io) (requires GitHub login)
2. Verify your email address in Account Settings
3. Generate API token from crates.io
4. Run `cargo login` with your token

### Publishing Process
```bash
# 1. Verify everything is ready
cargo publish --dry-run

# 2. Review the package contents
cargo package --list

# 3. Check package size
ls -lh target/package/*.crate

# 4. Publish to crates.io
cargo publish
```

### Important Notes
- **Publishing is permanent** - versions cannot be overwritten or deleted
- Use `cargo yank --vers X.X.X` to prevent new dependencies on a broken version
- Yanked versions can still be used by existing projects
- Maximum package size: 10MB
- Consider using `cargo-release` tool for automated releasing

### Trusted Publishing (2025+)
Configure GitHub Actions to publish without API tokens using OpenID Connect (OIDC):
- Set up trusted publishing on crates.io
- Configure GitHub repository permissions
- Use official publish action in CI/CD

## Questions to Resolve

- Should we provide a `Store` struct that holds the client, or keep functional API?
- How to balance simplicity vs. flexibility in the API design?
- Should we abstract away `AttributeValue` or expose it directly?
- What level of opinionated defaults should we provide?

## Notes

- Library targets Rust 2024 edition
- Uses AWS SDK v1.x (latest stable)
- MIT licensed
- Follows Rust API Guidelines
- All releases documented in CHANGELOG.md
