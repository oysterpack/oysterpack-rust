Feature: [01D3W2RTE80P64E1W1TD61KGBN] A global Executor will be automatically provided by the Executor registry

  The global Executor's thread pool size will match the number of cpu cores.

  Scenario: [01D3W2RF94W85YGQ49JFDXB3XB] Spawn 10 tasks on 10 threads using the global Executor
    Then [01D3W2RF94W85YGQ49JFDXB3XB] the executor completed task count will increase by 100

  Scenario: [01D4P2Z3JWR05CND2N96TMBKT2] Spawn 10 tasks on 10 threads using an Executor
    Then [01D4P2Z3JWR05CND2N96TMBKT2] the executor completed task count will increase by 100