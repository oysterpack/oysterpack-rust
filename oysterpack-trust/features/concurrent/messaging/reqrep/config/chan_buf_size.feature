Feature: [01D4T5NV48PVFBC2R3Q80B6W72] The buffer size is configurable for the channel used to connect the ReqRep client with the backend service.

  - By default, the channel buffer size is configured to match the number of CPUs.

  Scenario: [01D4T5RY9XDB6MYDE6X7R3X766] Use the default channel buffer size
    Given [01D4T5RY9XDB6MYDE6X7R3X766] the backend service is a Processor sleeps for 1 sec on each request
    When [01D4T5RY9XDB6MYDE6X7R3X766] the client sends N + 1 messages, where N = number of CPUs
    Then [01D4T5RY9XDB6MYDE6X7R3X766-1] the client was able to send messages with out being blocked
    And [01D4T5RY9XDB6MYDE6X7R3X766-2] the number of received messages should be 1 - the rest are queued

  Scenario: [01D4T61JB50KNT3Y7VQ10VX2NR] Set the channel buffer size to equal 2x the number of CPUs
    Given [01D4T61JB50KNT3Y7VQ10VX2NR] the backend service is a Processor sleeps for 1 sec on each request
    When [01D4T61JB50KNT3Y7VQ10VX2NR] the client sends 2N + 1 messages, where N = number of CPUs
    Then [01D4T61JB50KNT3Y7VQ10VX2NR-1] the client was able to send messages with out being blocked
    And [01D4T61JB50KNT3Y7VQ10VX2NR-2] the number of received messages should be 1 - the rest are queued

  Scenario: [01D4TWXFJSYV9XTNMKKBERS4VT] Set the channel buffer size to 0
    Given [01D4TWXFJSYV9XTNMKKBERS4VT] the backend Processor echoes back the request immediately
    When [01D4TWXFJSYV9XTNMKKBERS4VT] the client sends 3 requests
    Then [01D4TWXFJSYV9XTNMKKBERS4VT] all replies are received successfully

