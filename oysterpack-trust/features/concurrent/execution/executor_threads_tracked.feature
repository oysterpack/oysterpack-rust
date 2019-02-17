Feature: [01D3W3G8A7H32MVG3WYBER6J13] Executor threads are tracked

  - thread pool size
  - number of started threads

  If a Future task panics, then the thread will die and not be replaced. Eventually the thread pool will be dead and not
  able to execute any futures. To prevent this scenario and isolate panicking tasks, you have 2 options:

  1. Catch unwinding panics by wrapping the future in a CatchUnwind future - see FutureExt::catch_unwind()
  2. Configure the Executor to catch unwinding panics for all futures

  Scenario: [01D3W3GDYVS4P2SR0SECVT0JJT] Spawning tasks that panic
    Given [01D3W3GDYVS4P2SR0SECVT0JJT-1] an Executor
    When [01D3W3GDYVS4P2SR0SECVT0JJT-2] when a task is spawned which panics
    Then [01D3W3GDYVS4P2SR0SECVT0JJT-3] then the thread pool size did not change
    And [01D3W3GDYVS4P2SR0SECVT0JJT-4] the number of started threads did not change
    And [01D3W3GDYVS4P2SR0SECVT0JJT-5] number of completed tasks have increased

