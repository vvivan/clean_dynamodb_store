# AWS Clean DynamoDB Store

`clean_dynamodb_store` is a Rust library designed to follow clean architecture principles, offering a straightforward and efficient DynamoDB store implementation. It simplifies interactions with AWS DynamoDB, making it easier to perform common database operations such as inserting and deleting items in a DynamoDB table.

## Features

- Easy-to-use asynchronous API for DynamoDB.
- Supports basic DynamoDB operations like put (insert/update) and delete items.
- Built on top of `aws-sdk-dynamodb` for robust and up-to-date DynamoDB access.
- Designed with clean architecture principles in mind.

## Prerequisites

Before you begin, ensure you have met the following requirements:

- Rust 2021 edition or later.
- AWS account and configured AWS CLI or environment variables for AWS access.

## Installation

Add `clean_dynamodb_store` to your `Cargo.toml`:

```toml
[dependencies]
clean_dynamodb_store = "0.0.2"
```
## Usage

Putting an Item into a DynamoDB Table

```rust
use clean_dynamodb_store::put_item;
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_dynamodb::Error> {
    let table_name = "your_table_name";
    let mut item = HashMap::new();
    item.insert("id".to_string(), AttributeValue::S("example_id".to_string()));
    item.insert("content".to_string(), AttributeValue::S("Hello, world!".to_string()));

    put_item(table_name, item).await?;
    Ok(())
}
```

Deleting an Item from a DynamoDB Table

```rust
use clean_dynamodb_store::delete_item;
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_dynamodb::Error> {
    let table_name = "your_table_name";
    let mut key = HashMap::new();
    key.insert("id".to_string(), AttributeValue::S("example_id".to_string()));

    delete_item(table_name, key).await?;
    Ok(())
}
```

## License

Distributed under the MIT License. See LICENSE for more information.

## Contact

Ivan Videnovic - videnovici@yahoo.com

