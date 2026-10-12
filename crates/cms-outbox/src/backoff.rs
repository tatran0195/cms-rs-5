use chrono::Duration;

/// Configuration for exponential backoff calculation with jitter.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Base initial delay in seconds (default: 1s).
    pub initial_seconds: u64,
    /// Multiplier factor per attempt (default: 2.0).
    pub factor: f64,
    /// Maximum backoff duration in seconds (default: 3600s = 1 hour).
    pub max_seconds: u64,
    /// Jitter ratio applied to the calculated delay (default: 0.20 = ±20%).
    pub jitter_ratio: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_seconds: 1,
            factor: 2.0,
            max_seconds: 3600,
            jitter_ratio: 0.20,
        }
    }
}

impl BackoffConfig {
    /// Calculate the backoff duration for a given attempt index using secure random jitter.
    pub fn calculate(&self, attempt: i32) -> Duration {
        let jitter_factor = get_random_jitter_factor();
        self.calculate_deterministic(attempt, jitter_factor)
    }

    /// Calculate the backoff duration deterministically with a supplied jitter factor.
    /// `jitter_factor` must be in the range `[-1.0, 1.0]`.
    pub fn calculate_deterministic(&self, attempt: i32, jitter_factor: f64) -> Duration {
        let attempt = attempt.clamp(0, 30) as f64;
        let base_secs = (self.initial_seconds as f64) * self.factor.powf(attempt);
        let capped_secs = base_secs.min(self.max_seconds as f64);

        // Clamp jitter_factor to [-1.0, 1.0]
        let clamped_factor = jitter_factor.clamp(-1.0, 1.0);
        let jittered_secs = capped_secs * (1.0 + clamped_factor * self.jitter_ratio);

        // Ensure at least 1 second and not exceeding max_seconds * (1 + jitter_ratio)
        let final_secs = jittered_secs.max(1.0).round() as i64;
        Duration::seconds(final_secs)
    }
}

/// Helper function to generate a random jitter factor in `[-1.0, 1.0]`.
fn get_random_jitter_factor() -> f64 {
    let mut buf = [0u8; 4];
    if getrandom::fill(&mut buf).is_err() {
        return 0.0;
    }
    let val = u32::from_le_bytes(buf);
    // Map u32 [0, u32::MAX] to [-1.0, 1.0]
    let normalized = (val as f64) / (u32::MAX as f64);
    (normalized * 2.0) - 1.0
}

/// Convenience function calculating backoff with default configuration.
pub fn calculate_backoff(attempt: i32) -> Duration {
    BackoffConfig::default().calculate(attempt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_monotonicity() {
        let config = BackoffConfig::default();
        let b0 = config.calculate_deterministic(0, 0.0).num_seconds();
        let b1 = config.calculate_deterministic(1, 0.0).num_seconds();
        let b2 = config.calculate_deterministic(2, 0.0).num_seconds();
        let b3 = config.calculate_deterministic(3, 0.0).num_seconds();

        assert_eq!(b0, 1);
        assert_eq!(b1, 2);
        assert_eq!(b2, 4);
        assert_eq!(b3, 8);
    }

    #[test]
    fn test_backoff_cap() {
        let config = BackoffConfig::default();
        let b_high = config.calculate_deterministic(20, 0.0).num_seconds();
        assert_eq!(b_high, 3600);
    }

    #[test]
    fn test_backoff_jitter_bounds() {
        let config = BackoffConfig::default();
        let min_b2 = config.calculate_deterministic(2, -1.0).num_seconds();
        let max_b2 = config.calculate_deterministic(2, 1.0).num_seconds();

        // 4 * (1 - 0.20) = 3.2 -> rounded to 3
        assert_eq!(min_b2, 3);
        // 4 * (1 + 0.20) = 4.8 -> rounded to 5
        assert_eq!(max_b2, 5);
    }

    #[test]
    fn test_backoff_never_zero_or_negative() {
        let config = BackoffConfig::default();
        let b_neg = config.calculate_deterministic(-5, -1.0).num_seconds();
        assert!(b_neg >= 1);
    }
}
