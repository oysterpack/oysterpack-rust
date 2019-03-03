Feature: [01D51SE8P5JWRXZ4RFBDQFR6GW] Each request is assigned a unique MessageId for tracking purposes.

  - the MessageId is meant to be used for tracking purposes
  - the MessageId is made available to the client via ReplyReceiver

  Scenario: [01D51SXCCY1YNEG8JSBGW72SYF] Send 100 decoupled requests
    Then [01D51SXCCY1YNEG8JSBGW72SYF] all requests are assigned unique MessageId(s)
