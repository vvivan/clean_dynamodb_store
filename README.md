# AWS Clean DynamoDB Store

`clean_dynamodb_store` is a Rust library designed to follow clean architecture principles, offering a straightforward and efficient DynamoDB store implementation. It simplifies interactions with AWS DynamoDB, making it easier to perform common database operations such as inserting and deleting items in a DynamoDB table.

## Features

- **Type-safe API** - Work with your own Rust structs using serde
- **Efficient client reuse** - Following AWS SDK best practices
- **Optimized for AWS Lambda** - Minimal cold start overhead
- **Dual API** - High-level type-safe methods + low-level HashMap methods
- **Full serde support** - Flattening, enums, custom serialization
- **Input validation** - Table names and items/keys
- **Custom error types** - Better error handling with thiserror
- **Clean architecture** - Designed with SOLID principles in mind

## Prerequisites

Before you begin, ensure you have met the following requirements:

- Rust 2024 edition or later
- AWS account and configured AWS CLI or environment variables for AWS access

## Installation

Add `clean_dynamodb_store` to your `Cargo.toml`:

```toml
[dependencies]
clean_dynamodb_store = "0.0.2"
```
## Usage

Create a `DynamoDbStore` once and reuse it across operations for optimal performance.

### Type-Safe API (Recommended)

Work with your own structs using serde - no manual AttributeValue construction needed:

```rust
use clean_dynamodb_store::DynamoDbStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    age: u32,
}

#[derive(Serialize)]
struct UserKey {
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store once, reuse many times
    let store = DynamoDbStore::new().await?;

    // Put an item
    let user = User {
        id: "user123".to_string(),
        name: "John Doe".to_string(),
        age: 30,
    };
    store.put("users", &user).await?;

    // Get an item
    let key = UserKey { id: "user123".to_string() };
    let user: Option<User> = store.get("users", &key).await?;

    if let Some(user) = user {
        println!("Found user: {} (age {})", user.name, user.age);
    }

    // Delete an item
    store.delete("users", &key).await?;

    Ok(())
}
```

### Table-Scoped API (Repository Pattern)

For implementing the repository pattern or working extensively with specific tables, you can create table-bound stores:

```rust
use clean_dynamodb_store::DynamoDbStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct UserKey {
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Create table-bound stores - great for repository pattern
    let users = store.for_table("users");
    let orders = store.for_table("orders");

    // Use without passing table name on each call
    let user = User {
        id: "user123".to_string(),
        name: "John Doe".to_string(),
    };
    users.put(&user).await?;

    let key = UserKey { id: "user123".to_string() };
    let user: Option<User> = users.get(&key).await?;

    users.delete(&key).await?;

    Ok(())
}
```

**When to use table-scoped stores:**
- Implementing repository pattern (one repository per entity/table)
- Building domain models with clean architecture principles
- Working extensively with specific tables
- Want cleaner method signatures without table name repetition

### Low-Level API

For advanced use cases, you can work directly with DynamoDB's AttributeValue types:

```rust
use clean_dynamodb_store::DynamoDbStore;
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Put an item
    let mut item = HashMap::new();
    item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
    item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
    store.put_item("users", item).await?;

    // Delete an item
    let mut key = HashMap::new();
    key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
    store.delete_item("users", key).await?;

    Ok(())
}
```

## AWS Lambda Usage

For AWS Lambda functions, initialize the store in `main()` to reuse the client across warm invocations:

```rust
use clean_dynamodb_store::DynamoDbStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize once during cold start
    let store = DynamoDbStore::new().await?;

    // Pass to handler - reused across warm invocations
    lambda_runtime::run(service_fn(|event| handler(event, &store))).await
}

async fn handler(
    event: Event,
    store: &DynamoDbStore,
) -> Result<Response, Box<dyn std::error::Error>> {
    // Use store with type-safe API - no client creation overhead!
    let user = User {
        id: event.id,
        name: event.name,
    };
    store.put("users", &user).await?;

    Ok(Response::success())
}
```

## License

Distributed under the MIT License. See LICENSE for more information.

## Contact

Ivan Videnovic - videnovici@yahoo.com

