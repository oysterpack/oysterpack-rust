Feature: [01D5J1EQ3W3M40892VXBKSYY0Q] ReqRep clients are globally registered using ReqRepId as the registry key

  Scenario: [01D5J1HJMGKN7AF39DPP4TBYRE] Register a ReqRep service using a unique ReqRepId
    Then [01D5J1HJMGKN7AF39DPP4TBYRE-1] the client is successfully registered
    And [01D5J1HJMGKN7AF39DPP4TBYRE-2] the client can be retrieved via its ReqRepId

  Scenario: [01D5J244J52Y4A7WGZ67ZNP0RS] Try to register 2 ReqRep services using the same ReqRepId
    Given [01D5J244J52Y4A7WGZ67ZNP0RS] the first ReqRep service successfully registers
    When [01D5J244J52Y4A7WGZ67ZNP0RS] the second service tries to register using the same ReqRepId
    Then [01D5J244J52Y4A7WGZ67ZNP0RS] the service fails to register with a `ClientRegistrationError::ClientAlreadyRegistered` error