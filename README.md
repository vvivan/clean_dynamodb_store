# AWS Clean DynamoDB Store

`clean_dynamodb_store` is a Rust library designed to follow clean architecture principles, offering a straightforward and efficient DynamoDB store implementation. It simplifies interactions with AWS DynamoDB, making it easier to perform common database operations such as inserting and deleting items in a DynamoDB table.

## Features

- **Complete CRUD Operations** - Put, Get, Delete, Update with type-safe and low-level APIs
- **Advanced Querying** - Query and Scan operations with filter expressions
- **Batch Operations** - Efficient batch reads and writes with automatic chunking
- **Type-safe API** - Work with your own Rust structs using serde
- **Efficient client reuse** - Following AWS SDK best practices
- **Optimized for AWS Lambda** - Minimal cold start overhead
- **Dual API** - High-level type-safe methods + low-level HashMap methods
- **Full serde support** - Flattening, enums, custom serialization
- **Update Expressions** - Partial updates with SET, ADD, REMOVE, DELETE
- **Pagination Support** - Query and Scan with `last_evaluated_key`
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

### Update Operations

For partial item updates without replacing the entire item:

```rust
use clean_dynamodb_store::DynamoDbStore;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct UserKey {
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    let key = UserKey { id: "user123".into() };

    // Update specific attributes using update expressions
    let update_expression = "SET age = :age, #n = :name".to_string();

    let mut values = HashMap::new();
    values.insert(":age".to_string(), AttributeValue::N("31".to_string()));
    values.insert(":name".to_string(), AttributeValue::S("John Updated".to_string()));

    let mut names = HashMap::new();
    names.insert("#n".to_string(), "name".to_string()); // 'name' is a reserved keyword

    store.update("users", &key, update_expression, Some(values), Some(names)).await?;

    Ok(())
}
```

**Update expression actions:**
- `SET` - Add or update attributes
- `REMOVE` - Delete attributes
- `ADD` - Increment numbers or add to sets
- `DELETE` - Remove from sets

### Query Operations

Efficiently retrieve items by partition key (and optional sort key):

```rust
use clean_dynamodb_store::DynamoDbStore;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Order {
    user_id: String,
    order_id: String,
    total: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Query all orders for a specific user
    let key_condition = "user_id = :user_id".to_string();

    let mut values = HashMap::new();
    values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));

    let result = store.query::<Order>("orders", key_condition, values, None).await?;

    println!("Found {} orders", result.count);
    for order in result.items {
        println!("Order {}: ${}", order.order_id, order.total);
    }

    // Handle pagination if needed
    if let Some(last_key) = result.last_evaluated_key {
        // Use last_key for next query
    }

    Ok(())
}
```

### Scan Operations

Scan entire table (use sparingly, prefer Query when possible):

```rust
use clean_dynamodb_store::DynamoDbStore;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct User {
    id: String,
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Scan with filter
    let filter = Some("age > :min_age".to_string());

    let mut values = HashMap::new();
    values.insert(":min_age".to_string(), AttributeValue::N("18".to_string()));

    let result = store.scan::<User>("users", filter, Some(values), None).await?;

    println!("Found {} users (scanned {})", result.count, result.scanned_count);

    Ok(())
}
```

### Batch Operations

Efficiently write or read large numbers of items using batch operations. The library automatically handles chunking and retries with exponential backoff.

#### Batch Write

For writing large numbers of items, batch operations chunk into groups of 25 (DynamoDB's BatchWriteItem limit):

```rust
use clean_dynamodb_store::DynamoDbStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Create 1000 users
    let users: Vec<User> = (0..1000)
        .map(|i| User {
            id: format!("user{}", i),
            name: format!("User {}", i),
            age: 20 + (i % 50),
        })
        .collect();

    // Batch write - automatically chunks into groups of 25 and retries failures
    let result = store.batch_put("users", &users).await?;

    println!("Successfully wrote {} items", result.successful);
    if result.failed > 0 {
        println!("Failed to write {} items", result.failed);
        for failed in &result.failed_items {
            println!("  Error: {}", failed.error);
        }
    }

    Ok(())
}
```

#### Batch Get

For retrieving large numbers of items, batch operations chunk into groups of 100 (DynamoDB's BatchGetItem limit):

```rust
use clean_dynamodb_store::DynamoDbStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct UserKey {
    id: String,
}

#[derive(Deserialize)]
struct User {
    id: String,
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = DynamoDbStore::new().await?;

    // Create 250 keys to retrieve
    let keys: Vec<UserKey> = (0..250)
        .map(|i| UserKey {
            id: format!("user{}", i),
        })
        .collect();

    // Batch get - automatically chunks into groups of 100 and retries failures
    let result = store.batch_get::<UserKey, User>("users", &keys).await?;

    println!("Successfully retrieved {} items", result.successful);
    for user in &result.items {
        println!("User: {} (age {})", user.name, user.age);
    }

    if result.failed > 0 {
        println!("Failed to retrieve {} keys", result.failed);
    }

    Ok(())
}
```

**Batch operations features:**
- Automatic chunking (25 items for write, 100 for get)
- Exponential backoff retry for throttled requests (up to 3 retries)
- Detailed success/failure reporting
- Works with both type-safe API and table-scoped stores

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

