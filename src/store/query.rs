use aws_sdk_dynamodb::types::AttributeValue;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::DynamoDbStore;

/// Result type for paginated query operations.
///
/// Contains the retrieved items, count, and pagination information.
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    /// The items retrieved from the query
    pub items: Vec<T>,
    /// The number of items returned
    pub count: usize,
    /// The primary key of the last item evaluated (for pagination)
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

impl DynamoDbStore {
    /// Queries items from a DynamoDB table using low-level HashMap API.
    ///
    /// This method retrieves items that match a key condition expression. It's optimized
    /// for retrieving items with the same partition key and optionally filtering by sort key.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `key_condition_expression` - Expression to filter items (e.g., "id = :id" or "id = :id AND created_at > :date")
    /// * `expression_attribute_values` - HashMap mapping placeholder values in the expression (e.g., ":id") to AttributeValues
    /// * `expression_attribute_names` - Optional HashMap mapping placeholder names in the expression (e.g., "#name") to actual attribute names
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryResult<HashMap<String, AttributeValue>>)` containing the retrieved items and pagination info.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table name is empty
    /// - The key condition expression is empty
    /// - Expression attribute values are empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The key condition expression is invalid
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
    ///     let key_condition_expression = "user_id = :user_id".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     let result = store.query_items(
    ///         "orders",
    ///         key_condition_expression,
    ///         values,
    ///         None,
    ///     ).await?;
    ///
    ///     println!("Found {} orders", result.count);
    ///     for item in result.items {
    ///         println!("Order: {:?}", item);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with sort key condition
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
    ///     // Query items with partition key and sort key range
    ///     let key_condition_expression = "user_id = :user_id AND created_at > :date".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///     values.insert(":date".to_string(), AttributeValue::N("1640000000".to_string()));
    ///
    ///     let result = store.query_items("events", key_condition_expression, values, None).await?;
    ///
    ///     println!("Found {} recent events", result.count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn query_items(
        &self,
        table_name: &str,
        key_condition_expression: String,
        expression_attribute_values: HashMap<String, AttributeValue>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<QueryResult<HashMap<String, AttributeValue>>> {
        Self::validate_table_name(table_name)?;

        if key_condition_expression.trim().is_empty() {
            return Err(Error::Validation("Key condition expression cannot be empty".to_string()));
        }

        if expression_attribute_values.is_empty() {
            return Err(Error::Validation("Expression attribute values cannot be empty".to_string()));
        }

        let result = self
            .client
            .query()
            .table_name(table_name)
            .key_condition_expression(key_condition_expression)
            .set_expression_attribute_values(Some(expression_attribute_values))
            .set_expression_attribute_names(expression_attribute_names)
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        let items = result.items.unwrap_or_default();
        let count = items.len();
        let last_evaluated_key = result.last_evaluated_key;

        Ok(QueryResult {
            items,
            count,
            last_evaluated_key,
        })
    }

    /// Queries items from a DynamoDB table and deserializes them into type-safe structs.
    ///
    /// This is a higher-level alternative to [`query_items`](Self::query_items) that automatically
    /// deserializes the retrieved items to the specified type using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
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
    /// - The table name is empty
    /// - The key condition expression is empty
    /// - Expression attribute values are empty
    /// - Item deserialization fails
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
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
    ///
    ///     let key_condition_expression = "user_id = :user_id".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     let result = store.query::<Order>(
    ///         "orders",
    ///         key_condition_expression,
    ///         values,
    ///         None,
    ///     ).await?;
    ///
    ///     println!("Found {} orders", result.count);
    ///     for order in result.items {
    ///         println!("Order {}: ${:.2}", order.order_id, order.total);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with pagination
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use serde::Deserialize;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Deserialize)]
    /// struct Event {
    ///     user_id: String,
    ///     created_at: u64,
    ///     event_type: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let key_condition_expression = "user_id = :user_id".to_string();
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":user_id".to_string(), AttributeValue::S("user123".to_string()));
    ///
    ///     let result = store.query::<Event>("events", key_condition_expression, values, None).await?;
    ///
    ///     println!("First page: {} events", result.count);
    ///
    ///     // Check if there are more results
    ///     if result.last_evaluated_key.is_some() {
    ///         println!("More results available - use last_evaluated_key for pagination");
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn query<T: DeserializeOwned>(
        &self,
        table_name: &str,
        key_condition_expression: String,
        expression_attribute_values: HashMap<String, AttributeValue>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<QueryResult<T>> {
        let result = self
            .query_items(
                table_name,
                key_condition_expression,
                expression_attribute_values,
                expression_attribute_names,
            )
            .await?;

        // Deserialize items to type T
        let deserialized_items: Result<Vec<T>> = result
            .items
            .iter()
            .map(|item| {
                serde_dynamo::from_item(item.clone())
                    .map_err(|e| Error::Validation(format!("Failed to deserialize item: {}", e)))
            })
            .collect();

        Ok(QueryResult {
            items: deserialized_items?,
            count: result.count,
            last_evaluated_key: result.last_evaluated_key,
        })
    }
}
