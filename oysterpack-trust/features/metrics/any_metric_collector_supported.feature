Feature: [01D3JAHR4Z02XTJGTNE4D63VRT] MetricRegistry supports any metrics Collector

  Scenario: [01D3JAKE384RJA4FM9NJJNDPV6] registering a collector
    Given [01D3JAKE384RJA4FM9NJJNDPV6-1] a collector
    When [01D3JAKE384RJA4FM9NJJNDPV6-2] it is registered
    Then [01D3JAKE384RJA4FM9NJJNDPV6-3] it's Desc(s) can be retrieved
    And [01D3JAKE384RJA4FM9NJJNDPV6-4] it's returned in the set of registered Collector(s)
    And [01D3JAKE384RJA4FM9NJJNDPV6-5] it's metrics can be gathered

  Scenario: [01D3JB2EJG4VKFV0522C3T6QDV] registering a duplicate collector
    Given [01D3JB2EJG4VKFV0522C3T6QDV-1] a collector is registered
    When [01D3JB2EJG4VKFV0522C3T6QDV-2] a collector with a duplicate Desc is registered
    Then [01D3JB2EJG4VKFV0522C3T6QDV-3] the collector will fail to register

  Scenario: [01D3JC3SE5Q98MQ96ZAGHVW7RF] registering multiple collectors
    Given [01D3JC3SE5Q98MQ96ZAGHVW7RF-1] multiple collectors are registered
    When [01D3JC3SE5Q98MQ96ZAGHVW7RF-2] Desc(s) are retrieved
    Then [01D3JC3SE5Q98MQ96ZAGHVW7RF-3] all of the collectors' Desc(s) are included
    When [01D3JC3SE5Q98MQ96ZAGHVW7RF-4] metrics are gathered
    Then [01D3JC3SE5Q98MQ96ZAGHVW7RF-5] metrics for all of the collector(s) Desc(s) are included
    When [01D3JC3SE5Q98MQ96ZAGHVW7RF-6] metrics are gathered using the collector(s) Desc ids
    Then [01D3JC3SE5Q98MQ96ZAGHVW7RF-7] metrics for all of the collector(s) Desc(s) are included
