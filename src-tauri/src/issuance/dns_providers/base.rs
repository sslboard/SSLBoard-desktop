use anyhow::{Result, anyhow};
use log::{debug, warn};
use std::thread;
use std::time::{Duration, Instant};

use crate::issuance::dns::{DnsPropagationResult, PropagationState};

use super::testing;

// Constants for DNS operations
const MAX_RECORD_CREATE_RETRIES: u32 = 3;
const RECORD_CREATE_RETRY_DELAY: Duration = Duration::from_millis(500);

/// Executes an operation with retry logic.
/// Retries up to max_attempts times with exponential backoff.
/// Returns the result of the first successful operation, or the last error.
fn retry_with_backoff<T, F>(mut operation: F, max_attempts: u32, delay: Duration) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    for attempt in 0..max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_attempts - 1 {
                    return Err(e);
                }
                debug!(
                    "[dns-retry] Operation failed (attempt {}), retrying in {}ms: {}",
                    attempt + 1,
                    delay.as_millis(),
                    e
                );
                thread::sleep(delay);
            }
        }
    }
    unreachable!()
}

/// Represents a DNS record returned from listing operations.
#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub value: String,
}

/// Trait for atomic DNS operations that each provider must implement.
/// These are low-level operations that interact directly with the provider's API.
pub trait AtomicDnsOperations: Send + Sync {
    /// Creates a single TXT record and returns its ID.
    /// The implementation should handle zone discovery internally if needed.
    fn create_one_record(&mut self, record_name: &str, value: &str) -> Result<String>;

    /// Deletes a single TXT record by its ID.
    fn delete_one_record(&mut self, record_id: &str) -> Result<()>;

    /// Lists all TXT records matching the given record name.
    /// Returns a vector of records with their IDs, names, and values.
    fn list_records(&mut self, record_name: &str) -> Result<Vec<DnsRecord>>;

    /// Gets the zone ID for a given domain.
    /// Implementations should cache this value to avoid repeated API calls.
    fn get_zone_id(&mut self, domain: &str) -> Result<String>;

    /// Normalizes a TXT record value for comparison.
    /// Each provider may format values differently (with/without quotes, etc.),
    /// so this method should normalize the value to a canonical form for comparison.
    /// The default implementation trims whitespace and removes quotes.
    fn normalize_value(&self, value: &str) -> String {
        // Default: trim whitespace and quotes (for Google DNS responses)
        value.trim().trim_matches('"').trim().to_string()
    }
}

/// Base trait for DNS providers with high-level operations.
/// Provides default implementations that use AtomicDnsOperations and handle
/// retrying, parallelization, and DNS testing.
pub trait DnsProviderBase: Send + Sync {
    /// Returns a mutable reference to the atomic operations implementation.
    fn atomic_ops(&mut self) -> &mut dyn AtomicDnsOperations;

    /// Sets a single TXT record with retry logic.
    /// This is a convenience method that calls set_txt_records with a single record.
    fn set_txt_record(&mut self, record_name: &str, value: &str) -> Result<()> {
        self.set_txt_records(vec![(record_name.to_string(), value.to_string())])
    }

    /// Sets multiple TXT records in parallel with retry logic.
    /// Each record is created independently, and failures are collected.
    /// Note: For now, this processes records sequentially. Implementations can override
    /// this method to provide true parallelization if they can clone their state.
    fn set_txt_records(&mut self, records: Vec<(String, String)>) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        if records.len() == 1 {
            // Single record - no need for parallelization
            let (record_name, value) = &records[0];
            return self.set_txt_record_with_retry(record_name, value);
        }

        // Multiple records - process sequentially for now
        // Implementations can override this to parallelize if they support cloning
        let mut errors = Vec::new();
        for (record_name, value) in records {
            if let Err(e) = self.set_txt_record_with_retry(&record_name, &value) {
                errors.push((record_name, e));
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!(
                "Failed to set {} record(s): {}",
                errors.len(),
                errors[0].1
            ));
        }

        Ok(())
    }

    /// Sets a single TXT record with retry logic and existence checking.
    /// This is the internal method that handles retries and checks for existing records.
    fn set_txt_record_with_retry(&mut self, record_name: &str, value: &str) -> Result<()> {
        // Check if record already exists with correct value
        let normalized_value = self.atomic_ops().normalize_value(value);
        match self.atomic_ops().list_records(record_name) {
            Ok(existing) => {
                for record in existing {
                    let normalized_existing = self.atomic_ops().normalize_value(&record.value);
                    if normalized_existing == normalized_value {
                        // Record with correct value already exists
                        debug!(
                            "[dns-base] Record {} already exists with correct value",
                            record_name
                        );
                        return Ok(());
                    }
                }
            }
            Err(_) => {
                // If listing fails, continue to create
                debug!("[dns-base] Failed to list existing records, proceeding with create");
            }
        }

        // Create the record with retry logic
        retry_with_backoff(
            || self.atomic_ops().create_one_record(record_name, value).map(|_| ()),
            MAX_RECORD_CREATE_RETRIES,
            RECORD_CREATE_RETRY_DELAY,
        )
    }

    /// Tests if a TXT record is visible via Google DNS with retry logic.
    /// Uses Google DNS HTTPS API to check for record propagation.
    /// Uses provider-specific normalization for value comparison.
    fn test_txt_record(
        &mut self,
        record_name: &str,
        expected_value: &str,
        timeout: Duration,
        interval: Duration,
    ) -> Result<DnsPropagationResult> {
        let started = Instant::now();
        let mut attempt = 0;
        // Normalize expected value using provider-specific logic
        let normalized_expected = self.atomic_ops().normalize_value(expected_value);

        loop {
            attempt += 1;
            debug!(
                "[dns-test] Propagation check attempt {} for {}",
                attempt, record_name
            );

            match testing::query_google_dns_with_normalization(
                record_name,
                &normalized_expected,
                self.atomic_ops(),
            ) {
                Ok(result) => match result.state {
                    PropagationState::Found => {
                        debug!(
                            "[dns-test] Record found after {}ms (attempt {})",
                            started.elapsed().as_millis(),
                            attempt
                        );
                        return Ok(result);
                    }
                    PropagationState::WrongContent => {
                        warn!(
                            "[dns-test] TXT record at {} has wrong value. Expected: {}. Observed: {:?}",
                            record_name, expected_value, result.observed_values
                        );
                        return Ok(result);
                    }
                    PropagationState::NxDomain
                    | PropagationState::Pending
                    | PropagationState::Error => {
                        if started.elapsed() >= timeout {
                            warn!(
                                "[dns-test] Propagation timeout after {}ms (attempt {}), last state: {:?}",
                                started.elapsed().as_millis(),
                                attempt,
                                result.state
                            );
                            return Ok(result);
                        }

                        let elapsed = started.elapsed();
                        let remaining = timeout.saturating_sub(elapsed);
                        debug!(
                            "[dns-test] Record not found yet (state={:?}), waiting {}s before next check ({}s remaining)",
                            result.state,
                            interval.as_secs(),
                            remaining.as_secs()
                        );
                        thread::sleep(interval);
                        continue;
                    }
                },
                Err(e) => {
                    if started.elapsed() >= timeout {
                        warn!(
                            "[dns-test] Propagation check failed after {}ms (attempt {}): {}",
                            started.elapsed().as_millis(),
                            attempt,
                            e
                        );
                        return Err(e);
                    }
                    debug!(
                        "[dns-test] DNS query failed (attempt {}), retrying in {}s: {}",
                        attempt,
                        interval.as_secs(),
                        e
                    );
                    thread::sleep(interval);
                }
            }
        }
    }

    /// Deletes multiple TXT records.
    /// First lists all records matching the record names, then deletes them in parallel.
    fn delete_txt_records(&mut self, record_names: Vec<String>) -> Result<()> {
        if record_names.is_empty() {
            return Ok(());
        }

        // Collect all record IDs to delete
        let mut records_to_delete = Vec::new();
        for record_name in &record_names {
            match self.atomic_ops().list_records(record_name) {
                Ok(records) => {
                    for record in records {
                        records_to_delete.push((record_name.clone(), record.id));
                    }
                }
                Err(e) => {
                    warn!(
                        "[dns-cleanup] Failed to list records for {}: {}",
                        record_name, e
                    );
                    // Continue with other records
                }
            }
        }

        if records_to_delete.is_empty() {
            return Ok(());
        }

        // Delete records sequentially
        // Note: Parallel deletion would require cloning adapter state, which is complex
        // For now, we delete sequentially. Implementations can override this method
        // if they can provide a way to clone their state for parallel operations.
        let mut errors = Vec::new();
        for (record_name, record_id) in records_to_delete {
            match self.atomic_ops().delete_one_record(&record_id) {
                Ok(()) => {}
                Err(e) => {
                    errors.push((record_name, record_id, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!(
                "Failed to delete {} record(s): {}",
                errors.len(),
                errors[0].2
            ));
        }

        Ok(())
    }

    /// Deletes a single TXT record by name.
    /// This is a convenience method that calls delete_txt_records with a single name.
    fn delete_txt_record(&mut self, record_name: &str) -> Result<()> {
        self.delete_txt_records(vec![record_name.to_string()])
    }
}
