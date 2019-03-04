Feature: [01D3XX3ZBB7VW0GGRA60PMFC1M] Time conversion functions to report timings in seconds as f64

  - In prometheus, it is a common practice to report timer metrics in secs. The following utility functions are provided
    to convert a time into seconds as f64
    - `pub fn as_float_secs(nanos: u64) -> f64`
    - `pub fn duration_as_float_secs(duration: Duration) -> f64`

  Scenario: [01D3XX46RZ63QYR0AAWVBCHWGP] Convert 1_000_000 ns into a sec
    Then [01D3XX46RZ63QYR0AAWVBCHWGP] the time returned should be 0.001 sec

  Scenario: [01D3XZ6GCY1ECSKMBC6870ZBS0] Convert a Duration into secs
    Then [01D3XZ6GCY1ECSKMBC6870ZBS0] Duration::from_millis(1) reults in 0.001 sec

