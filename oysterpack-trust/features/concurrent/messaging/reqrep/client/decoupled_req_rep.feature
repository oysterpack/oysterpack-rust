Feature: [01D4R2Y8D8FCJGJ1JTFDVT4KD5] Sending request is decoupled from receiving reply

  Scenario: [01D4R3896YCW74JWQH2N3CG4Y0] Decoupled async request / reply
    Given [01D4R3896YCW74JWQH2N3CG4Y0] a ReqRep service and an Executor
    When [01D4R3896YCW74JWQH2N3CG4Y0] an async request is sent and then spawn a new task to receive the reply
    Then [01D4R3896YCW74JWQH2N3CG4Y0] the reply is received successfully

  Scenario: [01D4RDHVTVKWHBBRX1P1EHX30A] Decoupled async request / reply where the reply is discarded
    Given [01D4RDHVTVKWHBBRX1P1EHX30A] a ReqRep service and an Executor
    When [01D4RDHVTVKWHBBRX1P1EHX30A] an async request is sent and then the ReplyReciever is closed
    Then [01D4RDHVTVKWHBBRX1P1EHX30A] the backend service processes the request successfully



