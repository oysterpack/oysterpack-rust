Feature: [01D3PQBDWM4BAJQKXF9R0MQED7] Metric collectors for the registered metrics can be retrieved from the global metric registry.

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

  Scenario: [01D3PSPRYX7XHSGX0JFC8TT59H] All metric collectors are returned
    When [01D3PSPRYX7XHSGX0JFC8TT59H-2] all metric collectors are retrieved
    Then [01D3PSPRYX7XHSGX0JFC8TT59H-3] all collector descriptors should match the descriptors retrieved from the metric registry

  Scenario: [01D3PX3BGCMV2PS6FDXHH0ZEB1] Specify a filter for which metric collectors to return
    When [01D3PX3BGCMV2PS6FDXHH0ZEB1-2] collectors are retrieved using a filter
    Then [01D3PX3BGCMV2PS6FDXHH0ZEB1-3] collectors matching the filter are returned

  Scenario: [01D3PX3NRADQPMS95EB5C7ECD7] Retrieving collectors for specified MetricId(s)
    When [01D3PX3NRADQPMS95EB5C7ECD7-2] retrieving collectors for MetricId(s)
    Then [01D3PX3NRADQPMS95EB5C7ECD7-3] the returned collectors contain descriptors that match the MetricId(s)