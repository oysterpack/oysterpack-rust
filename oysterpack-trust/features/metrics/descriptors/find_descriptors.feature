Feature: [01D3SF7KGJZZM50TXXW5HX4N99] Find descriptors matching filters

  Background:
    Given [01D3PQBDWM4BAJQKXF9R0MQED7] metrics are registered

  Scenario: [01D3PSPCNHH6CSW08RTFKZZ8SP] Find metric descriptors matching a filter
    When [01D3PSPCNHH6CSW08RTFKZZ8SP] retrieving descriptors
    Then [01D3PSPCNHH6CSW08RTFKZZ8SP] descriptors matching the filter are returned

  Scenario: [01D3PSP4TQK6ESKSB6AEFWAAYF] Find descriptors for MetricId(s)
    When [01D3PSP4TQK6ESKSB6AEFWAAYF] retrieving descriptors
    Then [01D3PSP4TQK6ESKSB6AEFWAAYF] descriptors are returned with names matching the specified MetricId(s)

  Scenario: [01D48G17F9EYM0XEZBZ794SGMW] Find descriptors for MetricId(s) containing some non-registered MetricId(s)
    When [01D48G17F9EYM0XEZBZ794SGMW] retrieving descriptors
    Then [01D48G17F9EYM0XEZBZ794SGMW] descriptors are returned with names matching the specified registered MetricId(s)

  Scenario: [01D48TCVH8R57XSNQN4E89PYXC] Find descriptors where all MetricId(s) are not registered
    When [01D48TCVH8R57XSNQN4E89PYXC] retrieving descriptors
    Then [01D48TCVH8R57XSNQN4E89PYXC] no descriptors are returned

  Scenario: [01D48TCN32NHVFEYSJCHCQE451] Find descriptors for an empty &[MetricId]
    When [01D48TCN32NHVFEYSJCHCQE451] retrieving descriptors
    Then [01D48TCN32NHVFEYSJCHCQE451] no descriptors are returned

  Scenario: [01D48FX6T8SAJZWHDTZBQYWFAG] Find descriptors that match const labels
    When [01D48FX6T8SAJZWHDTZBQYWFAG] retrieving descriptors
    Then [01D48FX6T8SAJZWHDTZBQYWFAG] descriptors are returned with matching labels

  Scenario: [01D48TM50MN9ZPGYD1TD2QBSKA] Find descriptors against labels containing unknown label names and values
    When [01D48TM50MN9ZPGYD1TD2QBSKA] retrieving descriptors
    Then [01D48TM50MN9ZPGYD1TD2QBSKA] descriptors are returned with matching labels

  Scenario: [01D48TKY8GJJ56Z14NAX000DPZ] Find descriptors against labels that are all unknown
    When [01D48TKY8GJJ56Z14NAX000DPZ] retrieving descriptors
    Then [01D48TKY8GJJ56Z14NAX000DPZ] no descriptors are returned

  Scenario: [01D48TK6AMZCQ5CNYMJC0NVR37] Find descriptors against an empty label
    When [01D48TK6AMZCQ5CNYMJC0NVR37] retrieving descriptors
    Then [01D48TK6AMZCQ5CNYMJC0NVR37] no descriptors are returned
