Feature: [01D4RV5JQPQHXQNNJR8740J39J] Sending request is coupled with receiving reply

  Scenario: [01D4RDC36HSVM7M65SCQK13T2S] Coupled async request / reply
    Given [01D4RDC36HSVM7M65SCQK13T2S] a ReqRep service and an Executor
    When [01D4RDC36HSVM7M65SCQK13T2S] async request is sent
    Then [01D4RDC36HSVM7M65SCQK13T2S] the task awaits async for the reply



