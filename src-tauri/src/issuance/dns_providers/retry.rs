use anyhow::Result;
use log::{debug, warn};
use std::time::{Duration, Instant};

use crate::issuance::dns::{DnsPropagationResult, PropagationState};
use crate::issuance::dns_providers::query_google_dns;

/// Retries checking DNS propagation via public DNS (DoH) until the record is found
/// or timeout is reached. This is used for both testing and issuance flows.
///
/// Returns the final propagation result. The caller should check if `state == PropagationState::Found`.
pub fn poll_dns_propagation(
    record_name: &str,
    expected_value: &str,
    timeout: Duration,
    interval: Duration,
) -> Result<DnsPropagationResult> {
    let started = Instant::now();
    let mut attempt = 0;

    loop {
        attempt += 1;
        debug!(
            "[dns-retry] Propagation check attempt {} for {}",
            attempt, record_name
        );

        let result = query_google_dns(record_name, expected_value)?;

        match result.state {
            PropagationState::Found => {
                debug!(
                    "[dns-retry] Record found after {}ms (attempt {})",
                    started.elapsed().as_millis(),
                    attempt
                );
                return Ok(result);
            }
            PropagationState::WrongContent => {
                // Wrong value - don't retry, fail immediately
                warn!(
                    "[dns-retry] TXT record at {} has wrong value. Expected: {}. Observed: {:?}",
                    record_name, expected_value, result.observed_values
                );
                return Ok(result);
            }
            PropagationState::NxDomain | PropagationState::Pending | PropagationState::Error => {
                // Record not found yet or error - retry if we have time
                if started.elapsed() >= timeout {
                    warn!(
                        "[dns-retry] Propagation timeout after {}ms (attempt {}), last state: {:?}",
                        started.elapsed().as_millis(),
                        attempt,
                        result.state
                    );
                    return Ok(result);
                }

                let elapsed = started.elapsed();
                let remaining = timeout.saturating_sub(elapsed);
                debug!(
                    "[dns-retry] Record not found yet (state={:?}), waiting {}s before next check ({}s remaining)",
                    result.state, interval.as_secs(), remaining.as_secs()
                );
                std::thread::sleep(interval);
                continue;
            }
        }
    }
}

/// Generic retry utility for provider-level verification (checking via provider API, not public DNS).
/// This retries a closure until it returns Ok(true) or timeout is reached.
///
/// The closure should return:
/// - `Ok(true)` if verification succeeded
/// - `Ok(false)` if verification should be retried
/// - `Err(_)` if verification should fail immediately (no retry)
pub fn retry_provider_verification<F>(
    record_name: &str,
    operation: &str,
    timeout: Duration,
    interval: Duration,
    mut verify_fn: F,
) -> Result<()>
where
    F: FnMut() -> Result<bool>,
{
    let started = Instant::now();
    let mut attempt = 0;

    loop {
        attempt += 1;
        debug!(
            "[provider-retry] {} verification attempt {} for {}",
            operation, attempt, record_name
        );

        match verify_fn() {
            Ok(true) => {
                debug!(
                    "[provider-retry] {} verification succeeded after {}ms (attempt {})",
                    operation,
                    started.elapsed().as_millis(),
                    attempt
                );
                return Ok(());
            }
            Ok(false) => {
                // Retry condition
                if started.elapsed() >= timeout {
                    warn!(
                        "[provider-retry] {} verification timeout after {}ms (attempt {})",
                        operation,
                        started.elapsed().as_millis(),
                        attempt
                    );
                    return Err(anyhow::anyhow!(
                        "{} verification failed for {}: timeout after {}ms",
                        operation,
                        record_name,
                        started.elapsed().as_millis()
                    ));
                }

                let elapsed = started.elapsed();
                let remaining = timeout.saturating_sub(elapsed);
                debug!(
                    "[provider-retry] {} not yet verified, waiting {}ms before next check ({}ms remaining)",
                    operation, interval.as_millis(), remaining.as_millis()
                );
                std::thread::sleep(interval);
                continue;
            }
            Err(e) => {
                // Immediate failure
                warn!(
                    "[provider-retry] {} verification failed immediately: {}",
                    operation, e
                );
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn retry_provider_verification_succeeds_immediately() {
        let calls = Arc::new(Mutex::new(0));
        let calls_clone = calls.clone();
        let result = retry_provider_verification(
            "test.example.com",
            "test",
            Duration::from_secs(5),
            Duration::from_millis(100),
            move || {
                *calls_clone.lock().unwrap() += 1;
                Ok(true)
            },
        );
        assert!(result.is_ok());
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn retry_provider_verification_retries_until_success() {
        let calls = Arc::new(Mutex::new(0));
        let calls_clone = calls.clone();
        let result = retry_provider_verification(
            "test.example.com",
            "test",
            Duration::from_secs(2),
            Duration::from_millis(50),
            move || {
                let mut count = calls_clone.lock().unwrap();
                *count += 1;
                Ok(*count >= 3)
            },
        );
        assert!(result.is_ok());
        assert_eq!(*calls.lock().unwrap(), 3);
    }

    #[test]
    fn retry_provider_verification_fails_on_timeout() {
        let result = retry_provider_verification(
            "test.example.com",
            "test",
            Duration::from_millis(100),
            Duration::from_millis(20),
            || Ok(false),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn retry_provider_verification_fails_immediately_on_error() {
        let calls = Arc::new(Mutex::new(0));
        let calls_clone = calls.clone();
        let result = retry_provider_verification(
            "test.example.com",
            "test",
            Duration::from_secs(5),
            Duration::from_millis(100),
            move || {
                *calls_clone.lock().unwrap() += 1;
                Err(anyhow::anyhow!("immediate failure"))
            },
        );
        assert!(result.is_err());
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}
