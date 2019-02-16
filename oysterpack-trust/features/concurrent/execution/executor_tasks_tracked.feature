Feature: [01D3W3G8A7H32MVG3WYBER6J13] Spawned tasks are tracked

  - the total number of spawned tasks
  - the total number of completed tasks
  - the active number of tasks = (spawned task count) - (completed task count)

  Scenario: [01D3W3GDYVS4P2SR0SECVT0JJT] Spawn tasks on the global Executor
    Given [01D3W3GDYVS4P2SR0SECVT0JJT-1] the starting number of spawned tasks
    When [01D3W3GDYVS4P2SR0SECVT0JJT-2] 10 tasks are spawned
    Then [01D3W3GDYVS4P2SR0SECVT0JJT-3] the spawned task count will increase by 10
    And [01D3W3GDYVS4P2SR0SECVT0JJT-4] the completed task count will increase by 10
    And [01D3W3GDYVS4P2SR0SECVT0JJT-5] the active task count will be 0

