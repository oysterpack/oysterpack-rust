Feature: [01D3W3G8A7H32MVG3WYBER6J13] Spawned tasks are tracked via metrics

  Metrics
  - spawned task count
    - M01D2DMYKJSPRG6H419R7ZFXVRH - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID
  - completed task count
    - 01D39C05YGY6NY3RD18TJ6975H - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID
  - panicked task count
    - 01D3950A0931ESKR66XG7KMD7Z - IntCounterVec
    - Labels: L01D2DN1VBMW6XC7EQ971PBGW68 -> ExecutorId ULID

  - active task count = (spawned task count) - (completed task count)

  Scenario: [01D3Y1D8SJZ8JWPGJKFK4BYHP0] Spawning tasks
    When [01D3Y1D8SJZ8JWPGJKFK4BYHP0] 5 normal tasks and 3 panic tasks are spawned
    Then [01D3Y1D8SJZ8JWPGJKFK4BYHP0-1] the spawned task count will increase by 8
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-2] the completed task count will increase by 8
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-3] the active task count will be 0
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-4] the panicked task count will increase by 3
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-5] metrics match executor counts

