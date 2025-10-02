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
//! ## Recommended Usage (with `DynamoDbStore`)
//!
//! For best performance, create a [`DynamoDbStore`] once and reuse it across operations:
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
//! ## Convenience Functions
//!
//! For simple use cases, convenience functions are available that create a client per operation:
//!
//! ```rust,no_run
//! use clean_dynamodb_store::put_item;
//! use aws_sdk_dynamodb::types::AttributeValue;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut item = HashMap::new();
//!     item.insert("id".to_string(), AttributeValue::S("example_id".to_string()));
//!     item.insert("content".to_string(), AttributeValue::S("Hello, world!".to_string()));
//!
//!     put_item("your_table_name", item).await?;
//!     Ok(())
//! }
//! ```
//!
//! **Note**: For better performance with multiple operations, use [`DynamoDbStore`] instead.

pub mod delete_item;
pub mod error;
pub mod put_item;
pub mod store;

pub use delete_item::delete_item;
pub use error::{Error, Result};
pub use put_item::put_item;
pub use store::DynamoDbStore;
