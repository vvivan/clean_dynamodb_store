use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

use crate::error::{Error, Result};

mod single;
mod batch;
mod table_bound;
mod update;
mod query;
mod scan;

// Re-export query and scan result types
pub use query::QueryResult;
pub use scan::ScanResult;

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
    pub item: HashMap<String, aws_sdk_dynamodb::types::AttributeValue>,
    /// Error message describing why it failed
    pub error: String,
}

/// Result of a batch get operation.
///
/// Contains retrieved items and information about successful and failed keys after all retry attempts.
#[derive(Debug, Clone)]
pub struct BatchGetResult<T> {
    /// Number of successfully retrieved items
    pub successful: usize,
    /// Number of failed keys after all retries
    pub failed: usize,
    /// Successfully retrieved items
    pub items: Vec<T>,
    /// Keys that permanently failed with error details
    pub failed_keys: Vec<FailedKey>,
}

/// Information about a key that failed to retrieve after all retry attempts.
#[derive(Debug, Clone)]
pub struct FailedKey {
    /// The key that failed (in DynamoDB's AttributeValue format)
    pub key: HashMap<String, aws_sdk_dynamodb::types::AttributeValue>,
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
    pub(super) fn validate_table_name(table_name: &str) -> Result<()> {
        if table_name.trim().is_empty() {
            return Err(Error::Validation("Table name cannot be empty".to_string()));
        }
        Ok(())
    }

    /// Validates that items or keys are not empty.
    pub(super) fn validate_not_empty(map: &HashMap<String, aws_sdk_dynamodb::types::AttributeValue>, field_name: &str) -> Result<()> {
        if map.is_empty() {
            return Err(Error::Validation(format!("{} cannot be empty", field_name)));
        }
        Ok(())
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
