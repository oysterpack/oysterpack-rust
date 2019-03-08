Feature: [01D4ZAQBNT7MF2E0PWW77BJ6HS] The backend service Processor lifecycle hooks are invoked on service startup and shutdown

  - when the backend service starts up and before processing any messages, the Processor::init() lifecycle method is called
  - when the backend service is shutdown, Processor::destroy() is invoked

  Scenario: [01D4ZAX312JQKD2T104CGQXZJF] Starting and stopping a ReqRep service
    Given [01D4ZAX312JQKD2T104CGQXZJF] a Processor that emits lifecycle messages on a channel
    When [01D4ZAX312JQKD2T104CGQXZJF] the ReqRep service is started and then shutdown
    Then [01D4ZAX312JQKD2T104CGQXZJF] lifecycle messages are recieved from the Processor