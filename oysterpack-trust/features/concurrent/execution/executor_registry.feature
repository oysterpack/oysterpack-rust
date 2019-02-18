Feature: [01D3W0H2B7KNTBJTGDYP3CRB7K] A global Executor registry is provided.

  - `ExecutorBuilder` is used to register new Executor(s) with the global registry
  - Each Executor is identified by its ExecutorId, which is used as the registry key
  - The following Executor properties are configurable
    - thread pool size
    - thread stack size
    - catching unwinding panics for spawned futures - by default, this is set to true

  Scenario: [01D3W0MDTMRJ6GNFCQCPTS55HG] Registering an Executor
    Given [01D3W0MDTMRJ6GNFCQCPTS55HG-1] a unique ExecutorId
    When [01D3W0MDTMRJ6GNFCQCPTS55HG-2] the Executor is registered
    Then [01D3W0MDTMRJ6GNFCQCPTS55HG-3] the Executor can be retrieved from the registry
    And [01D3W0MDTMRJ6GNFCQCPTS55HG-4] the Executor thread pool size will match the number of cpu cores
    And [01D3W0MDTMRJ6GNFCQCPTS55HG-5] the Executor is configured to catch unwinding panics

  Scenario: [01D40G5CFDP2RS7V75WJQCSME4] Registering an Executor configured with a custom thread pool size
    When [01D40G5CFDP2RS7V75WJQCSME4-1] an Executor is registered with thread pool size = 20
    Then [01D40G5CFDP2RS7V75WJQCSME4-2] the Executor can be retrieved from the registry
    And [01D40G5CFDP2RS7V75WJQCSME4-3] the Executor thread pool size will be 20

  Scenario: [01D40G6X1ABZK6532CVE00EWHW] Registering an Executor configured with a custom thread stack size
    When [01D40G6X1ABZK6532CVE00EWHW-1] an Executor is registered with thread stack size = 64 KB
    Then [01D40G6X1ABZK6532CVE00EWHW-2] Executor::stack_size() returns 64 KB

  Scenario: [01D40G78JNHX519WEP1A1E5FVT] Registering an Executor configured to not catch unwinding panics
    When [01D40G78JNHX519WEP1A1E5FVT-1] an Executor is registered with `catch_unwind` = false
    Then [01D40G78JNHX519WEP1A1E5FVT-2] Executor::catch_unwind() returns false

  Scenario: [01D40G7FQDMWEVGSGFH96KQMZ0] Registering an Executor using an ExecutorId that is already in use
    Given [01D40G7FQDMWEVGSGFH96KQMZ0-1] an Executor is already registered
    When [01D40G7FQDMWEVGSGFH96KQMZ0-2] registering the Executor using an ExecutorId that is already registered
    Then [01D40G7FQDMWEVGSGFH96KQMZ0-3] registration will fail