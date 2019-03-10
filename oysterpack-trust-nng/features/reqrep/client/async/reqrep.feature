Feature: [01D5J2BK1ZCH1NQJ1JNA3R305M] The nng client interface is ReqRep

  - the Client type is `ReqRep<Message, Result<Message, RequestError>>`

  Scenario: [01D5J2CS8VFK0547YZ3B8368YC] Submit 100 requests
    Given [01D5J2CS8VFK0547YZ3B8368YC] a server is running
    When [01D5J2CS8VFK0547YZ3B8368YC] async requests are sent in separate async tasks
    Then [01D5J2CS8VFK0547YZ3B8368YC] all responses are received from separate async tasks