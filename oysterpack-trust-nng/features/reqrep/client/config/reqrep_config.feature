Feature: [01D5J2VWG5KCP6P10K07QPXCCA] ReqRepConfig is required to register an nng Client

  Scenario: [01D5J39S53X89BWDHE43X3Y2VC] Register a new nng Client
    Given [01D5J39S53X89BWDHE43X3Y2VC] an nng server is running
    When [01D5J39S53X89BWDHE43X3Y2VC] the new Client is registered
    Then [01D5J39S53X89BWDHE43X3Y2VC-1] the Client is successfully registered
    And [01D5J39S53X89BWDHE43X3Y2VC-2] can successfully send requests