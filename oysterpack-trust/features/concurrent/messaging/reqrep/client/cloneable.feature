Feature: [01D4RW7WRVBBGTBZEQCXMFN51V] The ReqRep client can be shared by cloning it.

  Scenario: [01D4RW8V6K8HR8R1QR8DMN2AQC] Clone the ReqRep client and send requests from multiple threads
    Given [01D4RW8V6K8HR8R1QR8DMN2AQC] a ReqRep client and Executor
    When [01D4RW8V6K8HR8R1QR8DMN2AQC] the client is cloned
    Then [01D4RW8V6K8HR8R1QR8DMN2AQC] requests can be sent from each cloned client from multiple spawned tasks