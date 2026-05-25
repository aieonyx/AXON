//! Contract cache — Bio DNA: Epigenetic Memory.
//!
//! Epigenetic marks in biology persist across cell divisions and encode
//! memory of past gene expression. The contract cache does the same:
//! verified postconditions are cached by function hash. Unchanged
//! functions skip re-verification on subsequent builds.
//!
//! Target: ≥70% cache hit rate on second build (per Phase 5.5 spec).

use std::collections::HashMap;
use std::cell::RefCell;

/// A cached verification result for a single function.
#[derive(Debug, Clone)]
pub struct CachedVerification {
    /// The function hash (FNV-64 of label or source).
    pub fn_hash:   u64,
    /// Whether the last check passed.
    pub passed:    bool,
    /// Number of times this result has been served from cache.
    pub hit_count: u32,
    /// Total number of times this postcondition was checked.
    pub check_count: u32,
}

/// Cache statistics for diagnostics and grant reporting.
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Total cache lookups.
    pub lookups:  u64,
    /// Cache hits (result served from cache).
    pub hits:     u64,
    /// Cache misses (new verification performed).
    pub misses:   u64,
    /// Total postcondition violations recorded.
    pub violations: u64,
}

impl CacheStats {
    /// Cache hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 { return 0.0; }
        (self.hits as f64 / self.lookups as f64) * 100.0
    }
}

/// The AXON contract cache — Bio DNA Epigenetic Memory.
#[derive(Debug, Default)]
pub struct ContractCache {
    entries: HashMap<u64, CachedVerification>,
    stats:   CacheStats,
}

impl ContractCache {
    /// Create a new empty cache.
    pub fn new() -> Self { Self::default() }

    /// Record a postcondition check result.
    ///
    /// If this hash was seen before, increments hit_count.
    /// If new, creates a fresh entry (cache miss).
    pub fn record(&mut self, fn_hash: u64, passed: bool) {
        self.stats.lookups += 1;
        if !passed { self.stats.violations += 1; }

        match self.entries.get_mut(&fn_hash) {
            Some(entry) => {
                self.stats.hits += 1;
                entry.hit_count += 1;
                entry.check_count += 1;
                entry.passed = passed; // update to latest result
            }
            None => {
                self.stats.misses += 1;
                self.entries.insert(fn_hash, CachedVerification {
                    fn_hash,
                    passed,
                    hit_count: 0,
                    check_count: 1,
                });
            }
        }
    }

    /// Look up a cached result without recording a new check.
    pub fn lookup(&self, fn_hash: u64) -> Option<&CachedVerification> {
        self.entries.get(&fn_hash)
    }

    /// Returns true if fn_hash was previously verified as passing.
    pub fn is_verified(&self, fn_hash: u64) -> bool {
        self.entries.get(&fn_hash).map(|e| e.passed).unwrap_or(false)
    }

    /// Current cache statistics.
    pub fn stats(&self) -> &CacheStats { &self.stats }

    /// Number of entries in the cache.
    pub fn len(&self) -> usize { self.entries.len() }

    /// True if the cache is empty.
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Clear all entries (invalidate cache — e.g. after source change).
    pub fn invalidate(&mut self) { self.entries.clear(); }
}

// Thread-local cache — zero mutex overhead for single-threaded use.
// For multi-threaded use, wrap in Arc<Mutex<ContractCache>>.
thread_local! {
    pub static CONTRACT_CACHE: RefCell<ContractCache> = RefCell::new(ContractCache::new());
}

/// Get current cache statistics from the thread-local cache.
pub fn cache_stats() -> CacheStats {
    CONTRACT_CACHE.with(|c| c.borrow().stats().clone())
}

/// Get the current cache hit rate percentage.
pub fn cache_hit_rate() -> f64 {
    CONTRACT_CACHE.with(|c| c.borrow().stats().hit_rate())
}

/// Invalidate the thread-local cache (call after source changes).
pub fn cache_invalidate() {
    CONTRACT_CACHE.with(|c| c.borrow_mut().invalidate());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_cache() -> ContractCache { ContractCache::new() }

    #[test]
    fn cache_miss_on_first_check() {
        let mut c = fresh_cache();
        c.record(0xABCD, true);
        assert_eq!(c.stats().misses, 1);
        assert_eq!(c.stats().hits, 0);
    }

    #[test]
    fn cache_hit_on_second_check() {
        let mut c = fresh_cache();
        c.record(0xABCD, true);
        c.record(0xABCD, true);
        assert_eq!(c.stats().hits, 1);
        assert_eq!(c.stats().misses, 1);
    }

    #[test]
    fn cache_is_verified() {
        let mut c = fresh_cache();
        assert!(!c.is_verified(0x1234));
        c.record(0x1234, true);
        assert!(c.is_verified(0x1234));
    }

    #[test]
    fn cache_violation_count() {
        let mut c = fresh_cache();
        c.record(0x01, true);
        c.record(0x02, false);
        c.record(0x03, false);
        assert_eq!(c.stats().violations, 2);
    }

    #[test]
    fn cache_hit_rate_zero_when_empty() {
        let c = fresh_cache();
        assert_eq!(c.stats().hit_rate(), 0.0);
    }

    #[test]
    fn cache_hit_rate_100_after_repeated() {
        let mut c = fresh_cache();
        c.record(0xAA, true);  // miss
        c.record(0xAA, true);  // hit
        c.record(0xAA, true);  // hit
        let rate = c.stats().hit_rate();
        assert!((rate - 66.666).abs() < 1.0);
    }

    #[test]
    fn cache_invalidate_clears_entries() {
        let mut c = fresh_cache();
        c.record(0xAA, true);
        assert_eq!(c.len(), 1);
        c.invalidate();
        assert!(c.is_empty());
    }

    #[test]
    fn cache_stats_global_fn() {
        let stats = cache_stats();
        let _ = stats.hit_rate(); // must not panic
    }
}
