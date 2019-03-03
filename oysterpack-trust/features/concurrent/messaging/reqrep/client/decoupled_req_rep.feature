Feature: [01D4R2Y8D8FCJGJ1JTFDVT4KD5] Sending request is decoupled from receiving reply

  - ReqRep::send() is used to send the request and returns a ReplyReceiver
  - ReplyReceiver is used to receive the reply at the client's leisure
    - this enables the client to do additional work between sending the request and receiving the reply

  Scenario: [01D4R3896YCW74JWQH2N3CG4Y0] Send decoupled request
    Then [01D4R3896YCW74JWQH2N3CG4Y0] the reply is received successfully

  Scenario: [01D4RDHVTVKWHBBRX1P1EHX30A] Fire and forget - closing the ReplyReceiver
    Then [01D4RDHVTVKWHBBRX1P1EHX30A] the backend service processed the request successfully

  Scenario: [01D521HRC5G63GTWMJ7QQ1KHVS] Fire and forget - discarding the ReplyReceiver
    Then [01D521HRC5G63GTWMJ7QQ1KHVS] the backend service processed the request successfully

  Scenario: [01D5213GKE1JCB85WET013V681] Backend service panics while processing request
    Then [01D5213GKE1JCB85WET013V681-1] the ReplyReciever should fail because the corresponding sender will be dropped
    And [01D5213GKE1JCB85WET013V681-2] sending any additional requests will fail because the backend service will be terminated

  Scenario: [01D524VZWM925RKEBVP5C0WXYJ] Submit requests when the channel is full
    Then [01D524VZWM925RKEBVP5C0WXYJ] all requests will be processed successfully
