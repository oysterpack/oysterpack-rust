Feature: [01D43V2S6HBV642EKK5YGJNH32] MetricId can be used as the metric name.

  - MetricId(s) are ULID(s)
  - Valid metric names in prometheus must not start with number. Thus when registering a metric using a MetricId, the
    MetricId ULID is prefixed with a 'M'

  Scenario: [01D3PB6MDJ85MWP3SQ1H94S6R7] Define MetricId as a constant
    Then [01D3PB6MDJ85MWP3SQ1H94S6R7] MetricId can be used as a constant

  Scenario: [01D4GEXWKASWC6MHRZVSEHJG5G] Register a metric using a MetricId
    Then [01D4GEXWKASWC6MHRZVSEHJG5G] the metric desc can be retrieved via the MetricId