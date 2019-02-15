Feature: [01D3J441N6BM05NKCBQEVYTZY8] Gathering metrics

  Background:
    Given [01D3J441N6BM05NKCBQEVYTZY8] metrics are registered for the following types:
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

  Scenario: [01D3PPPT1ZNXPKKWM29R14V5ZT] Gathering all metrics
    When [01D3PPPT1ZNXPKKWM29R14V5ZT-2] all metrics are gathered
    Then [01D3PPPT1ZNXPKKWM29R14V5ZT-3] metrics are returned for all registered metric descriptors

  Scenario: [01D3PPY3E710BYY8DQDKVQ31KY] Gather metrics for specific Desc.id(s)
    When [01D3PPY3E710BYY8DQDKVQ31KY-2] metrics are gathered
    Then [01D3PPY3E710BYY8DQDKVQ31KY-3] metrics are returned for specified Desc.id(s)

  Scenario: [01D3PQ2KMBY07K48Q281SMPED6] Gather metrics for specific Desc.fq_name(s)
    When [01D3PQ2KMBY07K48Q281SMPED6-2] metrics are gathered
    Then [01D3PQ2KMBY07K48Q281SMPED6-3] metrics are returned for specified Desc.fq_name(s)