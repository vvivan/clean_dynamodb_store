use aws_sdk_dynamodb::{
    operation::delete_item::DeleteItemOutput,
    operation::put_item::PutItemOutput,
    operation::update_item::UpdateItemOutput,
    types::AttributeValue,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::Result;
use super::{BatchGetResult, BatchWriteResult, QueryResult, ScanResult, TableBoundStore};

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

    /// Batch retrieves items using type-safe structs.
    ///
    /// This method automatically handles chunking keys into batches of 100 and retrying
    /// unprocessed keys with exponential backoff.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of keys to retrieve
    ///
    /// # Returns
    ///
    /// Returns [`BatchGetResult<T>`] containing retrieved items and failure information.
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
    ///     let keys: Vec<UserKey> = (0..150)
    ///         .map(|i| UserKey {
    ///             id: format!("user{}", i),
    ///         })
    ///         .collect();
    ///
    ///     let result = users.batch_get::<UserKey, User>(&keys).await?;
    ///     println!("Retrieved {} users", result.items.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_get<K: Serialize, T: DeserializeOwned>(&self, keys: &[K]) -> Result<BatchGetResult<T>> {
        self.store.batch_get(&self.table_name, keys).await
    }

    /// Batch retrieves items using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `keys` - Vector of keys to retrieve (as AttributeValue HashMaps)
    ///
    /// # Returns
    ///
    /// Returns [`BatchGetResult`] containing retrieved items and failure information.
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
    ///     let mut keys = Vec::new();
    ///     for i in 0..150 {
    ///         let mut key = HashMap::new();
    ///         key.insert("id".to_string(), AttributeValue::S(format!("user{}", i)));
    ///         keys.push(key);
    ///     }
    ///
    ///     let result = users.batch_get_items(keys).await?;
    ///     println!("Retrieved: {}, Failed: {}", result.successful, result.failed);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_get_items(&self, keys: Vec<HashMap<String, AttributeValue>>) -> Result<BatchGetResult<HashMap<String, AttributeValue>>> {
        self.store.batch_get_items(&self.table_name, keys).await
    }

    /// Updates an item using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `key` - A HashMap containing the primary key attributes
    /// * `update_expression` - A string that defines how to update the item
    /// * `expression_attribute_values` - Optional HashMap mapping placeholder values in the update expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the update expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(UpdateItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key map is empty
    /// - The update expression is empty
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - The update expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    pub async fn update_item(
        &self,
        key: HashMap<String, AttributeValue>,
        update_expression: String,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<UpdateItemOutput> {
        self.store.update_item(
            &self.table_name,
            key,
            update_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }

    /// Updates an item using a type-safe key struct.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key struct identifying the item to update
    /// * `update_expression` - A string that defines how to update the item
    /// * `expression_attribute_values` - Optional HashMap mapping placeholder values in the update expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the update expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(UpdateItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The update expression is empty
    /// - Key serialization fails
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - The update expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use serde::Serialize;
    /// use std::collections::HashMap;
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
    ///     let key = UserKey { id: "user123".into() };
    ///     let update_expression = "SET age = :age".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":age".to_string(), AttributeValue::N("31".to_string()));
    ///
    ///     users.update(&key, update_expression, Some(values), None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update<K: Serialize>(
        &self,
        key: &K,
        update_expression: String,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<UpdateItemOutput> {
        self.store.update(
            &self.table_name,
            key,
            update_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }

    /// Queries items using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `key_condition_expression` - Expression to filter items
    /// * `expression_attribute_values` - HashMap mapping placeholder values in the expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryResult<HashMap<String, AttributeValue>>)` containing the retrieved items and pagination info.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key condition expression is empty
    /// - Expression attribute values are empty
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - The key condition expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    pub async fn query_items(
        &self,
        key_condition_expression: String,
        expression_attribute_values: HashMap<String, AttributeValue>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<QueryResult<HashMap<String, AttributeValue>>> {
        self.store.query_items(
            &self.table_name,
            key_condition_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }

    /// Queries items and deserializes them into type-safe structs.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `key_condition_expression` - Expression to filter items
    /// * `expression_attribute_values` - HashMap mapping placeholder values in the expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryResult<T>)` containing the retrieved items and pagination info.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key condition expression is empty
    /// - Expression attribute values are empty
    /// - Item deserialization fails
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use serde::Deserialize;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Deserialize)]
    /// struct Order {
    ///     user_id: String,
    ///     order_id: String,
    ///     total: f64,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///     let orders = store.for_table("orders");
    ///
    ///     let key_condition_expression = "user_id = :user_id".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     let result = orders.query::<Order>(key_condition_expression, values, None).await?;
    ///
    ///     println!("Found {} orders", result.count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn query<T: DeserializeOwned>(
        &self,
        key_condition_expression: String,
        expression_attribute_values: HashMap<String, AttributeValue>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<QueryResult<T>> {
        self.store.query(
            &self.table_name,
            key_condition_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }

    /// Scans all items using low-level HashMap API.
    ///
    /// # Arguments
    ///
    /// * `filter_expression` - Optional expression to filter items after scanning
    /// * `expression_attribute_values` - Optional HashMap mapping placeholder values in the filter expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the filter expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(ScanResult<HashMap<String, AttributeValue>>)` containing the retrieved items and counts.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - The filter expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    pub async fn scan_items(
        &self,
        filter_expression: Option<String>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<ScanResult<HashMap<String, AttributeValue>>> {
        self.store.scan_items(
            &self.table_name,
            filter_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }

    /// Scans all items and deserializes them into type-safe structs.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `filter_expression` - Optional expression to filter items after scanning
    /// * `expression_attribute_values` - Optional HashMap mapping placeholder values in the filter expression
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the filter expression
    ///
    /// # Returns
    ///
    /// Returns `Ok(ScanResult<T>)` containing the retrieved items and counts.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Item deserialization fails
    /// - AWS credentials are not properly configured
    /// - The table does not exist
    /// - The filter expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use serde::Deserialize;
    /// use std::collections::HashMap;
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
    ///     let users = store.for_table("users");
    ///
    ///     let filter_expression = Some("age > :min_age".to_string());
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":min_age".to_string(), AttributeValue::N("18".to_string()));
    ///
    ///     let result = users.scan::<User>(filter_expression, Some(values), None).await?;
    ///
    ///     println!("Found {} users", result.count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn scan<T: DeserializeOwned>(
        &self,
        filter_expression: Option<String>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<ScanResult<T>> {
        self.store.scan(
            &self.table_name,
            filter_expression,
            expression_attribute_values,
            expression_attribute_names,
        ).await
    }
}
