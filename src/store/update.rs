use aws_sdk_dynamodb::{
    operation::update_item::UpdateItemOutput,
    types::AttributeValue,
};
use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::DynamoDbStore;

impl DynamoDbStore {
    /// Updates an item in a DynamoDB table using low-level HashMap API.
    ///
    /// This method allows you to modify attributes of an existing item or add new attributes
    /// using update expressions. It provides fine-grained control over how items are updated.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `key` - A HashMap containing the primary key attributes that identify the item to update
    /// * `update_expression` - A string that defines how to update the item (e.g., "SET #name = :name, #age = :age")
    /// * `expression_attribute_values` - Optional HashMap mapping placeholder values in the update expression (e.g., ":name") to AttributeValues
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the update expression (e.g., "#name") to actual attribute names
    ///
    /// # Returns
    ///
    /// Returns `Ok(UpdateItemOutput)` on success, containing the response from DynamoDB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - The key map is empty
    /// - The update expression is empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The key does not match the table's key schema
    /// - The update expression is invalid
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
    ///     // Define the key
    ///     let mut key = HashMap::new();
    ///     key.insert("id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     // Define the update expression
    ///     let update_expression = "SET #name = :name, age = :age".to_string();
    ///
    ///     // Define expression attribute values
    ///     let mut values = HashMap::new();
    ///     values.insert(":name".to_string(), AttributeValue::S("John Doe".to_string()));
    ///     values.insert(":age".to_string(), AttributeValue::N("31".to_string()));
    ///
    ///     // Define expression attribute names (for reserved keywords)
    ///     let mut names = HashMap::new();
    ///     names.insert("#name".to_string(), "name".to_string());
    ///
    ///     store.update_item(
    ///         "users",
    ///         key,
    ///         update_expression,
    ///         Some(values),
    ///         Some(names),
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with ADD operation
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
    ///
    ///     // Increment a counter
    ///     let update_expression = "ADD login_count :inc".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":inc".to_string(), AttributeValue::N("1".to_string()));
    ///
    ///     store.update_item("users", key, update_expression, Some(values), None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_item(
        &self,
        table_name: &str,
        key: HashMap<String, AttributeValue>,
        update_expression: String,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<UpdateItemOutput> {
        Self::validate_table_name(table_name)?;
        Self::validate_not_empty(&key, "Key")?;

        if update_expression.trim().is_empty() {
            return Err(Error::Validation("Update expression cannot be empty".to_string()));
        }

        let result = self
            .client
            .update_item()
            .table_name(table_name)
            .set_key(Some(key))
            .update_expression(update_expression)
            .set_expression_attribute_values(expression_attribute_values)
            .set_expression_attribute_names(expression_attribute_names)
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        Ok(result)
    }

    /// Updates an item using a type-safe key struct.
    ///
    /// This is a higher-level alternative to [`update_item`](Self::update_item) that works with
    /// any key type implementing [`Serialize`]. The key struct is automatically converted to
    /// DynamoDB's AttributeValue format using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Any type that implements [`Serialize`] representing the primary key
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
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
    /// - The table name is empty
    /// - The update expression is empty
    /// - Key serialization fails
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
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
    ///
    ///     let key = UserKey {
    ///         id: "user123".to_string(),
    ///     };
    ///
    ///     let update_expression = "SET #name = :name, age = :age".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":name".to_string(), AttributeValue::S("Jane Doe".to_string()));
    ///     values.insert(":age".to_string(), AttributeValue::N("28".to_string()));
    ///
    ///     let mut names = HashMap::new();
    ///     names.insert("#name".to_string(), "name".to_string());
    ///
    ///     store.update(
    ///         "users",
    ///         &key,
    ///         update_expression,
    ///         Some(values),
    ///         Some(names),
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with conditional update
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
    ///
    ///     let key = UserKey { id: "user123".to_string() };
    ///
    ///     // Increment counter only
    ///     let update_expression = "ADD view_count :inc".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":inc".to_string(), AttributeValue::N("1".to_string()));
    ///
    ///     store.update("users", &key, update_expression, Some(values), None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update<K: Serialize>(
        &self,
        table_name: &str,
        key: &K,
        update_expression: String,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<UpdateItemOutput> {
        Self::validate_table_name(table_name)?;

        let key_map = serde_dynamo::to_item(key)
            .map_err(|e| Error::Validation(format!("Failed to serialize key: {}", e)))?;

        self.update_item(
            table_name,
            key_map,
            update_expression,
            expression_attribute_values,
            expression_attribute_names,
        )
        .await
    }
}
