Feature: [01D59WRTHWQRPC8DYMN76RJ5X0] Backend Processor panics are tracked as a metric.

  Scenario: [01D59WYME5Y6Z8TEHKPEH6ZFTR] Processor panics while processing a request - service terminates
    Then [01D59WYME5Y6Z8TEHKPEH6ZFTR] the panic counter will be incremented

  Scenario: [01D5DCAFHBYKKMP4BH89VADGNB] Processor panics while processing a request - service recovers and keeps running
    Then [01D5DCAFHBYKKMP4BH89VADGNB] the panic counter will be incremented