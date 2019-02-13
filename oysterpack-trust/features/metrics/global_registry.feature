Feature: [01D3J441N6BM05NKCBQEVYTZY8] A global prometheus metrics registry

  Scenario: [01D3J3D7PA4NR9JABZWT635S6B] Registering metrics
    Given [01D3J3D7PA4NR9JABZWT635S6B-1] metrics are registered for the following types:
      |MetricType|
      |IntCounter|
      |Counter   |
    Then [01D3J3D7PA4NR9JABZWT635S6B-2] the registered metric's Desc can be retrieved via `MetricRegistry::descs()` and `MetricRegistry::filter_descs()`
    And [01D3J3D7PA4NR9JABZWT635S6B-3] the metrics are gathered as part of global metric collection - `MetricRegistry::gather()`
    And [01D3J3D7PA4NR9JABZWT635S6B-4] metrics can be gathered using Desc ids - `MetricsRegistry::gather_metrics()`
    And [01D3J3D7PA4NR9JABZWT635S6B-5] its metrics can be gathered using its MetricId name - `MetricsRegistry::gather_metrics_by_name()`
    And [01D3J3D7PA4NR9JABZWT635S6B-6] the metric collector can be retrieved - `MetricsRegistry::collectors()`
    And [01D3J3D7PA4NR9JABZWT635S6B-7] the metric family count shows the metrics were added - `MetricsRegistry::metric_family_count()`

  Scenario: [01D3J3DRS0CJ2YN99KAWQ19103] Register the same metric twice
    Given [01D3J3DRS0CJ2YN99KAWQ19103-1] a metric is already registered with the global registry
    When [01D3J3DRS0CJ2YN99KAWQ19103-2] a duplicate metric is registered with the same name and labels
    Then [01D3J3DRS0CJ2YN99KAWQ19103-3] the duplicate metric will fail to register