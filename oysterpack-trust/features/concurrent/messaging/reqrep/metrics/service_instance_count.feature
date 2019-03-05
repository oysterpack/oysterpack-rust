Feature: [01D4ZHRS7RV42RXN1R83Q8QDPA] The number of running ReqRep service backend instances will be tracked

  Counts are tracked per ReqRepId

  Scenario: [01D4ZJJJCHMHAK12MGEY5EF6VF] start up 10 instances of a ReqRep service using the same ReqRepId
    Then [01D4ZJJJCHMHAK12MGEY5EF6VF-1] the service instance count will be 10
    When [01D4ZJJJCHMHAK12MGEY5EF6VF-2] 3 clients fall out of scope
    Then [01D4ZJJJCHMHAK12MGEY5EF6VF-3] the service instance count will be 7