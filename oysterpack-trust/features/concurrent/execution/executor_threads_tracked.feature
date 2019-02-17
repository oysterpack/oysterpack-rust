Feature: [01D3W3G8A7H32MVG3WYBER6J13] Executor threads are tracked

  - thread pool size
  - number of started threads

  Scenario: [01D3W3GDYVS4P2SR0SECVT0JJT] Register a new Executor with 10 threads
    When [01D3W3GDYVS4P2SR0SECVT0JJT-1] a new Executor is registered with 10 threads
    Then [01D3W3GDYVS4P2SR0SECVT0JJT-2] the Executor thread pool size is 10
    And [01D3W3GDYVS4P2SR0SECVT0JJT-3] and the total number of Executor threads increased by 10

