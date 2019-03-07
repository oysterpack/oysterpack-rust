Feature: [01D59X5KJ7Q72C2F2FP2VYVGS1] ReqRep related metrics can be easily gathered

  The following ReqRep metrics are collected and tracked by ReqRepId:
  - ReqRep service instance count
  - ReqRep client requests sent
  - Processor timer metrics for FutureReply
    - excludes requests that panic
  - Processor panics for FutureReply

  Scenario: [01D5AKRF2JQJTQZQAHZFTV5CEG] Start multiple ReqRep services
    When [01D5AKRF2JQJTQZQAHZFTV5CEG] all ReqRep metric descriptors are retrieved
    Then [01D5AKRF2JQJTQZQAHZFTV5CEG] descriptors for expected metric types are returned

