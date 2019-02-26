Feature: [01D3XX3ZBB7VW0GGRA60PMFC1M] Helper functions for collecting timer based metrics

  - pub fn time<F>(clock: &Clock, f: F) -> u64 where F: FnOnce()
  - pub fn time_with_result<F, T>(clock: &quanta::Clock, f: F) -> (u64, T) where F: FnOnce() -> T
  - pub fn as_float_secs(nanos: u64) -> f64
    - in prometheus, it is a common practice to report timer metrics in secs

  Scenario: [01D3XX46RZ63QYR0AAWVBCHWGP] Timing a function that sleeps for 1 ms
    Then [01D3XX46RZ63QYR0AAWVBCHWGP] the time returned should be ~0.001 sec

  Scenario: [01D3XZ6GCY1ECSKMBC6870ZBS0] Timing a function that sleeps for 1 ms and returns a result
    Then [01D3XZ6GCY1ECSKMBC6870ZBS0] the time returned should be ~0.001 sec and the result is returned

