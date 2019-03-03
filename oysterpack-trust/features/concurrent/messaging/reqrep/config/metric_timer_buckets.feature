Feature: [01D4V1PZ43Z5P7XGED38V6DXHA] TimerBuckets are configurable per ReqRep

  TimerBuckets are used to configure a histogram metric used to time message processing in the backend service.
  TimerBuckets are not a one size fits all, and need to be tailored to the performance requirements for the backend Processor.

  Scenario: [01D4V1WN16Q2P0B536GJ84R0SN] Configure a ReqRep service with TimerBuckets
    When [01D4V1WN16Q2P0B536GJ84R0SN] requests are submitted with varying sleep times
    Then [01D4V1WN16Q2P0B536GJ84R0SN] the service's message processing timer histogram metrics align with the sleep times