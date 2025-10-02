use aws_sdk_dynamodb::{
    Client, operation::delete_item::DeleteItemOutput, operation::put_item::PutItemOutput,
    types::AttributeValue,
};
use std::collections::HashMap;

use crate::error::{Error, Result};

/// A DynamoDB store that maintains a reusable client connection.
///
/// This struct follows AWS best practices by reusing the DynamoDB client across operations,
/// which significantly improves performance by avoiding the overhead of creating a new client
/// for each operation.
///
/// The client is thread-safe and can be cloned cheaply (shallow clone).
///
/// # Example
///
/// ```rust,no_run
/// use clean_dynamodb_store::DynamoDbStore;
/// use aws_sdk_dynamodb::types::AttributeValue;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create store once, reuse many times
///     let store = DynamoDbStore::new().await?;
///
///     // Put an item
///     let mut item = HashMap::new();
///     item.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
///     store.put_item("users", item).await?;
///
///     // Delete an item
///     let mut key = HashMap::new();
///     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
///     store.delete_item("users", key).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DynamoDbStore {
    client: Client,
}

impl DynamoDbStore {
    /// Creates a new DynamoDB store with the default AWS configuration.
    ///
    /// This loads AWS credentials and configuration from the environment
    /// (environment variables, AWS config files, or IAM roles).
    ///
    /// # Errors
    ///
    /// Returns an error if AWS configuration cannot be loaded.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_from_env().await;
        Ok(Self {
            client: Client::new(&config),
        })
    }

    /// Creates a new DynamoDB store from an existing AWS SDK config.
    ///
    /// Use this when you need custom configuration or want to share
    /// configuration across multiple AWS services.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_config;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = aws_config::load_from_env().await;
    ///     let store = DynamoDbStore::from_config(&config);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_config(config: &aws_config::SdkConfig) -> Self {
        Self {
            client: Client::new(config),
        }
    }

    /// Creates a new DynamoDB store from an existing DynamoDB client.
    ///
    /// Use this when you already have a configured client you want to reuse.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = aws_config::load_from_env().await;
    ///     let client = Client::new(&config);
    ///     let store = DynamoDbStore::from_client(client);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    /// Validates that a table name is not empty.
    fn validate_table_name(table_name: &str) -> Result<()> {
        if table_name.trim().is_empty() {
            return Err(Error::Validation("Table name cannot be empty".to_string()));
        }
        Ok(())
    }

    /// Validates that items or keys are not empty.
    fn validate_not_empty(map: &HashMap<String, AttributeValue>, field_name: &str) -> Result<()> {
        if map.is_empty() {
            return Err(Error::Validation(format!("{} cannot be empty", field_name)));
        }
        Ok(())
    }

    /// Inserts or updates an item in a DynamoDB table.
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
    ///     item.insert("name".to_string(), AttributeValue::S("John Doe".to_string()));
    ///     item.insert("age".to_string(), AttributeValue::N("30".to_string()));
    ///
    ///     store.put_item("users", item).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn put_item(
        &self,
        table_name: &str,
        item: HashMap<String, AttributeValue>,
    ) -> Result<PutItemOutput> {
        Self::validate_table_name(table_name)?;
        Self::validate_not_empty(&item, "Item")?;

        let result = self
            .client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        Ok(result)
    }

    /// Deletes an item from a DynamoDB table.
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
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     // For a table with partition key "id"
    ///     let mut key = HashMap::new();
    ///     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     store.delete_item("users", key).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with Sort Key
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
    ///     // For a table with partition key "user_id" and sort key "timestamp"
    ///     let mut key = HashMap::new();
    ///     key.insert("user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///     key.insert("timestamp".to_string(), AttributeValue::N("1640000000".to_string()));
    ///
    ///     store.delete_item("events", key).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete_item(
        &self,
        table_name: &str,
        key: HashMap<String, AttributeValue>,
    ) -> Result<DeleteItemOutput> {
        Self::validate_table_name(table_name)?;
        Self::validate_not_empty(&key, "Key")?;

        let result = self
            .client
            .delete_item()
            .table_name(table_name)
            .set_key(Some(key))
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        Ok(result)
    }
}
