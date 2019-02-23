Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering a subset of the metrics

  Metrics can be gathered matching on:
  - descriptors on ID, name, or labels
    - matching on DescId is the same as matching on Desc name and const labels
  - MetricId(s)

  Background:
    Given [01D3J441N6BM05NKCBQEVYTZY8] metrics are registered

  Scenario: [01D3PPY3E710BYY8DQDKVQ31KY] Gather metrics for DescId(s)
    When [01D3PPY3E710BYY8DQDKVQ31KY] metrics are gathered for specified Desc IDs
    Then [01D3PPY3E710BYY8DQDKVQ31KY] metrics are returned that match on Desc name and const labels

  Scenario: [01D4BXN2ZMYRHNGRRCSTKVN0AP] Gather metrics for DescId(s) containing dup ids
    When [01D4BXN2ZMYRHNGRRCSTKVN0AP] metrics are gathered
    Then [01D4BXN2ZMYRHNGRRCSTKVN0AP] metrics are returned that match on Desc name and const labels

  Scenario: [01D4D0GEXXQ3WKK78DYC0RJHKD] Gather metrics for DescId(s) containing some that do not match
    When [01D4D0GEXXQ3WKK78DYC0RJHKD] metrics are gathered
    Then [01D4D0GEXXQ3WKK78DYC0RJHKD] metrics are returned that match on Desc name and const labels

  Scenario: [01D4D1774JHQNB8X0QRBYEAEBW] Gather metrics for DescId(s) containing none that not match
    When [01D4D1774JHQNB8X0QRBYEAEBW] metrics are gathered
    Then [01D4D1774JHQNB8X0QRBYEAEBW] no metrics are returned

  Scenario: [01D4D1WEKJBFQSR0Z1Q10ZHD2R] Gather metrics for DescId(s) with an empty &[DescId]
    When [01D4D1WEKJBFQSR0Z1Q10ZHD2R] metrics are gathered
    Then [01D4D1WEKJBFQSR0Z1Q10ZHD2R] no metrics are returned

  Scenario: [01D3PQ2KMBY07K48Q281SMPED6] Gather metrics for descriptor names
    When [01D3PQ2KMBY07K48Q281SMPED6] metrics are gathered
    Then [01D3PQ2KMBY07K48Q281SMPED6] metrics are returned that match on the MetricFamily name

  Scenario: [01D4BXX8A1SY3CYA8V9330F7QM] Gather metrics for descriptor names with dup names
    When [01D4BXX8A1SY3CYA8V9330F7QM] metrics are gathered
    Then [01D4BXX8A1SY3CYA8V9330F7QM] metrics are returned that match on the MetricFamily name

  Scenario: [01D4D2YZXEES3GHA30J5ZZFPGF] Gather metrics for descriptor names containing some that do not match
    When [01D4D2YZXEES3GHA30J5ZZFPGF] metrics are gathered
    Then [01D4D2YZXEES3GHA30J5ZZFPGF] metrics are returned that match on the MetricFamily name

  Scenario: [01D4D302NGKYAVCHDF4A1Z6SB3] Gather metrics for descriptor names containing none that match
    When [01D4D302NGKYAVCHDF4A1Z6SB3] metrics are gathered
    Then [01D4D302NGKYAVCHDF4A1Z6SB3] no metrics are returned

  Scenario: [01D4D30ABTZ72781C5NDP42217] Gather metrics for descriptor names using an empty &[Name]
    When [01D4D30ABTZ72781C5NDP42217] metrics are gathered
    Then [01D4D30ABTZ72781C5NDP42217] no metrics are returned

  Scenario: [01D3VC85Q8MVBJ543SHZ4RE9T2] Gather metrics for MetricId(s)
    # the MetricId is used as the Desc name
    When [01D3VC85Q8MVBJ543SHZ4RE9T2] metrics are gathered
    Then [01D3VC85Q8MVBJ543SHZ4RE9T2] metrics are returned that match on the MetricFamily name

  Scenario: [01D4D3C0EBPZX8NWCYRD8YJ0Y3] Gather metrics for MetricId(s) containing dups
    # the MetricId is used as the Desc name
    When [01D4D3C0EBPZX8NWCYRD8YJ0Y3] metrics are gathered
    Then [01D4D3C0EBPZX8NWCYRD8YJ0Y3] metrics are returned that match on the MetricFamily name

  Scenario: [01D4D3EX9TP87RQ2S11PFNXG2T] Gather metrics for MetricId(s) containing some that do not match
    # the MetricId is used as the Desc name
    When [01D4D3EX9TP87RQ2S11PFNXG2T] metrics are gathered
    Then [01D4D3EX9TP87RQ2S11PFNXG2T] metrics are returned that match on the MetricFamily name

  Scenario: [01D4D3EKJME2MCH81DXTAMGMJS] Gather metrics for MetricId(s) containing none that match
    # the MetricId is used as the Desc name
    When [01D4D3EKJME2MCH81DXTAMGMJS] metrics are gathered
    Then [01D4D3EKJME2MCH81DXTAMGMJS] no metrics are returned

  Scenario: [01D4D3EBMA7XR2FWA1Q6E5F560] Gather metrics for MetricId(s) using an empty &[MetricId]
    # the MetricId is used as the Desc name
    When [01D4D3EBMA7XR2FWA1Q6E5F560] metrics are gathered
    Then [01D4D3EBMA7XR2FWA1Q6E5F560] no metrics are returned

  Scenario: [01D43MQQ1H59ZGJ9G2AMEJB5RF] Gather metrics for labels
    When [01D43MQQ1H59ZGJ9G2AMEJB5RF] metrics are gathered
    Then [01D43MQQ1H59ZGJ9G2AMEJB5RF] metrics are returned that match on labels

  Scenario: [01D4D40A3652FWV58EQMY6907F] Gather metrics for labels with some non-matching labels
    When [01D4D40A3652FWV58EQMY6907F] metrics are gathered
    Then [01D4D40A3652FWV58EQMY6907F] metrics are returned that match on labels

  Scenario: [01D4D417QGFCY2XSSARWWH49P5] Gather metrics for labels with no matching labels
    When [01D4D417QGFCY2XSSARWWH49P5] metrics are gathered
    Then [01D4D417QGFCY2XSSARWWH49P5] no metrics are returned

  Scenario: [01D4D3WKY9607QG71S76DE65W8] Gather metrics for labels using an empty HashMap
    When [01D4D3WKY9607QG71S76DE65W8] metrics are gathered
    Then [01D4D3WKY9607QG71S76DE65W8] no metrics are returned