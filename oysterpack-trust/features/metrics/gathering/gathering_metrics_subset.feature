Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering a subset of the metrics

  Metrics can be gathered matching:
  - descriptors on ID, name, or labels
  - MetricId(s)

  Background:
    Given [01D3J441N6BM05NKCBQEVYTZY8] metrics are registered

  Scenario: [01D3PPY3E710BYY8DQDKVQ31KY] Gather metrics for DescId(s)
    # matching on DescId is the same as matching on Desc name and const labels
    When [01D3PPY3E710BYY8DQDKVQ31KY] metrics are gathered for specified Desc IDs
    Then [01D3PPY3E710BYY8DQDKVQ31KY] metrics are returned that match on Desc name and const labels

  Scenario: [01D3PQ2KMBY07K48Q281SMPED6] Gather metrics for descriptor names
    When [01D3PQ2KMBY07K48Q281SMPED6] metrics are gathered
    Then [01D3PQ2KMBY07K48Q281SMPED6] metrics are returned that match on the MetricFamily name

  Scenario: [01D3VC85Q8MVBJ543SHZ4RE9T2] Gather metrics for MetricId(s)
    # the MetricId is used as the Desc name
    When [01D3VC85Q8MVBJ543SHZ4RE9T2] metrics are gathered
    Then [01D3VC85Q8MVBJ543SHZ4RE9T2] metrics are returned that match on the MetricFamily name

  Scenario: [01D43MQQ1H59ZGJ9G2AMEJB5RF] Gather metrics for labels
    When [01D43MQQ1H59ZGJ9G2AMEJB5RF] metrics are gathered
    Then [01D43MQQ1H59ZGJ9G2AMEJB5RF] metrics are returned that match on labels