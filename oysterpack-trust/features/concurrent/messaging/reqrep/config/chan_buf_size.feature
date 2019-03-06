Feature: [01D4T5NV48PVFBC2R3Q80B6W72] The request channel buffer size is configurable

  - This is referring to the channel used by the ReqRep client to send requests to the backend service.
  - By default, the channel buffer size is 0.
    - The channel's capacity is equal to buffer + num-senders. In other words, each sender gets a guaranteed slot in the
      channel capacity, and on top of that there are buffer "first come, first serve" slots available to all senders.

  Scenario: [01D4T5RY9XDB6MYDE6X7R3X766] Use the default channel buffer size - send 10 requests from single ReqRep instance
    Given [01D4T5RY9XDB6MYDE6X7R3X766] a Processor that sleeps for 10 secs on each request
    When [01D4T5RY9XDB6MYDE6X7R3X766] the client sends 10 requests from a single task
    Then [01D4T5RY9XDB6MYDE6X7R3X766] the number of sent requests after 50 ms is 2
    # - the first request sent is immediately picked up for processing
    # - then the receiver is free to pick up the next message - 2 messages sent
    # - all other send attempts must wait until the backend processor takes the message from the receiver in order to progress

  Scenario: [01D52JKNS9FXDXQYPADGFWM3QK] Use the default channel buffer size - send 10 requests from 10 ReqRep instance
    Given [01D52JKNS9FXDXQYPADGFWM3QK] a Processor that sleeps for 10 secs on each request
    When [01D52JKNS9FXDXQYPADGFWM3QK] 10 client sends 10 requests from separate tasks
    Then [01D52JKNS9FXDXQYPADGFWM3QK] the number of sent requests after 50 ms is 11

  Scenario: [01D4T61JB50KNT3Y7VQ10VX2NR] Set the channel buffer size to 1 - send 10 requests from single ReqRep instance
    Given [01D4T61JB50KNT3Y7VQ10VX2NR] a Processor that sleeps for 10 secs on each request
    When [01D4T61JB50KNT3Y7VQ10VX2NR] the client sends 10 requests from a single task
    Then [01D4T61JB50KNT3Y7VQ10VX2NR] the number of sent requests after 50 ms is 3
    # - the first request sent is immediately picked up for processing
    # - the second request is picked up by the receiver
    # - the third request is put into the channel buffer

  Scenario: [01D52MG7FVEWE4HK6J05VRR49F] Set the channel buffer size to 1 - send 10 requests from 10 ReqRep instance
    Given [01D52MG7FVEWE4HK6J05VRR49F] a Processor that sleeps for 10 secs on each request
    When [01D52MG7FVEWE4HK6J05VRR49F] the client sends 10 requests from a single task
    Then [01D52MG7FVEWE4HK6J05VRR49F] the number of sent requests after 50 ms is 12

