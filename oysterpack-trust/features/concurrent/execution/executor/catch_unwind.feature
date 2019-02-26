Feature: [01D3YVY445KA4YF5KYMHHQK2TP] Executors can be configured to catch unwinding panics for wrap spawned futures

  - The global executor is automatically configured to catch unwinding panics.
  - If an unwinding panic is not caught, then it will kill the Executor thread. The thread will not be replaced, which
    overtime will lead to a broken Executor - there will be no threads to execute futures.

  Scenario: [01D3YW91CYQRB0XVAKF580WX04] Spawning tasks after spawning tasks that panic on the global executor
    Given [01D3YW91CYQRB0XVAKF580WX04] panicking tasks are spawned that are twice the number of threads in the global Executor
    When [01D3YW91CYQRB0XVAKF580WX04] normal tasks are spawned
    Then [01D3YW91CYQRB0XVAKF580WX04] the tasks continue to be processed successfully

  Scenario: [01D4KN8W6290KB5574RBNPYXN3] Tasks panic on an Executor that is not configured to catch unwinding panics
    Given [01D4KN8W6290KB5574RBNPYXN3] panicking tasks are spawned that match the number Executor thread pool size
    When [01D4KN8W6290KB5574RBNPYXN3] a task is spawned
    Then [01D4KN8W6290KB5574RBNPYXN3] the task will never get picked up to process because all threads in the pool have died

Feature: [01D4KNDR8DYRG4ABBYM469DP52] Executors can be configured with a panic handler channel.

  - This only applies if the Executor is configured to catch unwinding panics.
  - The same executor is used to deliver the panic error on the channel, i.e., the executor will spawn a task
    to send the panic error on the channel
  - When a panic is caught, the cause of the panic is sent async on the error channel

  Scenario: [01D4KNQM13Y5CEGAP9WTSQ7PJY] An Executor is configured with a panic handler channel
    When [01D4KNQM13Y5CEGAP9WTSQ7PJY] a task panics
    Then [01D4KNQM13Y5CEGAP9WTSQ7PJY] the error is sent on the panic channel

