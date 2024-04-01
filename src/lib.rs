use aws_sdk_dynamodb::{operation::put_item::PutItemOutput, types::AttributeValue};
use std::collections::HashMap;

pub async fn put_item(
    table_name: &str,
    item: HashMap<String, AttributeValue>,
) -> Result<PutItemOutput, aws_sdk_dynamodb::Error> {
    let config = aws_config::load_from_env().await;

    let result = aws_sdk_dynamodb::Client::new(&config)
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;

    Ok(result)
}
