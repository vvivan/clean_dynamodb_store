/// DynamoDB BatchWriteItem maximum items per batch
pub(crate) const DYNAMODB_BATCH_LIMIT: usize = 25;

/// Chunk items into batches with configurable size
///
/// Splits a slice into chunks of a specified size. If no chunk size is provided,
/// defaults to DynamoDB's BatchWriteItem limit of 25 items.
///
/// # Arguments
///
/// * `items` - Slice of items to chunk
/// * `chunk_size` - Optional chunk size. If None, defaults to `DYNAMODB_BATCH_LIMIT` (25)
///
/// # Returns
///
/// A vector of slices, where each slice contains at most `chunk_size` items.
///
/// # Example
///
/// ```ignore
/// // Internal utility - use through DynamoDbStore::batch_put() instead
/// let items: Vec<i32> = (0..100).collect();
/// let chunks = chunk_items(&items, None);
/// assert_eq!(chunks.len(), 4);
///
/// // Use custom chunk size (useful for testing)
/// let chunks = chunk_items(&items, Some(10));
/// assert_eq!(chunks.len(), 10);
/// ```
pub(crate) fn chunk_items<T>(items: &[T], chunk_size: Option<usize>) -> Vec<&[T]> {
    let size = chunk_size.unwrap_or(DYNAMODB_BATCH_LIMIT);
    items.chunks(size).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_empty() {
        let items: Vec<i32> = vec![];
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_with_default_size() {
        let items: Vec<i32> = (0..100).collect();
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 4); // 25 + 25 + 25 + 25
        assert_eq!(chunks[0].len(), 25);
    }

    #[test]
    fn test_chunk_less_than_default_limit() {
        let items: Vec<i32> = (0..10).collect();
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 10);
    }

    #[test]
    fn test_chunk_exactly_default_limit() {
        let items: Vec<i32> = (0..25).collect();
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 25);
    }

    #[test]
    fn test_chunk_with_custom_size() {
        let items: Vec<i32> = (0..100).collect();
        let chunks = chunk_items(&items, Some(10));
        assert_eq!(chunks.len(), 10); // 10 chunks of 10
        assert_eq!(chunks[0].len(), 10);
        assert_eq!(chunks[9].len(), 10);
    }

    #[test]
    fn test_chunk_custom_size_with_remainder() {
        let items: Vec<i32> = (0..23).collect();
        let chunks = chunk_items(&items, Some(10));
        assert_eq!(chunks.len(), 3); // 10 + 10 + 3
        assert_eq!(chunks[0].len(), 10);
        assert_eq!(chunks[1].len(), 10);
        assert_eq!(chunks[2].len(), 3);
    }

    #[test]
    fn test_chunk_default_with_remainder() {
        let items: Vec<i32> = (0..53).collect();
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 3); // 25 + 25 + 3
        assert_eq!(chunks[0].len(), 25);
        assert_eq!(chunks[1].len(), 25);
        assert_eq!(chunks[2].len(), 3);
    }

    #[test]
    fn test_chunk_single_item() {
        let items: Vec<i32> = vec![42];
        let chunks = chunk_items(&items, None);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 1);
        assert_eq!(chunks[0][0], 42);
    }

    #[test]
    fn test_chunk_custom_size_one() {
        let items: Vec<i32> = (0..5).collect();
        let chunks = chunk_items(&items, Some(1));
        assert_eq!(chunks.len(), 5); // Each item in its own chunk
        assert_eq!(chunks[0][0], 0);
        assert_eq!(chunks[4][0], 4);
    }
}
