use aws_sdk_dynamodb::types::AttributeValue;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::DynamoDbStore;

/// Result type for paginated scan operations.
///
/// Contains the retrieved items, counts, and pagination information.
#[derive(Debug, Clone)]
pub struct ScanResult<T> {
    /// The items retrieved from the scan
    pub items: Vec<T>,
    /// The number of items returned after applying any filter
    pub count: usize,
    /// The number of items evaluated before applying any filter
    pub scanned_count: usize,
    /// The primary key of the last item evaluated (for pagination)
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

impl DynamoDbStore {
    /// Scans all items in a DynamoDB table using low-level HashMap API.
    ///
    /// This method retrieves all items in a table, optionally filtered by a filter expression.
    /// Note that scan is less efficient than query as it examines every item in the table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
    /// * `filter_expression` - Optional expression to filter items after scanning (e.g., "age > :min_age")
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
    /// - The table name is empty
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The filter expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     // Scan all items in the table
    ///     let result = store.scan_items("users", None, None, None).await?;
    ///
    ///     println!("Found {} items", result.count);
    ///     println!("Scanned {} items", result.scanned_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with filter
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
    ///     let filter_expression = Some("age > :min_age".to_string());
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":min_age".to_string(), AttributeValue::N("18".to_string()));
    ///
    ///     let result = store.scan_items(
    ///         "users",
    ///         filter_expression,
    ///         Some(values),
    ///         None,
    ///     ).await?;
    ///
    ///     println!("Found {} users over 18", result.count);
    ///     println!("Scanned {} total users", result.scanned_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with reserved keyword
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
    ///     // Using expression attribute names for reserved keyword "status"
    ///     let filter_expression = Some("#status = :status_value".to_string());
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":status_value".to_string(), AttributeValue::S("active".to_string()));
    ///
    ///     let mut names = HashMap::new();
    ///     names.insert("#status".to_string(), "status".to_string());
    ///
    ///     let result = store.scan_items("users", filter_expression, Some(values), Some(names)).await?;
    ///
    ///     println!("Found {} active users", result.count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn scan_items(
        &self,
        table_name: &str,
        filter_expression: Option<String>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<ScanResult<HashMap<String, AttributeValue>>> {
        Self::validate_table_name(table_name)?;

        let result = self
            .client
            .scan()
            .table_name(table_name)
            .set_filter_expression(filter_expression)
            .set_expression_attribute_values(expression_attribute_values)
            .set_expression_attribute_names(expression_attribute_names)
            .send()
            .await
            .map_err(|e| Error::AwsSdk(Box::new(e.into())))?;

        let scanned_count = result.count() as usize;
        let last_evaluated_key = result.last_evaluated_key;
        let items = result.items.unwrap_or_default();
        let count = items.len();

        Ok(ScanResult {
            items,
            count,
            scanned_count,
            last_evaluated_key,
        })
    }

    /// Scans all items in a DynamoDB table and deserializes them into type-safe structs.
    ///
    /// This is a higher-level alternative to [`scan_items`](Self::scan_items) that automatically
    /// deserializes the retrieved items to the specified type using `serde_dynamo`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements [`DeserializeOwned`] for the item data
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the DynamoDB table
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
    /// - The table name is empty
    /// - Item deserialization fails
    /// - AWS credentials are not properly configured
    /// - The specified table does not exist
    /// - The filter expression is invalid
    /// - Network connectivity issues occur
    /// - IAM permissions are insufficient
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use serde::Deserialize;
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
    ///     let result = store.scan::<User>("users", None, None, None).await?;
    ///
    ///     println!("Found {} users", result.count);
    ///     for user in result.items {
    ///         println!("User: {} (age {})", user.name, user.age);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Example with filter
    ///
    /// ```rust,no_run
    /// use clean_dynamodb_store::DynamoDbStore;
    /// use aws_sdk_dynamodb::types::AttributeValue;
    /// use serde::Deserialize;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Deserialize)]
    /// struct Product {
    ///     id: String,
    ///     name: String,
    ///     price: f64,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     // Find all products under $50
    ///     let filter_expression = Some("price < :max_price".to_string());
    ///
    ///     let mut values = HashMap::new();
    ///     values.insert(":max_price".to_string(), AttributeValue::N("50.00".to_string()));
    ///
    ///     let result = store.scan::<Product>("products", filter_expression, Some(values), None).await?;
    ///
    ///     println!("Found {} affordable products", result.count);
    ///     println!("Scanned {} total products", result.scanned_count);
    ///
    ///     for product in result.items {
    ///         println!("{}: ${:.2}", product.name, product.price);
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
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Item {
    ///     id: String,
    ///     data: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = DynamoDbStore::new().await?;
    ///
    ///     let result = store.scan::<Item>("items", None, None, None).await?;
    ///
    ///     println!("First page: {} items", result.count);
    ///
    ///     // Check if there are more results
    ///     if result.last_evaluated_key.is_some() {
    ///         println!("More results available - use last_evaluated_key for pagination");
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn scan<T: DeserializeOwned>(
        &self,
        table_name: &str,
        filter_expression: Option<String>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
        expression_attribute_names: Option<HashMap<String, String>>,
    ) -> Result<ScanResult<T>> {
        let result = self
            .scan_items(
                table_name,
                filter_expression,
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

        Ok(ScanResult {
            items: deserialized_items?,
            count: result.count,
            scanned_count: result.scanned_count,
            last_evaluated_key: result.last_evaluated_key,
        })
    }
}
