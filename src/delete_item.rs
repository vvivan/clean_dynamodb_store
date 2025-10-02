use std::collections::HashMap;

use aws_sdk_dynamodb::{operation::delete_item::DeleteItemOutput, types::AttributeValue};

use crate::{DynamoDbStore, error::Result};

/// Deletes an item from a DynamoDB table.
///
/// **Note**: This is a convenience function that creates a new client for each operation.
/// For better performance, use [`DynamoDbStore`] directly to reuse the client across operations.
///
/// # Arguments
///
/// * `table_name` - The name of the DynamoDB table from which the item will be deleted
/// * `key` - A HashMap containing the primary key attributes that identify the item to delete.
///   Must include the partition key and sort key (if the table has one)
///
/// # Returns
///
/// Returns `Ok(DeleteItemOutput)` on success, containing the response from DynamoDB.
/// The operation succeeds even if the item doesn't exist in the table.
///
/// # Errors
///
/// Returns an error if:
/// - The table name is empty
/// - The key map is empty
/// - AWS credentials are not properly configured
/// - The specified table does not exist
/// - The key does not match the table's key schema
/// - Network connectivity issues occur
/// - IAM permissions are insufficient
///
/// # Example
///
/// ```rust,no_run
/// use clean_dynamodb_store::delete_item;
/// use aws_sdk_dynamodb::types::AttributeValue;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // For a table with partition key "id"
///     let mut key = HashMap::new();
///     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///
///     delete_item("users", key).await?;
///     Ok(())
/// }
/// ```
///
/// # Example with Sort Key
///
/// ```rust,no_run
/// use clean_dynamodb_store::delete_item;
/// use aws_sdk_dynamodb::types::AttributeValue;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // For a table with partition key "user_id" and sort key "timestamp"
///     let mut key = HashMap::new();
///     key.insert("user_id".to_string(), AttributeValue::S("user123".to_string()));
///     key.insert("timestamp".to_string(), AttributeValue::N("1640000000".to_string()));
///
///     delete_item("events", key).await?;
///     Ok(())
/// }
/// ```
///
/// # Performance Note
///
/// For better performance in applications with multiple operations, use [`DynamoDbStore`]:
///
/// ```rust,no_run
/// use clean_dynamodb_store::DynamoDbStore;
/// use aws_sdk_dynamodb::types::AttributeValue;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = DynamoDbStore::new().await?;
///
///     let mut key = HashMap::new();
///     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     store.delete_item("users", key).await?;
///     Ok(())
/// }
/// ```
pub async fn delete_item(
    table_name: &str,
    key: HashMap<String, AttributeValue>,
) -> Result<DeleteItemOutput> {
    let store = DynamoDbStore::new().await?;
    store.delete_item(table_name, key).await
}
