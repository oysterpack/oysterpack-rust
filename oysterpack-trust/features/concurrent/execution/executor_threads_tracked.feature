Feature: [01D3W3G8A7H32MVG3WYBER6J13] Executor threads are tracked

  - thread pool size
  - number of started threads

  Scenario: [01D3W3GDYVS4P2SR0SECVT0JJT] Spawning tasks that panic
    Given [01D3W3GDYVS4P2SR0SECVT0JJT-1] an Executor
    When [01D3W3GDYVS4P2SR0SECVT0JJT-2] when a task is spawned which panics
    Then [01D3W3GDYVS4P2SR0SECVT0JJT-3] then the thread pool size remains the same
    And [01D3W3GDYVS4P2SR0SECVT0JJT-4] the number of started threads will remain the same, i.e., the thread the task panicked on did not die
    And [01D3W3GDYVS4P2SR0SECVT0JJT-5] number of completed tasks will not have increased

