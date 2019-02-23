Feature: [01D43V1W2BHDR5MK08D1HFSFZX] A global prometheus metrics registry is provided.

  Scenario: [01D3J3D7PA4NR9JABZWT635S6B] Using the global registry from multiple threads
    Given [01D3J3D7PA4NR9JABZWT635S6B] there are 2 threads using the global registry
    When [01D3J3D7PA4NR9JABZWT635S6B] 1 thread registers a metric
    Then [01D3J3D7PA4NR9JABZWT635S6B] the other thread will be able to see that the metric was registered via the global registry

  Rule: Descriptors registered with the same registry have to fulfill certain consistency and uniqueness criteria if they share the same fully-qualified name.

  # They must have the same help string and the same label names (aka label dimensions) in each, constLabels and variableLabels,
  # but they must differ in the values of the constLabels.
  #
  # Descriptors that share the same fully-qualified names and the same label values of their constLabels are considered equal.

  Scenario: [01D3J3DRS0CJ2YN99KAWQ19103] Register 2 metrics using the same MetricId and no labels
    Given [01D3J3DRS0CJ2YN99KAWQ19103] an Counter is already registered with the global registry
    When [01D3J3DRS0CJ2YN99KAWQ19103] a Gauge is registered using the same MetricId
    Then [01D3J3DRS0CJ2YN99KAWQ19103] the duplicate metric will fail to register

  Scenario: [01D3MT4JY1NZH2WW0347B9ZAS7] Register 2 metrics using the same MetricId and same const labels
    Given [01D3MT4JY1NZH2WW0347B9ZAS7] a Counter is already registered with the global registry
    When [01D3MT4JY1NZH2WW0347B9ZAS7] a Gauge is registered using the same MetricId and const labels
    Then [01D3MT4JY1NZH2WW0347B9ZAS7] the duplicate metric will fail to register

  Scenario: [01D3MT8KDP434DKZ6Y54C80BB0] Register 2 metrics using the same MetricId and same const label names but different label values
    Given [01D3MT8KDP434DKZ6Y54C80BB0] a Counter is already registered with the global registry
    When [01D3MT8KDP434DKZ6Y54C80BB0] a Gauge is registered using the same MetricId and const label names but different label values
    Then [01D3MT8KDP434DKZ6Y54C80BB0] the Gauge will successfully register

  Rule: descriptor `help` must not be blank

  Scenario: [01D4B036AWCJD6GCDNVGA5YVBB] Register metrics with a blank help message on the descriptor
    When [01D4B036AWCJD6GCDNVGA5YVBB] registering metrics for each of the MetricId supported types
    Then [01D4B036AWCJD6GCDNVGA5YVBB] the metrics will fail to register

  Scenario: [01D4B08N90FM8EZTT3X5Y72D3M] Register a collector containing multiple descriptors where 1 descriptor has a blank help message
    When [01D4B08N90FM8EZTT3X5Y72D3M] registering the collector
    Then [01D4B08N90FM8EZTT3X5Y72D3M] the collector will fail to register

  Rule: descriptor `help` max length is 250

  Scenario: [01D4B0S8QW63C6YFCB83CQZXA7] Register metrics with a help message length of 250
    When [01D4B0S8QW63C6YFCB83CQZXA7] registering metrics for each of the MetricId supported types
    Then [01D4B0S8QW63C6YFCB83CQZXA7] the metrics will succeed to register

  Scenario: [01D4B0RS3V7NHCPDSPQTJDNB6C] Register metrics with a help message length of 251
    When [01D4B0RS3V7NHCPDSPQTJDNB6C] registering metrics for each of the MetricId supported types
    Then [01D4B0RS3V7NHCPDSPQTJDNB6C] the metrics will fail to register

  Scenario: [01D4B0S1J3XV06GEZJGA9Q5F8V] Register a collector containing multiple descriptors where 1 descriptor has a help message length of 251
    When [01D4B0S1J3XV06GEZJGA9Q5F8V] registering the collector
    Then [01D4B0S1J3XV06GEZJGA9Q5F8V] the collector will fail to register

  Rule: descriptor constant label name or value must not be blank

  Scenario: [01D4B0K42BC2TB0TAA2QP6BRWZ] Register metrics containing a descriptor with a blank label name
    When [01D4B0K42BC2TB0TAA2QP6BRWZ] registering metrics for each of the MetricId supported types
    Then [01D4B0K42BC2TB0TAA2QP6BRWZ] the metric will fail to register

  Scenario: [01D4B0KBWVFHEAVJSRD41TBJ6Z] Register a collector containing a descriptor with a blank label name
    When [01D4B0KBWVFHEAVJSRD41TBJ6Z] registering the collector
    Then [01D4B0KBWVFHEAVJSRD41TBJ6Z] the collector will fail to register

  Scenario: [01D4B0JCKY2ZQNXD0A0CQA89WK] Register a collector containing a descriptor with a blank label value
    When [01D4B0JCKY2ZQNXD0A0CQA89WK] registering the collector
    Then [01D4B0JCKY2ZQNXD0A0CQA89WK] the collector will fail to register

  Rule: descriptor label name or value max length is 30

  Scenario: [01D4B0W77XVHM7BP2PJ5M33HK7] Register metrics containing a descriptor with a const label name and value length of 30
    When [01D4B0W77XVHM7BP2PJ5M33HK7] registering metrics for each of the MetricId supported types
    Then [01D4B0W77XVHM7BP2PJ5M33HK7] the metric will succeed to register

  Scenario: [01D4B1TXRQSNKJNGBZ84XBAS55] Register metric vector containing a descriptor with a variable label name length of 30
    When [01D4B1TXRQSNKJNGBZ84XBAS55] registering metric vectors for each of the MetricId supported types
    Then [01D4B1TXRQSNKJNGBZ84XBAS55] the metric will succeed to register

  Scenario: [01D4B0XMQ2ZR2FHZHYM5KSBH90] Register metrics containing a descriptor with a label name and value length of 31
    When [01D4B0XMQ2ZR2FHZHYM5KSBH90] registering metrics for each of the MetricId supported types
    Then [01D4B0XMQ2ZR2FHZHYM5KSBH90] the metric will fail to register

  Scenario: [01D4B1XP3V78X2HG3Z8NA1H0KH] Register metric vector containing a descriptor with a variable label name length of 31
    When [01D4B1XP3V78X2HG3Z8NA1H0KH] registering metric vectors for each of the MetricId supported types
    Then [01D4B1XP3V78X2HG3Z8NA1H0KH] the metric will fail to register

  Scenario: [01D4B0YGEN4XF275ZE660W1PRC] Register a collector containing a descriptor with a const label name length of 31
    When [01D4B0YGEN4XF275ZE660W1PRC] registering metrics
    Then [01D4B0YGEN4XF275ZE660W1PRC] the metric will fail to register

  Scenario: [01D4B0Y6Y494DYFVE3YVQYXPPR] Register a collector containing a descriptor with a const label value length of 31
    When [01D4B0Y6Y494DYFVE3YVQYXPPR] registering metrics
    Then [01D4B0Y6Y494DYFVE3YVQYXPPR] the metric will fail to register

  Scenario: [01D4B1WB02B15YY1MP4BW24Q1P] Register a collector containing a descriptor with a variable label name length of 31
    When [01D4B1WB02B15YY1MP4BW24Q1P] registering metrics
    Then [01D4B1WB02B15YY1MP4BW24Q1P] the metric will fail to register

  Scenario: [01D4B0Y0GKYDGCB572SKR72H7J] Register a collector containing a descriptor with a help label name length of 31
    When [01D4B0Y0GKYDGCB572SKR72H7J] registering metrics
    Then [01D4B0Y0GKYDGCB572SKR72H7J] the metric will fail to register

  Rule: for metric vectors, at least 1 variable label must be defined on the descriptor

  Scenario: [01D4B1F6AXH4DHBXC42756CVNZ] Register a metric vectors with no variable labels
    When [01D4B1F6AXH4DHBXC42756CVNZ] registering metrics
    Then [01D4B1F6AXH4DHBXC42756CVNZ] the metric will fail to register

  Rule: for metric vectors, variable labels must be blank

  Scenario: [01D4B1FPWJ8RFWMYNEC6MD81VS] Register a metric vectors with blank labels
    When [01D4B1FPWJ8RFWMYNEC6MD81VS] registering metrics
    Then [01D4B1FPWJ8RFWMYNEC6MD81VS] the metric will fail to register

  Scenario: [01D4B1KQZ9F4FMKF51FHF84D72] Register a metric colletors containing descriptors with blank labels
    When [01D4B1KQZ9F4FMKF51FHF84D72] registering collector
    Then [01D4B1KQZ9F4FMKF51FHF84D72] the collector will fail to register

  Rule: for metric vectors, variable labels must be unique

  Scenario: [01D4B1ZKJ821A86MX88PPS05RY] Register a metric vectors with duplicate labels
    When [01D4B1ZKJ821A86MX88PPS05RY] registering metrics
    Then [01D4B1ZKJ821A86MX88PPS05RY] the metric will fail to register

  Scenario: [01D4B20762DXC3MZ2494AK6CT3] Register a metric colletors containing descriptors with duplicate labels
    When [01D4B20762DXC3MZ2494AK6CT3] registering collector
    Then [01D4B20762DXC3MZ2494AK6CT3] the collector will fail to register