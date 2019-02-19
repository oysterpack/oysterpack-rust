Feature: [01D418RZF94XJCRQ5D2V4DRMJ6] Executor metrics are collected

  - Number of tasks that the Executor has spawned.
    - M01D2DMYKJSPRG6H419R7ZFXVRH - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID
  - Number of tasks that the Executor has completed
    - 01D39C05YGY6NY3RD18TJ6975H - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID
  - Number of tasks that have panicked - tracked only if the Executor is configured to catch unwinding panics
    - 01D3950A0931ESKR66XG7KMD7Z - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID
  - Executor Thread pool size
    - 01D395423XG3514YP762RYTDJ1 - IntGaugeVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID

  Scenario: [01D41EX3HY16EH06RVHAHE2Q0F] Spawn tasks and verify metrics match
    Given [01D41EX3HY16EH06RVHAHE2Q0F-1] a new Executor
    When [01D41EX3HY16EH06RVHAHE2Q0F-2] 5 normal tasks and 5 panic tasks are spawned
    Then [01D41EX3HY16EH06RVHAHE2Q0F-3] the metrics retrieved from the metric registry will match against the Executor

  Scenario: [01D41GJ0WRB49AX2NX4T09BKA8] Verify total Executor threads against the metric registry
    When [01D41GJ0WRB49AX2NX4T09BKA8-1] a new Executor is registered
    Then [01D41GJ0WRB49AX2NX4T09BKA8-2] the total Executor thread count computed from the metric registry will match `execution::total_thread_count()`

