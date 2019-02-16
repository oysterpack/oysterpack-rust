Feature: [01D3W1C9YZDYMDPT98JCFS8F4P] The list of registered ExecutorId(s) can be retrieved from the Executor registry

  Scenario: [01D3W1NYG4YT4MM5HDR4YWT7ZD] Registering 2 Executor(s)
    Given [01D3W1NYG4YT4MM5HDR4YWT7ZD-1] 2 Executors are registered
    When [01D3W1NYG4YT4MM5HDR4YWT7ZD-2] registered ExecutorId(s) are retrieved from the registry
    Then [01D3W1NYG4YT4MM5HDR4YWT7ZD-3] the expected ExecutorId(s) are returned