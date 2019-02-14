Feature: [01D3PQBDWM4BAJQKXF9R0MQED7] Metric descriptors for the registered metrics can be retrieved from the global metric registry.

  Background:
    Given [01D3PQBDWM4BAJQKXF9R0MQED7] metrics are registered for the following types:
      |MetricType     |
      |IntCounter     |
      |Counter        |
      |CounterVec     |
      |IntGauge       |
      |Gauge          |
      |GaugeVec       |
      |Histogram      |
      |HistogramTimer |
      |HistogramVec   |

  Scenario: [01D3PSPRYX7XHSGX0JFC8TT59H] All metric descriptors are returned
  When [01D3PSPRYX7XHSGX0JFC8TT59H-2] all metric descriptors are retrieved
  Then [01D3PSPRYX7XHSGX0JFC8TT59H-3] then all returned should match the descs that are returned as part of gathering metrics

  Scenario: [01D3PSPCNHH6CSW08RTFKZZ8SP] Specify a filter for which metric descriptors to return
    When [01D3PSPCNHH6CSW08RTFKZZ8SP-2] a filter is specified
    Then [01D3PSPCNHH6CSW08RTFKZZ8SP-3] descriptors matching the filter are returned

  Scenario: [01D3PSP4TQK6ESKSB6AEFWAAYF] Retrieving descriptors for specified MetricId(s)
    When [01D3PSP4TQK6ESKSB6AEFWAAYF-2] specify MetricId(s)
    Then [01D3PSP4TQK6ESKSB6AEFWAAYF-3] descriptors whose names match the MetricId(s) are returned