use std::time::Duration;

/// Configuration for retry behavior with exponential backoff
///
/// This configuration defines how retries should be handled, including
/// the maximum number of attempts, initial delay, and backoff multiplier.
#[derive(Clone, Debug)]
pub(crate) struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: usize,
    /// Initial delay in milliseconds before first retry
    pub initial_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            backoff_multiplier: 2,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt
    ///
    /// Uses exponential backoff: delay = initial_delay * multiplier^attempt
    ///
    /// # Arguments
    ///
    /// * `attempt` - The attempt number (0-indexed)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Internal utility - used by batch operations
    /// let config = RetryConfig::default();
    /// assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
    /// assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
    /// assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
    /// ```
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let delay_ms = self.initial_delay_ms * self.backoff_multiplier.pow(attempt as u32);
        Duration::from_millis(delay_ms)
    }
}

/// Retry an async operation with exponential backoff
///
/// Executes an operation repeatedly until it succeeds, indicates no retry is needed,
/// or the maximum number of retries is reached.
///
/// # Type Parameters
///
/// * `F` - The operation closure
/// * `T` - The success result type
/// * `E` - The error type
/// * `Fut` - The future type returned by the closure
///
/// # Arguments
///
/// * `operation` - Closure that returns a future producing `Result<(T, bool), E>`
///   - The bool indicates whether a retry should be attempted
/// * `config` - Retry configuration specifying max retries and backoff parameters
///
/// # Returns
///
/// Returns the final result `T` or an error `E`.
///
/// # Example
///
/// ```ignore
/// // Internal utility - used by batch operations
/// let result = retry_with_backoff(
///     || async {
///         // Perform operation
///         let success = true;
///         let should_retry = false;
///         Ok((42, should_retry))
///     },
///     &RetryConfig::default(),
/// ).await?;
/// ```
pub(crate) async fn retry_with_backoff<F, T, E, Fut>(
    mut operation: F,
    config: &RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<(T, bool), E>>,
{
    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok((result, should_retry)) => {
                // Return if no retry needed or max retries reached
                if !should_retry || attempt == config.max_retries {
                    return Ok(result);
                }

                // Sleep with exponential backoff before next attempt
                let delay = config.delay_for_attempt(attempt);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!("Loop should return before this point");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.backoff_multiplier, 2);
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig::default();
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_custom_config_delay() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 50,
            backoff_multiplier: 3,
        };
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(50));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(150));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(450));
    }

    #[tokio::test]
    async fn test_no_retry_on_success() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result: Result<i32, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok((42, false)) // Success, no retry needed
                }
            },
            &RetryConfig::default(),
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_until_success() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result: Result<i32, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    let attempts = count.fetch_add(1, Ordering::SeqCst);
                    if attempts < 2 {
                        Ok((attempts as i32, true)) // Request retry
                    } else {
                        Ok((attempts as i32, false)) // Success
                    }
                }
            },
            &RetryConfig::default(),
        )
        .await;

        assert_eq!(result.unwrap(), 2);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result: Result<i32, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok((99, true)) // Always request retry
                }
            },
            &RetryConfig::default(),
        )
        .await;

        // Should return the result after max retries
        assert_eq!(result.unwrap(), 99);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 4); // 1 initial + 3 retries
    }

    #[tokio::test]
    async fn test_immediate_error() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result: Result<i32, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err("operation failed".to_string())
                }
            },
            &RetryConfig::default(),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "operation failed");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1); // No retries on error
    }

    #[tokio::test]
    async fn test_custom_retry_config() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 10,
            backoff_multiplier: 2,
        };

        let result: Result<i32, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok((0, true)) // Always retry
                }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_stops_on_success_flag() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result: Result<String, String> = retry_with_backoff(
            || {
                let count = count_clone.clone();
                async move {
                    let attempts = count.fetch_add(1, Ordering::SeqCst);
                    // Stop retrying after 2 attempts even though max is 3
                    if attempts == 1 {
                        Ok(("success".to_string(), false))
                    } else {
                        Ok(("retrying".to_string(), true))
                    }
                }
            },
            &RetryConfig::default(),
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2); // Stopped early
    }
}
