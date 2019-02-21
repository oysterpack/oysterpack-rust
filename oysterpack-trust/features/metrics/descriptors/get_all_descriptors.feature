Feature: [01D3SF7KGJZZM50TXXW5HX4N99] All metric descriptors can be retrieved from the metric registry.

  Scenario: [01D3SF3R0DTBTVRKC9PFHQEEM9] All metric descriptors are returned
    Given [01D3SF3R0DTBTVRKC9PFHQEEM9] metrics are registered
    When [01D3SF3R0DTBTVRKC9PFHQEEM9] retrieving all descriptors
    Then [01D3SF3R0DTBTVRKC9PFHQEEM9] the metric descriptors match the descriptors that are retrieved directory from collectors