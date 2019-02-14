Feature: [01D3M9X86BSYWW3132JQHWA3AT] Gathered metrics can be encoded in prometheus compatible text format

  Scenario: [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] gathering metrics
    Given [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-1] metrics are gathered
    When [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-2] metrics are text encoded
    Then [01D3M9ZJQSTWFFMKBR3Z2DXJ9N-3] the text encoding contains the gathered metrics

