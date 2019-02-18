Feature: [01D3W3G8A7H32MVG3WYBER6J13] Spawned tasks are tracked

  - the total number of spawned tasks
  - the total number of completed tasks
  - the active number of tasks = (spawned task count) - (completed task count)

  Scenario: [01D3Y1CYCKZHY675FKEPPX4JE4] Spawning tasks
    Given [01D3Y1CYCKZHY675FKEPPX4JE4-1] a new Executor
    When [01D3Y1CYCKZHY675FKEPPX4JE4-2] 10 tasks are spawned
    Then [01D3Y1CYCKZHY675FKEPPX4JE4-3] the spawned task count will increase by 10
    And [01D3Y1CYCKZHY675FKEPPX4JE4-4] the completed task count will increase by 10
    And [01D3Y1CYCKZHY675FKEPPX4JE4-5] the active task count will be 0

  Scenario: [01D3Y1D8SJZ8JWPGJKFK4BYHP0] Spawning panicking tasks which catch unwinding panics
    Given [01D3Y1D8SJZ8JWPGJKFK4BYHP0-1] a new Executor
    When [01D3Y1D8SJZ8JWPGJKFK4BYHP0-2] 10 tasks are spawned - 5 of which will panic
    Then [01D3Y1D8SJZ8JWPGJKFK4BYHP0-3] the spawned task count will increase by 10
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-4] the completed task count will increase by 10
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-5] the active task count will be 0
    And [01D3Y1D8SJZ8JWPGJKFK4BYHP0-6] the panicked task count will increase by 5

