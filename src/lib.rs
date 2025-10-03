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
//! Create a [`DynamoDbStore`] once and reuse it across operations for optimal performance.
//!
//! ### Type-Safe API (Recommended)
//!
//! The type-safe API works with your own structs using serde:
//!
//! ```rust,no_run
//! use clean_dynamodb_store::DynamoDbStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: String,
//!     name: String,
//!     age: u32,
//! }
//!
//! #[derive(Serialize)]
//! struct UserKey {
//!     id: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create store once, reuse many times
//!     let store = DynamoDbStore::new().await?;
//!
//!     // Put an item using a struct
//!     let user = User {
//!         id: "user123".to_string(),
//!         name: "John Doe".to_string(),
//!         age: 30,
//!     };
//!     store.put("users", &user).await?;
//!
//!     // Get an item
//!     let key = UserKey { id: "user123".to_string() };
//!     let user: Option<User> = store.get("users", &key).await?;
//!
//!     // Delete an item
//!     store.delete("users", &key).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Low-Level API
//!
//! For advanced use cases, you can use the low-level HashMap API:
//!
//! ```rust,no_run
//! use clean_dynamodb_store::DynamoDbStore;
//! use aws_sdk_dynamodb::types::AttributeValue;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: String,
//!     name: String,
//! }
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
//!
//! // async fn handler(event: Event, store: &DynamoDbStore) -> Result<Response, Error> {
//! //     let user = User { id: event.id, name: event.name };
//! //     store.put("users", &user).await?;
//! //     Ok(Response::success())
//! // }
//! ```

pub mod error;
pub mod store;

pub use error::{Error, Result};
pub use store::DynamoDbStore;
