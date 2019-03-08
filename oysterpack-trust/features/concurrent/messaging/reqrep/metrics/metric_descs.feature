Feature: [01D59X5KJ7Q72C2F2FP2VYVGS1] ReqRep related metric descriptors can be easily retrieved

  The following ReqRep metrics are collected and tracked by ReqRepId:
  - ReqRep service instance count
  - ReqRep client requests sent
  - Processor timer metrics for FutureReply
    - excludes requests that panic
  - Processor panics for FutureReply

  Scenario: [01D5AKRF2JQJTQZQAHZFTV5CEG] Get ReqRep related metric descriptors
    Then [01D5AKRF2JQJTQZQAHZFTV5CEG] descriptors for expected metric types are returned

