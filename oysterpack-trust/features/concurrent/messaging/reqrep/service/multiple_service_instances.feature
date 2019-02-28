Feature: [01D4RNXZ255JZ4523VQG70TBEZ] Multiple instances of a service with the same ReqRepId can be created

  - A ReqRep service backend is assigned a unique ServiceInstanceId
  - Two ReqRep service clients are connected to the same service backend instance if they have the same ServiceInstanceId

  Scenario: [01D4RS6T7X51WTACRTXNH29068] Start multiple instances of a ReqRep service
    Then [01D4RS6T7X51WTACRTXNH29068] each ReqRep client will be assigned a unique ServiceInstanceId