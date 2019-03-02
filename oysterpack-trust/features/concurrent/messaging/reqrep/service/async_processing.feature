Feature: [01D4Z9P9VVHP7NC4MWV6JQ5XBM] Backend service processing is executed async

  - Backend services receive messages to process in a non-blocking fashion, i.e., threads are not blocked waiting for messages.
  - Processor::process() is designed to return an async task which is scheduled to run on the the same Executor thread that
    is running the service task

  Scenario: [01D4Z9S1E16YN6YSFXEHHY1KQT] Startup 10 ReqRep services on a single-threaded Executor
    When [01D4Z9S1E16YN6YSFXEHHY1KQT] send 10 requests to each service on the same single-threaded Executor
    Then [01D4Z9S1E16YN6YSFXEHHY1KQT] all requests process successfully

  Scenario: [01D4ZAMSJRS22CF9FN2WFZGMZM] Startup 10 ReqRep services on a mutli-threaded Executor
    When [01D4ZAMSJRS22CF9FN2WFZGMZM] send 10 requests to each service on the same Executor
    Then [01D4ZAMSJRS22CF9FN2WFZGMZM] all requests process successfully

  Scenario: [01D4ZANG5S4SZ07AZ0QJ0A8XJW] Startup 10 ReqRep services on one Executor
    When [01D4ZANG5S4SZ07AZ0QJ0A8XJW] send 10 requests to each service on a different Executor
    Then [01D4ZANG5S4SZ07AZ0QJ0A8XJW] all requests process successfully

  Scenario: [01D4ZGXQ27F3P7MXDW20K4RGR9] Processor message processing task panics
    When [01D4ZGXQ27F3P7MXDW20K4RGR9] the request processing task panics
    Then [01D4ZGXQ27F3P7MXDW20K4RGR9-1] the ReqRep service backend task will be terminated
    And [01D4ZGXQ27F3P7MXDW20K4RGR9-2] the ReqRep client request will fail with a ChannelError::Disconnected
