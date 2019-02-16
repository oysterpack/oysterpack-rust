Feature: [01D3W0H2B7KNTBJTGDYP3CRB7K] A global Executor registry is provided.

  Executor(s) are registered using ExecutorId as the registry key.

  Scenario: [01D3W0MDTMRJ6GNFCQCPTS55HG] Registering an Executor with a default ThreadPoolBuilder
    Given [01D3W0MDTMRJ6GNFCQCPTS55HG-1] a unique ExecutorId and a default ThreadPoolBuilder
    When [01D3W0MDTMRJ6GNFCQCPTS55HG-2] the Executor is registered
    Then [01D3W0MDTMRJ6GNFCQCPTS55HG-3] the Executor can be retrieved from the registry
    And [01D3W0MDTMRJ6GNFCQCPTS55HG-4] the Executor thread pool size will match the number of cpu cores

  Scenario: [01D3W1NKKK36EQZFH7AD3SCHGC] Registering an Executor with ThreadPoolBuilder configured with a custom thread pool size
    Given [01D3W1NKKK36EQZFH7AD3SCHGC-1] a unique ExecutorId and a ThreadPoolBuilder configured for a pool size of 2x the number of cpu cores
    When [01D3W1NKKK36EQZFH7AD3SCHGC-2] the Executor is registered
    Then [01D3W1NKKK36EQZFH7AD3SCHGC-3] the Executor can be retrieved from the registry
    And [01D3W1NKKK36EQZFH7AD3SCHGC-4] the Executor thread pool size will match what's expected

  Scenario: [01D3W0VD8FQZB7H1Q5705D84X6] Registering an Executor using an ExecutorId that is already in use
    Given [01D3W0VD8FQZB7H1Q5705D84X6-1] an Executor is already registered
    When [01D3W0VD8FQZB7H1Q5705D84X6-2] registering the Executor using an ExecutorId that is already registered
    Then [01D3W0VD8FQZB7H1Q5705D84X6-3] registration will fail