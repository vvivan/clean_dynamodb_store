use aws_sdk_dynamodb::{operation::put_item::PutItemOutput, types::AttributeValue};
use std::collections::HashMap;

use crate::{DynamoDbStore, error::Result};

/// Inserts or updates an item in a DynamoDB table.
///
/// **Note**: This is a convenience function that creates a new client for each operation.
/// For better performance, use [`DynamoDbStore`] directly to reuse the client across operations.
///
/// # Arguments
///
/// * `table_name` - The name of the DynamoDB table where the item will be inserted or updated
/// * `item` - A HashMap containing the attribute names and values for the item
///
/// # Returns
///
/// Returns `Ok(PutItemOutput)` on success, containing the response from DynamoDB.
///
/// # Errors
///
/// Returns an error if:
/// - The table name is empty
/// - The item map is empty
/// - AWS credentials are not properly configured
/// - The specified table does not exist
/// - The item exceeds DynamoDB's size limits (400 KB)
/// - Network connectivity issues occur
/// - IAM permissions are insufficient
///
/// # Example
///
/// ```rust,no_run
/// use clean_dynamodb_store::put_item;
/// use aws_sdk_dynamodb::types::AttributeValue;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut item = HashMap::new();
///     item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
///     item.insert("age".to_string(), AttributeValue::N("30".to_string()));
///
///     put_item("users", item).await?;
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
///     let mut item = HashMap::new();
///     item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     store.put_item("users", item).await?;
///     Ok(())
/// }
/// ```
pub async fn put_item(
    table_name: &str,
    item: HashMap<String, AttributeValue>,
) -> Result<PutItemOutput> {
    let store = DynamoDbStore::new().await?;
    store.put_item(table_name, item).await
}
