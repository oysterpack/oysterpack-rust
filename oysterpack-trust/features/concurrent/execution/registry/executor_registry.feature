Feature: [01D3W0H2B7KNTBJTGDYP3CRB7K] A global Executor registry is provided.

  - `ExecutorBuilder` is used to construct and register new Executor(s) with the global registry
  - Each Executor is identified by its ExecutorId, which is used as the registry key
  - The following Executor properties are configurable
    - thread pool size - default = number of cpu cores
    - thread stack size - default = Rust default

  Scenario: [01D3W0MDTMRJ6GNFCQCPTS55HG] Registering an Executor with default settings
    Then [01D3W0MDTMRJ6GNFCQCPTS55HG-1] the Executor defaults are applied
    And [01D3W0MDTMRJ6GNFCQCPTS55HG-2] metrics are initialized

  Scenario: [01D40G5CFDP2RS7V75WJQCSME4] Registering an Executor configured with thread pool size = 20
    Then [01D40G5CFDP2RS7V75WJQCSME4] the Executor::thread_pool_size() returns 20

  Scenario: [01D40G6X1ABZK6532CVE00EWHW] Registering an Executor configured with a custom thread stack size
    Then [01D40G6X1ABZK6532CVE00EWHW] Executor::stack_size() returns 64 KB

  Scenario: [01D40G7FQDMWEVGSGFH96KQMZ0] Registering an Executor using the global ExecutorId
    Then [01D40G7FQDMWEVGSGFH96KQMZ0] registration will fail

  Scenario: [01D40WTESDPHA8BZVM2VS7VRK2] Registering an Executor using an ExecutorId that is already in use
    Then [01D40WTESDPHA8BZVM2VS7VRK2] registration will fail