Feature: [01D3JAHR4Z02XTJGTNE4D63VRT] Any `prometheus::core::Collector` can be registered

  Scenario: [01D3JAKE384RJA4FM9NJJNDPV6] Registering a collector
    Given [01D3JAKE384RJA4FM9NJJNDPV6] a collector
    When [01D3JAKE384RJA4FM9NJJNDPV6] it is registered along with other metrics
    Then [01D3JAKE384RJA4FM9NJJNDPV6-1] it can be retrieved from the registry
    And [01D3JAKE384RJA4FM9NJJNDPV6-2] its metric descriptors can be retrieved from the registry
    And [01D3JAKE384RJA4FM9NJJNDPV6-3] its metrics are gathered