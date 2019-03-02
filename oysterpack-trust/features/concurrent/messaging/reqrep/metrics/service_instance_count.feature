Feature: [01D4ZHRS7RV42RXN1R83Q8QDPA] The number of running ReqRep service backend instances will be tracked

  Counts are tracked per ReqRepId

  Scenario: [01D4ZJJJCHMHAK12MGEY5EF6VF] start up 10 instances of a ReqRep service using the same ReqRepId
    Given [01D4ZJJJCHMHAK12MGEY5EF6VF] the reply returns the ServiceInstanceId
    Then [01D4ZJJJCHMHAK12MGEY5EF6VF-1] the service instance count will be 10
    When [01D4ZJJJCHMHAK12MGEY5EF6VF-2] requests are sent using each of the ReqRep clients
    Then [01D4ZJJJCHMHAK12MGEY5EF6VF-3] replies from each of the clients will return different ServiceInstanceIds
    When [01D4ZJJJCHMHAK12MGEY5EF6VF-4] clients fall out of scope
    Then [01D4ZJJJCHMHAK12MGEY5EF6VF-5] the service instance count will decrease accordingly