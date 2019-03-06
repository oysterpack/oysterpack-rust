Feature: [01D585SEWBEKBBR0ZY3C5GR7A6] Processor is notified via `Processor::panicked()` if a panic occurred while processing the request.

  - The default `Processor::panicked()` implementation simply cascades the panic.

  Scenario: [01D4ZGXQ27F3P7MXDW20K4RGR9] Processor::process() panics using default behavior
    When [01D4ZGXQ27F3P7MXDW20K4RGR9] the request processing task panics
    Then [01D4ZGXQ27F3P7MXDW20K4RGR9-1] the ReqRep service backend task will terminate
    And [01D4ZGXQ27F3P7MXDW20K4RGR9-2] the ReqRep client is not usable usable

  Scenario: [01D586H94GS723PJ2R1W4PTR6B] Processor::process() panics but Processor is designed to recover from the panic
    When [01D586H94GS723PJ2R1W4PTR6B] the request processing task panics
    Then [01D586H94GS723PJ2R1W4PTR6B-1] the ReqRep service backend task will continue to run
    And [01D586H94GS723PJ2R1W4PTR6B-2] the ReqRep client is still usable
    And [01D586H94GS723PJ2R1W4PTR6B-3] timer metrics are not reported for requests that panicked while being processed