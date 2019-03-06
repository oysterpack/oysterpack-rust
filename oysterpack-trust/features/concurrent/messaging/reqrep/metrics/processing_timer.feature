Feature: [01D4ZS3J72KG380GFW4GMQKCFH] Message processing timer metrics are collected

  - timings are reported in seconds
  - timings for requests that triggered panics while being processed are not observed

  Scenario: [01D5028W0STBFHDAPWA79B4TGG] Processor sleeps for 10 ms
    Given [01D5028W0STBFHDAPWA79B4TGG] TimerBuckets containing a buckets for 5, 10, 15, 20 ms
    When [01D5028W0STBFHDAPWA79B4TGG] 5 requests are sent
    Then [01D5028W0STBFHDAPWA79B4TGG] all timings should be contained within the 15 ms upper bounded bucket

  Scenario: [01D5891JGSV2PPAM9G22FV9T42] Processor::process() panics but Processor is designed to recover from the panic
    When [01D5891JGSV2PPAM9G22FV9T42] the request processing task panics
    Then [01D5891JGSV2PPAM9G22FV9T42-1] the ReqRep service backend task will continue to run
    And [01D5891JGSV2PPAM9G22FV9T42-2] the ReqRep client is still usable
    And [01D5891JGSV2PPAM9G22FV9T42-3] timer metrics are not reported for requests that panicked while being processed