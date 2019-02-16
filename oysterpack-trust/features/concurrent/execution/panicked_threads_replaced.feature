Feature: [01D3W3984C244Q6J9NZ28CVPAC] Spawned tasks that panic will not cause threads in the Thread pool to die

  Scenario: [01D3W3C3W9WH25N7ZG0KZSEYS5] A spawned task panics
    When [01D3W3C3W9WH25N7ZG0KZSEYS5-1] a task is spawned which panics
    Then [01D3W3C3W9WH25N7ZG0KZSEYS5-2] the thread pool size is not affected