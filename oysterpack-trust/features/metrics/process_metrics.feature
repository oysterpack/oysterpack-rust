Feature: [01D3JB8ZGW3KJ3VT44VBCZM3HA] A process metrics Collector is automatically registered with the global metrics registry

  Scenario: [01D3JB9B4NP8T1PQ2Q85HY25FQ] gathering metrics
    When [01D3JB9B4NP8T1PQ2Q85HY25FQ-2] metrics are gathered
    Then [01D3JB9B4NP8T1PQ2Q85HY25FQ-3] process metrics are included

  Scenario: [01D3JBCE21WYX6VMWCM4GW2ZTE] gathering process metrics
    When [01D3JBCE21WYX6VMWCM4GW2ZTE-2] process metrics are gathered
    Then [01D3JBCE21WYX6VMWCM4GW2ZTE-3] they are successfully gathered