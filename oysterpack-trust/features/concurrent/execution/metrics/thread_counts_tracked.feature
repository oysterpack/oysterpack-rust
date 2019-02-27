Feature: [01D418RZF94XJCRQ5D2V4DRMJ6] Executor thread pool size is recorded as a metric

  - Executor Thread pool size
    - 01D395423XG3514YP762RYTDJ1 - IntGaugeVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID

  Scenario: [01D41GJ0WRB49AX2NX4T09BKA8] Verify total Executor threads match against the metric registry
    Given [01D41GJ0WRB49AX2NX4T09BKA8] multiple Executor(s) are registered
    Then [01D41GJ0WRB49AX2NX4T09BKA8] `execution::total_threads()` count will match the count computed against the metric registry