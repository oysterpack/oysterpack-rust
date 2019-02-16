Feature: [01D3JAHR4Z02XTJGTNE4D63VRT] The metric registry supports any `prometheus::core::Collector`

  Scenario: [01D3JAKE384RJA4FM9NJJNDPV6] registering a collector
    When [01D3JAKE384RJA4FM9NJJNDPV6-1] a collector is registered
    Then [01D3JAKE384RJA4FM9NJJNDPV6-2] its Desc(s) can be retrieved
    And [01D3JAKE384RJA4FM9NJJNDPV6-3] its returned in the set of registered Collector(s)
    And [01D3JAKE384RJA4FM9NJJNDPV6-4] its metrics can be gathered