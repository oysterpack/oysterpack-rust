Feature: [01D5J3S1EE020NPEGXHJWPNHCT] SocketConfig may be specified when registering the nng Client

  The socket is always configured as non-blocking. The client may configure the following additional socket options:
  - resend time
  - reconnect min / max times

  Scenario: [01D5J46H2E2EBBCSH464H1KMTZ] Register a client using socket config defaults with server running
    Given [01D5J46H2E2EBBCSH464H1KMTZ] the client is registered when nng server is running
    Then [01D5J46H2E2EBBCSH464H1KMTZ] the client can successfully send requests

  Scenario: [01D5J4T7XAVC26MMBN4SVA94YP] Register a client using socket config defaults before the server is started
    Given [01D5J4T7XAVC26MMBN4SVA94YP-1] the client is registered when the server is not running
    When [01D5J4T7XAVC26MMBN4SVA94YP-2] the client sends a request
    And [01D5J4T7XAVC26MMBN4SVA94YP-3] then the server is started
    Then [01D5J4T7XAVC26MMBN4SVA94YP-4] the client request completes successfully

  Scenario: [01D5J4T7XAVC26MMBN4SVA94YP] Register a client using socket config defaults and the server is restarted
    Given [01D5J4T7XAVC26MMBN4SVA94YP-1] the client is registered when the server is running
    And [01D5J4T7XAVC26MMBN4SVA94YP-2] then server is stopped
    And [01D5J4T7XAVC26MMBN4SVA94YP-3] then the client sends a request
    And [01D5J4T7XAVC26MMBN4SVA94YP-4] then the server is started
    Then [01D5J4T7XAVC26MMBN4SVA94YP-5] the client request completes

  Scenario: [01D5J668QCTDJM8FE51VCHTNCT] Client connects using socket config defaults - server is restarted while processing a request
    Given [01D5J668QCTDJM8FE51VCHTNCT-1] the client is registered when nng server is running
    And [01D5J668QCTDJM8FE51VCHTNCT-2] the client sends a request that requires 1 sec to process
    And [01D5J668QCTDJM8FE51VCHTNCT-3] the server is restarted
    Then [01D5J668QCTDJM8FE51VCHTNCT-4] the client request is automatically resent
    And [01D5J668QCTDJM8FE51VCHTNCT-5] the client request succeeds

