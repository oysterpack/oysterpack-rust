Feature: [01D4Z9P9VVHP7NC4MWV6JQ5XBM] Backend service processing is executed async

  - Backend services receive messages in a non-blocking fashion, i.e., threads are not blocked waiting for messages.
  - Processor::process() is designed to return an async task which is scheduled to run on the the same Executor thread that
    is running the service task

  Scenario: [01D4Z9S1E16YN6YSFXEHHY1KQT] Startup 10 ReqRep services on a single-threaded Executor
    Given [01D4Z9S1E16YN6YSFXEHHY1KQT] 10 ReqRep services are started
    When [01D4Z9S1E16YN6YSFXEHHY1KQT] send 10 requests to each service on the same single-threaded Executor
    Then [01D4Z9S1E16YN6YSFXEHHY1KQT] all requests process successfully

  Scenario: [01D4ZAMSJRS22CF9FN2WFZGMZM] Startup 10 ReqRep services on a mutli-threaded Executor
    Given [01D4ZAMSJRS22CF9FN2WFZGMZM] 10 ReqRep services are started
    When [01D4ZAMSJRS22CF9FN2WFZGMZM] send 10 requests to each service on the same Executor
    Then [01D4ZAMSJRS22CF9FN2WFZGMZM] all requests process successfully

  Scenario: [01D4ZANG5S4SZ07AZ0QJ0A8XJW] Startup 10 ReqRep services on one Executor and send requests on different Executor
    Given [01D4ZANG5S4SZ07AZ0QJ0A8XJW] 10 ReqRep services are started
    When [01D4ZANG5S4SZ07AZ0QJ0A8XJW] send 10 requests to each service on a different Executor
    Then [01D4ZANG5S4SZ07AZ0QJ0A8XJW] all requests process successfully
