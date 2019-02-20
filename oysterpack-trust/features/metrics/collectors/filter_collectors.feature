Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] Collectors can be retrieved selectively by applying filters

  Background:
    Given [01D46BWEKHMHZGSAZF4QQCZ0RV] collectors are registered

  Scenario: [01D3PX3BGCMV2PS6FDXHH0ZEB1] Select collectors using a filter
    When [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors are retrieved using a filter
    Then [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors matching the filter are returned

  Scenario: [01D3PX3NRADQPMS95EB5C7ECD7] Select collectors for MetricId(s)
    When [01D3PX3NRADQPMS95EB5C7ECD7] retrieving collectors for MetricId(s)
    Then [01D3PX3NRADQPMS95EB5C7ECD7] the returned collectors contain descriptors that match the MetricId(s)

  Scenario: [01D44BHGQVNTCMK7YXM2F0W65K] Select collectors for MetricId(s) containing some non-matching MetricId(s)
    When [01D44BHGQVNTCMK7YXM2F0W65K] retrieving collectors
    Then [01D44BHGQVNTCMK7YXM2F0W65K] collectors containing Desc names matching the MetricId(s) are returned

  Scenario: [01D44BHVAVQXV9WHA6CYGVB7V6] Select collectors for multiple MetricId(s) containing no matching MetricId(s)
    When [01D44BHVAVQXV9WHA6CYGVB7V6] retrieving collectors for MetricId(s)
    Then [01D44BHVAVQXV9WHA6CYGVB7V6] no collectors are returned

  Scenario: [01D44BJ3MGK6GMNJV8KAFSNFH9] Select collectors passing in an empty MetricId slice
    When [01D44BJ3MGK6GMNJV8KAFSNFH9] retrieving collectors
    Then [01D44BJ3MGK6GMNJV8KAFSNFH9] no collectors are returned

  Scenario: [01D44BJV9VR2RWBARMBS1A0MXC] Select collectors for a MetricId
    When [01D44BJV9VR2RWBARMBS1A0MXC] retrieving collector
    Then [01D44BJV9VR2RWBARMBS1A0MXC] collector is returned

  Scenario: [01D44BK3DYBM5JJJMBVXK36S49] Select collectors for a MetricId that is not registered
    When [01D44BK3DYBM5JJJMBVXK36S49] retrieving collector
    Then [01D44BK3DYBM5JJJMBVXK36S49] no collector is returned

  Scenario: [01D45SST98R0VJY58JM2X1WN7E] Select collectors for DescId(s)
    When [01D45SST98R0VJY58JM2X1WN7E] retrieving collectors for DescId(s)
    Then [01D45SST98R0VJY58JM2X1WN7E] the returned collectors contain descriptors that match the Descid(s)

  Scenario: [01D44BKW1E97TGFJGE23FK654K] Select collectors for DescId(s) containing some that are not registered
    When [01D44BKW1E97TGFJGE23FK654K] retrieving collectors
    Then [01D44BKW1E97TGFJGE23FK654K] collectors containing matching DescId(s) are returned

  Scenario: [01D44BM35C61QE76Q2JGKGBKV7] Select collectors for DescId(s) that are not registered
    When [01D44BM35C61QE76Q2JGKGBKV7] retrieving collectors
    Then [01D44BM35C61QE76Q2JGKGBKV7] no collectors are returned

  Scenario: [01D44BMDK667A9QNFMQ9H89T95] Select collectors with empty DescId slice
    When [01D44BMDK667A9QNFMQ9H89T95] retrieving collectors
    Then [01D44BMDK667A9QNFMQ9H89T95] no collectors are returned

  Scenario: [01D44BMWHBX0BY1JVRZHGA78HM] Find collector by metric DescId
    When [01D44BMWHBX0BY1JVRZHGA78HM] find collector by DescId
    Then [01D44BMWHBX0BY1JVRZHGA78HM] the collector is returned

  Scenario: [01D44BN406V10VRCBGWM4PBDTX] Find collector by metric DescId that is not registered
    When [01D44BN406V10VRCBGWM4PBDTX] find collector by DescId
    Then [01D44BN406V10VRCBGWM4PBDTX] no collector is returned