Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] All registered collectors can be retrieved from the registry

  Scenario: [01D3PSPRYX7XHSGX0JFC8TT59H] Retrieve all registered collectors
    Given [01D3PSPRYX7XHSGX0JFC8TT59H] collectors are registered
    When [01D3PSPRYX7XHSGX0JFC8TT59H] all metric collectors are retrieved
    Then [01D3PSPRYX7XHSGX0JFC8TT59H-1] all collector descriptors should match the descriptors retrieved from the metric registry
    And [01D3PSPRYX7XHSGX0JFC8TT59H-2] registry collector count matches the number of collectors returned

  Scenario: [01D3PX3BGCMV2PS6FDXHH0ZEB1] Specify a filter for which metric collectors to return
    When [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors are retrieved using a filter
    Then [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors matching the filter are returned

  Scenario: [01D3PX3NRADQPMS95EB5C7ECD7] Retrieving collectors for specified MetricId(s)
    When [01D3PX3NRADQPMS95EB5C7ECD7] retrieving collectors for MetricId(s)
    Then [01D3PX3NRADQPMS95EB5C7ECD7] the returned collectors contain descriptors that match the MetricId(s)