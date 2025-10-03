//! # AWS Clean DynamoDB Store
//!
//! `clean_dynamodb_store` is a Rust library designed to follow clean architecture principles,
//! offering a straightforward and efficient DynamoDB store implementation. It simplifies
//! interactions with AWS DynamoDB, making it easier to perform common database operations
//! such as inserting and deleting items in a DynamoDB table.
//!
//! ## Features
//!
//! - Easy-to-use asynchronous API for DynamoDB
//! - Efficient client reuse following AWS SDK best practices
//! - Supports basic DynamoDB operations like put (insert/update) and delete items
//! - Input validation for table names and items/keys
//! - Custom error types for better error handling
//! - Built on top of `aws-sdk-dynamodb` for robust and up-to-date DynamoDB access
//! - Designed with clean architecture principles in mind
//!
//! ## Prerequisites
//!
//! Before you begin, ensure you have:
//!
//! - Rust 2024 edition or later
//! - AWS account and configured AWS CLI or environment variables for AWS access
//!
//! ## Usage
//!
//! Create a [`DynamoDbStore`] once and reuse it across operations for optimal performance:
//!
//! ```rust,no_run
//! use clean_dynamodb_store::DynamoDbStore;
//! use aws_sdk_dynamodb::types::AttributeValue;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create store once, reuse many times
//!     let store = DynamoDbStore::new().await?;
//!
//!     // Put an item
//!     let mut item = HashMap::new();
//!     item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
//!     item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
//!     store.put_item("users", item).await?;
//!
//!     // Delete an item
//!     let mut key = HashMap::new();
//!     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
//!     store.delete_item("users", key).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## AWS Lambda Usage
//!
//! For AWS Lambda functions, initialize the store in `main()` before the handler
//! to reuse the client across warm invocations:
//!
//! ```rust,no_run
//! use clean_dynamodb_store::DynamoDbStore;
//! use aws_sdk_dynamodb::types::AttributeValue;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize once during cold start
//!     let store = DynamoDbStore::new().await?;
//!
//!     // Pass to handler - reused across warm invocations
//!     // lambda_runtime::run(service_fn(|event| handler(event, &store))).await
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod store;

pub use error::{Error, Result};
pub use store::DynamoDbStore;
