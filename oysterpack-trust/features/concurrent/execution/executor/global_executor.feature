Feature: [01D3W2RTE80P64E1W1TD61KGBN] A global Executor will be automatically provided by the Executor registry

  The global Executor's thread pool size will match the number of cpu cores.

  Scenario: [01D3W2RF94W85YGQ49JFDXB3XB] Use the global Executor from 2 different threads
    Given [01D3W2RF94W85YGQ49JFDXB3XB-1] the global Executor
    When [01D3W2RF94W85YGQ49JFDXB3XB-2] spawn 10 tasks from each thread
    Then [01D3W2RF94W85YGQ49JFDXB3XB-3] the executor completed task count will increase by 20