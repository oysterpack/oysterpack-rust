Feature: [01D3W2SQX48THY42D6X09P2KK9] Thread pool sizes for each registered Executor is tracked

  Scenario: [01D3W2TZK5WZVXB0FAMEZ61X4A] Register an Executor with 10 threads
    When [01D3W2TZK5WZVXB0FAMEZ61X4A-1] an Executor is registered with 10 threads
    Then [01D3W2TZK5WZVXB0FAMEZ61X4A-2] when Executor thread pool size is retrieved
    And [01D3W2RF94W85YGQ49JFDXB3XB-3] the Executor's thread pool size will be 10