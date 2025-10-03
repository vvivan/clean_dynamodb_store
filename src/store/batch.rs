use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::{BatchGetResult, BatchWriteResult, DynamoDbStore, FailedItem, FailedKey};

impl DynamoDbStore {
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
        for chunk in crate::chunking::chunk_for_write(&items) {
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
    pub(super) async fn execute_batch_write(
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

    /// Batch retrieves items from DynamoDB using the low-level HashMap API.
    ///
    /// This method automatically handles:
    /// - Chunking keys into batches of 100 (DynamoDB's limit)
    /// - Retrying unprocessed keys with exponential backoff
    /// - Collecting all retrieved items and failures
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `keys` - Vector of keys to retrieve (as AttributeValue HashMaps)
    ///
    /// # Returns
    ///
    /// Returns [`BatchGetResult`] containing retrieved items and failure information.
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
    ///     let mut keys = Vec::new();
    ///     for i in 0..150 {
    ///         let mut key = HashMap::new();
    ///         key.insert("id".to_string(), AttributeValue::S(format!("user{}", i)));
    ///         keys.push(key);
    ///     }
    ///
    ///     let result = store.batch_get_items("users", keys).await?;
    ///     println!("Retrieved: {}, Failed: {}", result.successful, result.failed);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_get_items(
        &self,
        table_name: &str,
        keys: Vec<HashMap<String, AttributeValue>>,
    ) -> Result<BatchGetResult<HashMap<String, AttributeValue>>> {
        Self::validate_table_name(table_name)?;

        if keys.is_empty() {
            return Ok(BatchGetResult {
                successful: 0,
                failed: 0,
                items: Vec::new(),
                failed_keys: Vec::new(),
            });
        }

        let mut all_items = Vec::new();
        let mut failed_keys = Vec::new();

        // Use chunking utility to split keys into DynamoDB-compliant batches
        for chunk in crate::chunking::chunk_for_get(&keys) {
            let chunk_keys: Vec<_> = chunk.to_vec();

            // Use retry utility for exponential backoff
            let retry_result = crate::retry::retry_with_backoff(
                || self.execute_batch_get(table_name, &chunk_keys),
                &crate::retry::RetryConfig::default(),
            )
            .await;

            match retry_result {
                Ok((mut items, mut failures)) => {
                    all_items.append(&mut items);
                    failed_keys.append(&mut failures);
                }
                Err(e) => {
                    // Complete batch failure - record all keys as failed
                    for key in chunk_keys {
                        failed_keys.push(FailedKey {
                            key,
                            error: format!("Batch get error: {}", e),
                        });
                    }
                }
            }
        }

        Ok(BatchGetResult {
            successful: all_items.len(),
            failed: failed_keys.len(),
            items: all_items,
            failed_keys,
        })
    }

    /// Execute a single batch get operation
    ///
    /// Returns (retrieved_items, failed_keys, should_retry)
    pub(super) async fn execute_batch_get(
        &self,
        table_name: &str,
        keys: &[HashMap<String, AttributeValue>],
    ) -> Result<((Vec<HashMap<String, AttributeValue>>, Vec<FailedKey>), bool)> {
        // Build keys and attributes for batch get
        let keys_and_attrs = KeysAndAttributes::builder()
            .set_keys(Some(keys.to_vec()))
            .build()
            .map_err(|e| Error::Validation(format!("Failed to build KeysAndAttributes: {}", e)))?;

        // Send batch get request
        let mut request_items = HashMap::new();
        request_items.insert(table_name.to_string(), keys_and_attrs);

        let output = self
            .client
            .batch_get_item()
            .set_request_items(Some(request_items))
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        // Collect retrieved items
        let mut retrieved_items = Vec::new();
        if let Some(responses) = output.responses
            && let Some(items) = responses.get(table_name)
        {
            retrieved_items = items.clone();
        }

        let failed_keys = Vec::new();

        // Check for unprocessed keys
        match output.unprocessed_keys {
            Some(unprocessed) if !unprocessed.is_empty() => {
                if let Some(_unprocessed_keys_and_attrs) = unprocessed.get(table_name) {
                    // Mark as needing retry (retry utility will handle it)
                    Ok(((retrieved_items, failed_keys), true))
                } else {
                    // All keys processed successfully
                    Ok(((retrieved_items, failed_keys), false))
                }
            }
            _ => {
                // All keys processed successfully
                Ok(((retrieved_items, failed_keys), false))
            }
        }
    }

    /// Batch retrieves items from DynamoDB using type-safe structs.
    ///
    /// This is a higher-level alternative to [`batch_get_items`](Self::batch_get_items) that works with
    /// any key type implementing [`Serialize`] and deserializes results to any type implementing
    /// [`DeserializeOwned`].
    ///
    /// The method automatically handles:
    /// - Chunking keys into batches of 100 (DynamoDB's limit)
    /// - Retrying unprocessed keys with exponential backoff (up to 3 retries)
    /// - Deserializing retrieved items to the specified type
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `keys` - Slice of keys to retrieve
    ///
    /// # Returns
    ///
    /// Returns [`BatchGetResult<T>`] containing retrieved items and failure information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - Key serialization fails
    /// - Item deserialization fails
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
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
    ///     let keys: Vec<UserKey> = (0..150)
    ///         .map(|i| UserKey {
    ///             id: format!("user{}", i),
    ///         })
    ///         .collect();
    ///
    ///     let result = store.batch_get::<UserKey, User>("users", &keys).await?;
    ///     println!("Retrieved {} users", result.items.len());
    ///     for user in result.items {
    ///         println!("User: {} (age {})", user.name, user.age);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_get<K: Serialize, T: DeserializeOwned>(
        &self,
        table_name: &str,
        keys: &[K],
    ) -> Result<BatchGetResult<T>> {
        Self::validate_table_name(table_name)?;

        // Convert all keys to HashMap using serde_dynamo
        let key_maps: Result<Vec<_>> = keys
            .iter()
            .map(|key| {
                serde_dynamo::to_item(key)
                    .map_err(|e| Error::Validation(format!("Failed to serialize key: {}", e)))
            })
            .collect();

        // Get items using low-level API
        let result = self.batch_get_items(table_name, key_maps?).await?;

        // Deserialize items to type T
        let deserialized_items: Result<Vec<T>> = result
            .items
            .iter()
            .map(|item| {
                serde_dynamo::from_item(item.clone())
                    .map_err(|e| Error::Validation(format!("Failed to deserialize item: {}", e)))
            })
            .collect();

        Ok(BatchGetResult {
            successful: result.successful,
            failed: result.failed,
            items: deserialized_items?,
            failed_keys: result.failed_keys,
        })
    }
}
