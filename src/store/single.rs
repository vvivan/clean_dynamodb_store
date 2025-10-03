use aws_sdk_dynamodb::{
    operation::delete_item::DeleteItemOutput,
    operation::put_item::PutItemOutput,
    types::AttributeValue,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::DynamoDbStore;

impl DynamoDbStore {
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
}
