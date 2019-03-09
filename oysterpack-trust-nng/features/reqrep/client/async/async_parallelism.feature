Feature: [01D5J2BK1ZCH1NQJ1JNA3R305M] The client is fully async and supports parallelism.

  - the level of parallelism is configured via DialerConfig::parallelism()
  - default parallelism is set to the number of logical CPUs

  Scenario: [01D5J2CS8VFK0547YZ3B8368YC]