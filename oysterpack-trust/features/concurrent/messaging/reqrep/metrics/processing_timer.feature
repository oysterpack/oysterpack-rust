Feature: [01D4ZS3J72KG380GFW4GMQKCFH] Message processing timer metrics are collected

  - timings are reported in seconds

  Scenario: [01D4ZS5W2QFN1DF2TTYW0J3RH5] Gather metrics for message processing
    Given [01D4ZS5W2QFN1DF2TTYW0J3RH5] 10 messages were sent
    When [01D4ZS5W2QFN1DF2TTYW0J3RH5] metrics are gathered
    Then [01D4ZS5W2QFN1DF2TTYW0J3RH5] timing metrics for all 10 requests are present

  Scenario: [01D5028W0STBFHDAPWA79B4TGG] Time a processor that sleeps for 10 ms
    Given [01D5028W0STBFHDAPWA79B4TGG] TimerBuckets containing a buckets for 5, 10, 15, 20 ms
    When [01D5028W0STBFHDAPWA79B4TGG] 5 requests are sent
    Then [01D5028W0STBFHDAPWA79B4TGG] all timings should be contained within the 15 ms upper bounded bucket