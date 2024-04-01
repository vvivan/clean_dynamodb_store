use std::collections::HashMap;

use aws_sdk_dynamodb::{operation::delete_item::DeleteItemOutput, types::AttributeValue};

pub async fn delete_item(
    table_name: &str,
    key: HashMap<String, AttributeValue>,
) -> Result<DeleteItemOutput, aws_sdk_dynamodb::Error> {
    let config = aws_config::load_from_env().await;

    let result = aws_sdk_dynamodb::Client::new(&config)
        .delete_item()
        .table_name(table_name)
        .set_key(Some(key))
        .send()
        .await?;

    Ok(result)
}
