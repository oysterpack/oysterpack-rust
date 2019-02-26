Feature: [01D3YVY445KA4YF5KYMHHQK2TP] Executors are configured to catch unwinding panics for spawned futures

  Scenario: [01D3YW91CYQRB0XVAKF580WX04] Spawning tasks after spawning tasks that panic on the global executor
    Given [01D3YW91CYQRB0XVAKF580WX04] panicking tasks are spawned that are twice the number of threads in the global Executor
    When [01D3YW91CYQRB0XVAKF580WX04] normal tasks are spawned
    Then [01D3YW91CYQRB0XVAKF580WX04] the tasks continue to be processed successfully
