Feature: [01D43V2S6HBV642EKK5YGJNH32] All prometheus metrics support MetricId and LabelId ULID based names.

  Valid metric and label names in prometheus must not start with number. Thus MetricId and LabelId names are prefixed with the following
  - MetricId::name() prefixes the ULID with 'M'
  - LabelId::name() prefixes the ULID with 'L'

  Metric descriptors and collectors can be looked up via MetricId. Metrics can be gathered for MetricId(s).

  Scenario: [01D3PB6MDJ85MWP3SQ1H94S6R7] MetricId can be defined as a constant
    Then [01D3PB6MDJ85MWP3SQ1H94S6R7-2] the fully qualified names are MetricId based names
