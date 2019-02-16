Feature: [01D3M9X86BSYWW3132JQHWA3AT] Gathered metrics can be encoded in prometheus compatible text format

  Scenario: [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] gathering metrics
    Given [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-1] metrics are registered for the following types:
      | MetricType     |
      | IntCounter     |
      | Counter        |
      | CounterVec     |
      | IntGauge       |
      | Gauge          |
      | GaugeVec       |
      | Histogram      |
      | HistogramTimer |
      | HistogramVec   |
    When [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-2] metrics are gathered and text encoded
    Then [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-3] the text encoded metrics contain the gathered metrics

