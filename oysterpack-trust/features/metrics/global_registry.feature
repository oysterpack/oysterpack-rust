Feature: [01D3J441N6BM05NKCBQEVYTZY8] A global prometheus metrics registry will be made available for the application.

  Scenario: [01D3J3D7PA4NR9JABZWT635S6B] Using the global registry from multiple threads
    Given [01D3J3D7PA4NR9JABZWT635S6B-1] there are 2 threads using the global registry
    When [01D3J3D7PA4NR9JABZWT635S6B-2] 1 thread registers a metric
    Then [01D3J3D7PA4NR9JABZWT635S6B-3] the other thread will be able to see that the metric was registered via the global registry

  Rule: Descriptors registered with the same registry have to fulfill certain consistency and uniqueness criteria if they share the same fully-qualified name.

  # They must have the same help string and the same label names (aka label dimensions) in each, constLabels and variableLabels,
  # but they must differ in the values of the constLabels.
  #
  # Descriptors that share the same fully-qualified names and the same label values of their constLabels are considered equal.

  Scenario: [01D3J3DRS0CJ2YN99KAWQ19103] Register 2 metrics using the same MetricId and no labels
    Given [01D3J3DRS0CJ2YN99KAWQ19103-1] an Counter is already registered with the global registry
    When [01D3J3DRS0CJ2YN99KAWQ19103-2] a Gauge is registered using the same MetricId
    Then [01D3J3DRS0CJ2YN99KAWQ19103-3] the duplicate metric will fail to register

  Scenario: [01D3MT4JY1NZH2WW0347B9ZAS7] Register 2 metrics using the same MetricId and same const labels
    Given [01D3MT4JY1NZH2WW0347B9ZAS7-1] a Counter is already registered with the global registry
    When [01D3MT4JY1NZH2WW0347B9ZAS7-2] a Gauge is registered using the same MetricId and const labels
    Then [01D3MT4JY1NZH2WW0347B9ZAS7-3] the duplicate metric will fail to register

  Scenario: [01D3MT8KDP434DKZ6Y54C80BB0] Register 2 metrics using the same MetricId and same const label names but different label values
    Given [01D3MT8KDP434DKZ6Y54C80BB0-1] a Counter is already registered with the global registry
    When [01D3MT8KDP434DKZ6Y54C80BB0-2] a Gauge is registered using the same MetricId and const label names but different label values
    Then [01D3MT8KDP434DKZ6Y54C80BB0-3] the Gauge will successfully register