Feature: [01D3JB8ZGW3KJ3VT44VBCZM3HA] A process metrics Collector is automatically registered with the global metrics registry

  The prometheus "process" feature provides `prometheus::process_collector::ProcessCollector`.

  Scenario: [01D3JB9B4NP8T1PQ2Q85HY25FQ] gathering all metrics
    Then [01D3JB9B4NP8T1PQ2Q85HY25FQ] process metrics are included

  Scenario: [01D4FTH1WN3WWZZZH2HN66Y1YK] All metrics descriptors are retrieved
    Then [01D4FTH1WN3WWZZZH2HN66Y1YK] process metric descriptors are included

  Scenario: [01D3JBCE21WYX6VMWCM4GW2ZTE] gathering process metrics
    Then [01D3JBCE21WYX6VMWCM4GW2ZTE] they are successfully gathered