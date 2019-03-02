Feature: [01D4ZS3J72KG380GFW4GMQKCFH] Message processing timer metrics are collected

  Scenario: [01D4ZS5W2QFN1DF2TTYW0J3RH5] Gather metrics for message processing
    Given [01D4ZS5W2QFN1DF2TTYW0J3RH5] 10 messages were sent
    When [01D4ZS5W2QFN1DF2TTYW0J3RH5] metrics are gathered
    Then [01D4ZS5W2QFN1DF2TTYW0J3RH5] timing metrics for all 10 requests are present