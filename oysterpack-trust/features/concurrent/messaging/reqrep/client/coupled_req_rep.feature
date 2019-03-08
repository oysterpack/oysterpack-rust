Feature: [01D4RV5JQPQHXQNNJR8740J39J] Sending request is coupled with receiving reply

  Scenario: [01D4RDC36HSVM7M65SCQK13T2S] ReqRep::send_rec()
    Then [01D4RDC36HSVM7M65SCQK13T2S] the reply is successfully received

  Scenario: [01D52268F786CHS3EE0QD5Y785] Backend service panics while processing request
    Then [01D52268F786CHS3EE0QD5Y785-1] request will fail because the ReplyReceiver channel becomes disconnected
    And [01D52268F786CHS3EE0QD5Y785-2] sending any additional requests will fail because the backend service has terminated



