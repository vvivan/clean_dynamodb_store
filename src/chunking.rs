/// DynamoDB BatchWriteItem maximum items per batch
pub(crate) const DYNAMODB_BATCH_LIMIT: usize = 25;

/// DynamoDB BatchGetItem maximum items per batch
pub(crate) const DYNAMODB_BATCH_GET_LIMIT: usize = 100;

/// Chunk items for DynamoDB BatchWriteItem operations (25 items per batch)
///
/// Splits items into chunks of 25 items each for use with DynamoDB's BatchWriteItem API.
///
/// # Arguments
///
/// * `items` - Slice of items to chunk
///
/// # Returns
///
/// A vector of slices, where each slice contains at most 25 items.
///
/// # Example
///
/// ```ignore
/// let items: Vec<i32> = (0..100).collect();
/// let chunks = chunk_for_write(&items);
/// assert_eq!(chunks.len(), 4); // 25 + 25 + 25 + 25
/// ```
pub(crate) fn chunk_for_write<T>(items: &[T]) -> Vec<&[T]> {
    items.chunks(DYNAMODB_BATCH_LIMIT).collect()
}

/// Chunk items for DynamoDB BatchGetItem operations (100 items per batch)
///
/// Splits items into chunks of 100 items each for use with DynamoDB's BatchGetItem API.
///
/// # Arguments
///
/// * `items` - Slice of items to chunk
///
/// # Returns
///
/// A vector of slices, where each slice contains at most 100 items.
///
/// # Example
///
/// ```ignore
/// let keys: Vec<i32> = (0..250).collect();
/// let chunks = chunk_for_get(&keys);
/// assert_eq!(chunks.len(), 3); // 100 + 100 + 50
/// ```
pub(crate) fn chunk_for_get<T>(items: &[T]) -> Vec<&[T]> {
    items.chunks(DYNAMODB_BATCH_GET_LIMIT).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for chunk_for_write (25-item batches)

    #[test]
    fn test_chunk_for_write_empty() {
        let items: Vec<i32> = vec![];
        let chunks = chunk_for_write(&items);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_for_write_less_than_limit() {
        let items: Vec<i32> = (0..10).collect();
        let chunks = chunk_for_write(&items);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 10);
    }

    #[test]
    fn test_chunk_for_write_exactly_at_limit() {
        let items: Vec<i32> = (0..25).collect();
        let chunks = chunk_for_write(&items);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 25);
    }

    #[test]
    fn test_chunk_for_write_more_than_limit() {
        let items: Vec<i32> = (0..100).collect();
        let chunks = chunk_for_write(&items);
        assert_eq!(chunks.len(), 4); // 25 + 25 + 25 + 25
        assert_eq!(chunks[0].len(), 25);
        assert_eq!(chunks[3].len(), 25);
    }

    #[test]
    fn test_chunk_for_write_with_remainder() {
        let items: Vec<i32> = (0..53).collect();
        let chunks = chunk_for_write(&items);
        assert_eq!(chunks.len(), 3); // 25 + 25 + 3
        assert_eq!(chunks[0].len(), 25);
        assert_eq!(chunks[1].len(), 25);
        assert_eq!(chunks[2].len(), 3);
    }

    // Tests for chunk_for_get (100-item batches)

    #[test]
    fn test_chunk_for_get_empty() {
        let items: Vec<i32> = vec![];
        let chunks = chunk_for_get(&items);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_for_get_less_than_limit() {
        let items: Vec<i32> = (0..50).collect();
        let chunks = chunk_for_get(&items);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 50);
    }

    #[test]
    fn test_chunk_for_get_exactly_at_limit() {
        let items: Vec<i32> = (0..100).collect();
        let chunks = chunk_for_get(&items);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 100);
    }

    #[test]
    fn test_chunk_for_get_more_than_limit() {
        let items: Vec<i32> = (0..300).collect();
        let chunks = chunk_for_get(&items);
        assert_eq!(chunks.len(), 3); // 100 + 100 + 100
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[2].len(), 100);
    }

    #[test]
    fn test_chunk_for_get_with_remainder() {
        let items: Vec<i32> = (0..250).collect();
        let chunks = chunk_for_get(&items);
        assert_eq!(chunks.len(), 3); // 100 + 100 + 50
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
        assert_eq!(chunks[2].len(), 50);
    }
}
