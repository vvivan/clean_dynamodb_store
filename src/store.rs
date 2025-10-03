use aws_sdk_dynamodb::{
    Client,
    operation::delete_item::DeleteItemOutput,
    operation::put_item::PutItemOutput,
    types::{AttributeValue, PutRequest, WriteRequest},
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};

/// Result of a batch write operation.
///
/// Contains information about successful and failed items after all retry attempts.
#[derive(Debug, Clone)]
pub struct BatchWriteResult {
    /// Number of successfully written items
    pub successful: usize,
    /// Number of failed items after all retries
    pub failed: usize,
    /// Items that permanently failed with error details
    pub failed_items: Vec<FailedItem>,
}

/// Information about an item that failed to write after all retry attempts.
#[derive(Debug, Clone)]
pub struct FailedItem {
    /// The item that failed (in DynamoDB's AttributeValue format)
    pub item: HashMap<String, AttributeValue>,
    /// Error message describing why it failed
    pub error: String,
}

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

    /// Inserts or updates an item using a type-safe struct.
    ///
    /// This is a higher-level alternative to [`put_item`](Self::put_item) that works with
    /// any type implementing [`Serialize`]. The struct is automatically converted to
    /// DynamoDB's AttributeValue format using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`Serialize`]
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `item` - A reference to the item to insert or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(PutItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - Serialization fails (invalid struct for DynamoDB)
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
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let user = User {
    ///         id: "user123".to_string(),
    ///         name: "John Doe".to_string(),
    ///         age: 30,
    ///     };
    ///
    ///     store.put("users", &user).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn put<T: Serialize>(
        &self,
        table_name: &str,
        item: &T,
    ) -> Result<PutItemOutput> {
        Self::validate_table_name(table_name)?;

        let item_map = serde_dynamo::to_item(item)
            .map_err(|e| Error::Validation(format!("Failed to serialize item: {}", e)))?;

        self.put_item(table_name, item_map).await
    }

    /// Deletes an item using a type-safe key struct.
    ///
    /// This is a higher-level alternative to [`delete_item`](Self::delete_item) that works with
    /// any type implementing [`Serialize`]. The key struct is automatically converted to
    /// DynamoDB's AttributeValue format using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `key` - A reference to the key struct identifying the item to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(DeleteItemOutput)` on success. The operation succeeds even if the item doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - Serialization fails (invalid key struct for DynamoDB)
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
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct UserKey {
    ///     id: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let key = UserKey {
    ///         id: "user123".to_string(),
    ///     };
    ///
    ///     store.delete("users", &key).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete<K: Serialize>(
        &self,
        table_name: &str,
        key: &K,
    ) -> Result<DeleteItemOutput> {
        Self::validate_table_name(table_name)?;

        let key_map = serde_dynamo::to_item(key)
            .map_err(|e| Error::Validation(format!("Failed to serialize key: {}", e)))?;

        self.delete_item(table_name, key_map).await
    }

    /// Retrieves an item from DynamoDB and deserializes it into a type-safe struct.
    ///
    /// This is a high-level method that retrieves an item using a key struct and
    /// automatically deserializes the result into the requested type using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `key` - A reference to the key struct identifying the item to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(T))` if the item exists and was successfully deserialized.
    /// Returns `Ok(None)` if the item does not exist in the table.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - Key serialization fails
    /// - Item deserialization fails (data doesn't match expected type)
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize)]
    /// struct UserKey {
    ///     id: String,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let key = UserKey {
    ///         id: "user123".to_string(),
    ///     };
    ///
    ///     match store.get::<UserKey, User>("users", &key).await? {
    ///         Some(user) => println!("Found user: {}", user.name),
    ///         None => println!("User not found"),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get<K: Serialize, T: DeserializeOwned>(
        &self,
        table_name: &str,
        key: &K,
    ) -> Result<Option<T>> {
        Self::validate_table_name(table_name)?;

        let key_map = serde_dynamo::to_item(key)
            .map_err(|e| Error::Validation(format!("Failed to serialize key: {}", e)))?;

        let result = self
            .client
            .get_item()
            .table_name(table_name)
            .set_key(Some(key_map))
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        match result.item {
            Some(item) => {
                let deserialized = serde_dynamo::from_item(item)
                    .map_err(|e| Error::Validation(format!("Failed to deserialize item: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Batch writes items to DynamoDB using the low-level HashMap API.
    ///
    /// This method automatically handles:
    /// - Chunking items into batches of 25 (DynamoDB's limit)
    /// - Retrying unprocessed items with exponential backoff
    /// - Collecting success/failure statistics
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `items` - Vector of items to write (as AttributeValue HashMaps)
    ///
    /// # Returns
    ///
    /// Returns [`BatchWriteResult`] containing counts of successful and failed items.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - Network connectivity issues occur
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
    ///     let mut items = Vec::new();
    ///     for i in 0..100 {
    ///         let mut item = HashMap::new();
    ///         item.insert("id".to_string(), AttributeValue::S(format!("user{}", i)));
    ///         item.insert("name".to_string(), AttributeValue::S(format!("User {}", i)));
    ///         items.push(item);
    ///     }
    ///
    ///     let result = store.batch_put_items("users", items).await?;
    ///     println!("Success: {}, Failed: {}", result.successful, result.failed);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_put_items(
        &self,
        table_name: &str,
        items: Vec<HashMap<String, AttributeValue>>,
    ) -> Result<BatchWriteResult> {
        Self::validate_table_name(table_name)?;

        if items.is_empty() {
            return Ok(BatchWriteResult {
                successful: 0,
                failed: 0,
                failed_items: Vec::new(),
            });
        }

        let mut successful = 0;
        let mut failed_items = Vec::new();

        // Use chunking utility to split items into DynamoDB-compliant batches
        for chunk in crate::chunking::chunk_items(&items, None) {
            let chunk_items: Vec<_> = chunk.to_vec();

            // Use retry utility for exponential backoff
            let retry_result = crate::retry::retry_with_backoff(
                || self.execute_batch_write(table_name, &chunk_items),
                &crate::retry::RetryConfig::default(),
            )
            .await;

            match retry_result {
                Ok((succeeded, mut failures)) => {
                    successful += succeeded;
                    failed_items.append(&mut failures);
                }
                Err(e) => {
                    // Complete batch failure - record all items as failed
                    for item in chunk_items {
                        failed_items.push(FailedItem {
                            item,
                            error: format!("Batch write error: {}", e),
                        });
                    }
                }
            }
        }

        Ok(BatchWriteResult {
            successful,
            failed: failed_items.len(),
            failed_items,
        })
    }

    /// Execute a single batch write operation
    ///
    /// Returns (successful_count, failed_items, should_retry)
    async fn execute_batch_write(
        &self,
        table_name: &str,
        items: &[HashMap<String, AttributeValue>],
    ) -> Result<((usize, Vec<FailedItem>), bool)> {
        // Build write requests
        let write_requests: Vec<WriteRequest> = items
            .iter()
            .map(|item| {
                WriteRequest::builder()
                    .put_request(
                        PutRequest::builder()
                            .set_item(Some(item.clone()))
                            .build()
                            .expect("PutRequest build should not fail"),
                    )
                    .build()
            })
            .collect();

        // Send batch write request
        let mut request_items = HashMap::new();
        request_items.insert(table_name.to_string(), write_requests);

        let output = self
            .client
            .batch_write_item()
            .set_request_items(Some(request_items))
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        // Process result and determine if retry is needed
        let total_items = items.len();
        let failed_items = Vec::new();

        match output.unprocessed_items {
            Some(unprocessed) if !unprocessed.is_empty() => {
                if let Some(unprocessed_requests) = unprocessed.get(table_name) {
                    let unprocessed_count = unprocessed_requests.len();
                    let successful = total_items - unprocessed_count;

                    // Mark as needing retry (retry utility will handle it)
                    // If this is the last retry attempt, the retry utility will return
                    // and we'll record these as failures in the next call
                    Ok(((successful, failed_items), true))
                } else {
                    // All items processed successfully
                    Ok(((total_items, failed_items), false))
                }
            }
            _ => {
                // All items processed successfully
                Ok(((total_items, failed_items), false))
            }
        }
    }

    /// Batch writes items to DynamoDB using type-safe structs.
    ///
    /// This is a higher-level alternative to [`batch_put_items`](Self::batch_put_items) that works with
    /// any type implementing [`Serialize`]. Items are automatically converted to DynamoDB's
    /// AttributeValue format using `serde_dynamo`.
    ///
    /// The method automatically handles:
    /// - Chunking items into batches of 25 (DynamoDB's limit)
    /// - Retrying unprocessed items with exponential backoff (up to 3 retries)
    /// - Collecting success/failure statistics
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`Serialize`]
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `items` - Slice of items to write
    ///
    /// # Returns
    ///
    /// Returns [`BatchWriteResult`] containing counts of successful and failed items.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - Serialization fails for any item
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let users: Vec<User> = (0..100)
    ///         .map(|i| User {
    ///             id: format!("user{}", i),
    ///             name: format!("User {}", i),
    ///             age: 20 + (i % 50),
    ///         })
    ///         .collect();
    ///
    ///     let result = store.batch_put("users", &users).await?;
    ///     println!("Successfully wrote {} items", result.successful);
    ///     if result.failed > 0 {
    ///         println!("Failed to write {} items", result.failed);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_put<T: Serialize>(
        &self,
        table_name: &str,
        items: &[T],
    ) -> Result<BatchWriteResult> {
        Self::validate_table_name(table_name)?;

        // Convert all items to HashMap using serde_dynamo
        let item_maps: Result<Vec<_>> = items
            .iter()
            .map(|item| {
                serde_dynamo::to_item(item)
                    .map_err(|e| Error::Validation(format!("Failed to serialize item: {}", e)))
            })
            .collect();

        self.batch_put_items(table_name, item_maps?).await
    }

    /// Creates a table-bound store for the specified table.
    ///
    /// This returns a [`TableBoundStore`] that eliminates the need to pass the table name
    /// on every operation. This is useful when implementing the repository pattern or when
    /// working with a specific table extensively.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table to bind to
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     // Create a table-bound store
    ///     let users = store.for_table("users");
    ///
    ///     let user = User { id: "123".into(), name: "John".into() };
    ///     users.put(&user).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn for_table(&self, table_name: impl Into<String>) -> TableBoundStore {
        TableBoundStore {
            store: self.clone(),
            table_name: table_name.into(),
        }
    }
}

/// A table-bound DynamoDB store that binds operations to a specific table.
///
/// This struct wraps a [`DynamoDbStore`] and a table name, eliminating the need to pass
/// the table name on every operation. This is particularly useful when:
///
/// - Implementing the repository pattern (one repository per entity/table)
/// - Working extensively with a specific table
/// - Building domain models with clean architecture principles
///
/// # Example
///
/// ```rust,no_run
/// use clean_dynamodb_store::DynamoDbStore;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// #[derive(Serialize)]
/// struct UserKey {
///     id: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = DynamoDbStore::new().await?;
///     let users = store.for_table("users");
///
///     // Put an item
///     let user = User { id: "123".into(), name: "John".into() };
///     users.put(&user).await?;
///
///     // Get an item
///     let key = UserKey { id: "123".into() };
///     let user: Option<User> = users.get(&key).await?;
///
///     // Delete an item
///     users.delete(&key).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TableBoundStore {
    store: DynamoDbStore,
    table_name: String,
}

impl TableBoundStore {
    /// Gets the table name this store is bound to.
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Inserts or updates an item using a type-safe struct.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`Serialize`]
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to insert or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(PutItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Serialization fails (invalid struct for DynamoDB)
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
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     let users = store.for_table("users");
    ///
    ///     let user = User { id: "123".into(), name: "John".into() };
    ///     users.put(&user).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn put<T: Serialize>(&self, item: &T) -> Result<PutItemOutput> {
        self.store.put(&self.table_name, item).await
    }

    /// Deletes an item using a type-safe key struct.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key struct identifying the item to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(DeleteItemOutput)` on success. The operation succeeds even if the item doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Serialization fails (invalid key struct for DynamoDB)
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
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct UserKey {
    ///     id: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     let users = store.for_table("users");
    ///
    ///     let key = UserKey { id: "123".into() };
    ///     users.delete(&key).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete<K: Serialize>(&self, key: &K) -> Result<DeleteItemOutput> {
        self.store.delete(&self.table_name, key).await
    }

    /// Retrieves an item from DynamoDB and deserializes it into a type-safe struct.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key struct identifying the item to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(T))` if the item exists and was successfully deserialized.
    /// Returns `Ok(None)` if the item does not exist in the table.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Key serialization fails
    /// - Item deserialization fails (data doesn't match expected type)
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize)]
    /// struct UserKey {
    ///     id: String,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     let users = store.for_table("users");
    ///
    ///     let key = UserKey { id: "123".into() };
    ///     match users.get::<UserKey, User>(&key).await? {
    ///         Some(user) => println!("Found user: {}", user.name),
    ///         None => println!("User not found"),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get<K: Serialize, T: DeserializeOwned>(&self, key: &K) -> Result<Option<T>> {
        self.store.get(&self.table_name, key).await
    }

    /// Inserts or updates an item using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `item` - A HashMap containing the attribute names and values for the item
    ///
    /// # Returns
    ///
    /// Returns `Ok(PutItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The item map is empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The item exceeds DynamoDB's size limits (400 KB)
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    pub async fn put_item(&self, item: HashMap<String, AttributeValue>) -> Result<PutItemOutput> {
        self.store.put_item(&self.table_name, item).await
    }

    /// Deletes an item using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `key` - A HashMap containing the primary key attributes
    ///
    /// # Returns
    ///
    /// Returns `Ok(DeleteItemOutput)` on success. The operation succeeds even if the item doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key map is empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The key does not match the table's key schema
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    pub async fn delete_item(&self, key: HashMap<String, AttributeValue>) -> Result<DeleteItemOutput> {
        self.store.delete_item(&self.table_name, key).await
    }

    /// Batch writes items using type-safe structs.
    ///
    /// This method automatically handles chunking items into batches of 25 and retrying
    /// unprocessed items with exponential backoff.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`Serialize`]
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to write
    ///
    /// # Returns
    ///
    /// Returns [`BatchWriteResult`] containing counts of successful and failed items.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     let users = store.for_table("users");
    ///
    ///     let items: Vec<User> = (0..100)
    ///         .map(|i| User {
    ///             id: format!("user{}", i),
    ///             name: format!("User {}", i),
    ///         })
    ///         .collect();
    ///
    ///     let result = users.batch_put(&items).await?;
    ///     println!("Success: {}, Failed: {}", result.successful, result.failed);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_put<T: Serialize>(&self, items: &[T]) -> Result<BatchWriteResult> {
        self.store.batch_put(&self.table_name, items).await
    }

    /// Batch writes items using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of items to write (as AttributeValue HashMaps)
    ///
    /// # Returns
    ///
    /// Returns [`BatchWriteResult`] containing counts of successful and failed items.
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
    ///     let users = store.for_table("users");
    ///
    ///     let mut items = Vec::new();
    ///     for i in 0..100 {
    ///         let mut item = HashMap::new();
    ///         item.insert("id".to_string(), AttributeValue::S(format!("user{}", i)));
    ///         items.push(item);
    ///     }
    ///
    ///     let result = users.batch_put_items(items).await?;
    ///     println!("Success: {}, Failed: {}", result.successful, result.failed);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_put_items(&self, items: Vec<HashMap<String, AttributeValue>>) -> Result<BatchWriteResult> {
        self.store.batch_put_items(&self.table_name, items).await
    }
}
