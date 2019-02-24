Feature: [01D3M9X86BSYWW3132JQHWA3AT] Gathered metrics can be encoded in prometheus compatible text format

  Scenario: [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] text encoding gathered metrics
    Given [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] metrics are registered
    When [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] metrics are gathered and text encoded
    Then [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] the text encoded metrics contain the gathered metrics

