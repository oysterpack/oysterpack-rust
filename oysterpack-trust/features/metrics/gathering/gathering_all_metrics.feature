Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering all metrics

  Scenario: [01D3PPPT1ZNXPKKWM29R14V5ZT] Gathering all metrics
    Given [01D3PPPT1ZNXPKKWM29R14V5ZT] metrics are registered
    When [01D3PPPT1ZNXPKKWM29R14V5ZT] metrics are gathered
    Then [01D3PPPT1ZNXPKKWM29R14V5ZT] metrics are returned for all registered metric descriptors