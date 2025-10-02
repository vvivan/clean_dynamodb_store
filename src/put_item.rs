use aws_sdk_dynamodb::{operation::put_item::PutItemOutput, types::AttributeValue};
use std::collections::HashMap;

/// Inserts or updates an item in a DynamoDB table.
///
/// This function creates a new DynamoDB client for each operation and uses AWS credentials
/// from the environment (via environment variables, AWS config files, or IAM roles).
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
/// async fn main() -> Result<(), aws_sdk_dynamodb::Error> {
///     let mut item = HashMap::new();
///     item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
///     item.insert("age".to_string(), AttributeValue::N("30".to_string()));
///
///     put_item("users", item).await?;
///     Ok(())
/// }
/// ```
pub async fn put_item(
    table_name: &str,
    item: HashMap<String, AttributeValue>,
) -> Result<PutItemOutput, aws_sdk_dynamodb::Error> {
    let config = aws_config::load_from_env().await;

    let result = aws_sdk_dynamodb::Client::new(&config)
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;

    Ok(result)
}
