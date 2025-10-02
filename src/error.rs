use thiserror::Error;

/// Errors that can occur when using the DynamoDB store.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred while interacting with AWS DynamoDB.
    #[error("AWS SDK error: {0}")]
    AwsSdk(#[from] Box<aws_sdk_dynamodb::Error>),

    /// Validation error for invalid input parameters.
    #[error("Validation error: {0}")]
    Validation(String),
}

/// A specialized Result type for DynamoDB store operations.
pub type Result<T> = std::result::Result<T, Error>;
