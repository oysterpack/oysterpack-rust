Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] All registered collectors can be retrieved from the registry

  Scenario: [01D3PSPRYX7XHSGX0JFC8TT59H] Retrieve all registered collectors
    Given [01D46BWEKHMHZGSAZF4QQCZ0RV] collectors are registered
    When [01D3PSPRYX7XHSGX0JFC8TT59H] all metric collectors are retrieved
    Then [01D3PSPRYX7XHSGX0JFC8TT59H-1] all collector descriptors should match the descriptors retrieved from the metric registry
    And [01D3PSPRYX7XHSGX0JFC8TT59H-2] registry collector count matches the number of collectors returned