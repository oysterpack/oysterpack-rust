Feature: [01D409J6XGK5SAKFD63SZHQZG7] Registered metrics will automatically be augmented with constant labels that identify the app, version, and instance.

  - LabelId(s) will be used as the label names:
    - 01D409Z1SMDQ0FF4FM5ZB33T33 - app name
    - 01D40A0EJHZSW588H9YZG247Y1 - app version
    - 01D40A10Y8BGE3JS44MDY217TM - app instance ULID

  Scenario: [01D40A9YJFX49TAS76N8QXJE7X] registering a collector
    When [01D40A9YJFX49TAS76N8QXJE7X-1] a collector is registered
    Then [01D40A9YJFX49TAS76N8QXJE7X-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AA7VWYETER324X0HH5H4H] registering a Counter metric
    When [01D40AA7VWYETER324X0HH5H4H-1] the metric is registered
    Then [01D40AA7VWYETER324X0HH5H4H-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AABS64CQBJ5SM4VYCA41Z] registering an IntCounter metric
    When [01D40AABS64CQBJ5SM4VYCA41Z-1] the metric is registered
    Then [01D40AABS64CQBJ5SM4VYCA41Z-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AACWP0WQ1QKV6153V7943] registering a CounterVec metric
    When [01D40AACWP0WQ1QKV6153V7943-1] the metric is registered
    Then [01D40AACWP0WQ1QKV6153V7943-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AADPFTHRVF0EBZZA3PMZE] registering an IntCounterVec metric
    When [01D40AADPFTHRVF0EBZZA3PMZE-1] the metric is registered
    Then [01D40AADPFTHRVF0EBZZA3PMZE-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D409TQD5MX1DBSMD7MBV5NNX] registering a Gauge metric
    When [01D409TQD5MX1DBSMD7MBV5NNX-1] the metric is registered
    Then [01D409TQD5MX1DBSMD7MBV5NNX-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AAEDPEDWV3N1G7BDYDFWJ] registering an IntGauge metric
    When [01D40AAEDPEDWV3N1G7BDYDFWJ-1] the metric is registered
    Then [01D40AAEDPEDWV3N1G7BDYDFWJ-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AAF46F0WVN6S7YZG9XQCH] registering a GaugeVec metric
    When [01D40AAF46F0WVN6S7YZG9XQCH-1] the metric is registered
    Then [01D40AAF46F0WVN6S7YZG9XQCH-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AAFSPW048Y7K8T9MQYKKV] registering an IntGaugeVec metric
    When [01D40AAFSPW048Y7K8T9MQYKKV-1] the metric is registered
    Then [01D40AAFSPW048Y7K8T9MQYKKV-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40AAH1DYX264VRWMHTA5JJB] registering a Histogram metric
    When [01D40AAH1DYX264VRWMHTA5JJB-1] the metric is registered
    Then [01D40AAH1DYX264VRWMHTA5JJB-2] its Desc(s) will contain the app name, version, and instance constant labels

  Scenario: [01D40ACCH7FGN4APP49Q8Q9VFZ] registering a HistogramVec metric
    When [01D40ACCH7FGN4APP49Q8Q9VFZ-1] the metric is registered
    Then [01D40ACCH7FGN4APP49Q8Q9VFZ-2] its Desc(s) will contain the app name, version, and instance constant labels
