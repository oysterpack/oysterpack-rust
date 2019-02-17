Feature: [01D3YVY445KA4YF5KYMHHQK2TP] Executors can be configured to catch unwinding panics for wrap spawned futures

  - The global executor is automatically configured to catch unwinding panics.
  - If an unwinding panic is not caught, then it will kill the Executor thread. The thread will not be replaced, which
    overtime will lead to a broken Executor - there will be no threads to execute futures.

  Scenario: [01D3YW91CYQRB0XVAKF580WX04] Spawning tasks after spawning tasks that panic on the global executor
    Given [01D3YW91CYQRB0XVAKF580WX04-1] panicking tasks are spawned that are twice the number of threads in the global Executor
    When [01D3YW91CYQRB0XVAKF580WX04-2] when normal tasks are spawned
    Then [01D3YW91CYQRB0XVAKF580WX04-3] then the tasks continue to be processed successfully
    And [01D3YW91CYQRB0XVAKF580WX04-4] the Executor's panicked task count increased accordingly

