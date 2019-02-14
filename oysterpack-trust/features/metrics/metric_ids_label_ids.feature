Feature: [01D3J441N6BM05NKCBQEVYTZY8] MetricId and LabelId provides ULID based names.

  Valid metric and label names in prometheus must not start with number. Thus MetricId and LabelId names are prefixed with the following
  - MetricId::name() prefixes the ULID with 'M'
  - LabelId::name() prefixes the ULID with 'L'

  Scenario: [01D3J3D7PA4NR9JABZWT635S6B] Registering metrics
    Given [01D3J3D7PA4NR9JABZWT635S6B-1] metrics are registered for the following types:
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
    Then [01D3J3D7PA4NR9JABZWT635S6B-2] the registered metric's Desc can be retrieved via `MetricRegistry::descs()` and `MetricRegistry::filter_descs()`
    And [01D3J3D7PA4NR9JABZWT635S6B-3] the metrics are gathered as part of global metric collection - `MetricRegistry::gather()`
    And [01D3J3D7PA4NR9JABZWT635S6B-4] metrics can be gathered using Desc ids - `MetricsRegistry::gather_metrics()`
    And [01D3J3D7PA4NR9JABZWT635S6B-5] its metrics can be gathered using its MetricId name - `MetricsRegistry::gather_metrics_by_name()`
    And [01D3J3D7PA4NR9JABZWT635S6B-6] the metric collector can be retrieved - `MetricsRegistry::collectors()`
    And [01D3J3D7PA4NR9JABZWT635S6B-7] the metric family count shows the metrics were added - `MetricsRegistry::metric_family_count()`


