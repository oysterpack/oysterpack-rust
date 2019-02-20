Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] Collectors can be retrieved selectively by applying filters

  Scenario: [01D3PX3BGCMV2PS6FDXHH0ZEB1] Select collectors that match a filter function
    Given [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors are registered
    When [01D3PX3BGCMV2PS6FDXHH0ZEB1] retrieving collectors
    Then [01D3PX3BGCMV2PS6FDXHH0ZEB1] collectors matching the filter are returned

  Scenario: [01D44BJHNK2SBM7DBD6J3NWSY3] Select collectors for multiple MetricId(s)
    Given [01D44BJHNK2SBM7DBD6J3NWSY3] collectors are registered
    When [01D44BJHNK2SBM7DBD6J3NWSY3] retrieving collectors
    Then [01D44BJHNK2SBM7DBD6J3NWSY3] collectors containing Desc names matching the MetricId(s) are returned

  Scenario: [01D44BHGQVNTCMK7YXM2F0W65K] Select collectors for multiple MetricId(s) containing some matching MetricId(s)
    Given [01D44BHGQVNTCMK7YXM2F0W65K] collectors are registered
    When [01D44BHGQVNTCMK7YXM2F0W65K] retrieving collectors
    Then [01D44BHGQVNTCMK7YXM2F0W65K] collectors containing Desc names matching the MetricId(s) are returned

  Scenario: [01D44BHVAVQXV9WHA6CYGVB7V6] Select collectors for multiple MetricId(s) containing no matching MetricId(s)
    Given [01D44BHVAVQXV9WHA6CYGVB7V6] collectors are registered
    When [01D44BHVAVQXV9WHA6CYGVB7V6] retrieving collectors for MetricId(s)
    Then [01D44BHVAVQXV9WHA6CYGVB7V6] no collectors are returned

  Scenario: [01D44BJ3MGK6GMNJV8KAFSNFH9] Select collectors passing in an empty MetricId slice
    Given [01D44BJ3MGK6GMNJV8KAFSNFH9] collectors are registered
    When [01D44BJ3MGK6GMNJV8KAFSNFH9] retrieving collectors
    Then [01D44BJ3MGK6GMNJV8KAFSNFH9] no collectors are returned

  Scenario: [01D44BJV9VR2RWBARMBS1A0MXC] Select collectors for a MetricId
    Given [01D44BJV9VR2RWBARMBS1A0MXC] collectors are registered
    When [01D44BJV9VR2RWBARMBS1A0MXC] retrieving collector
    Then [01D44BJV9VR2RWBARMBS1A0MXC] collector is returned

  Scenario: [01D44BK3DYBM5JJJMBVXK36S49] Select collectors for a MetricId that is not registered
    Given [01D44BK3DYBM5JJJMBVXK36S49] collectors are registered
    When [01D44BK3DYBM5JJJMBVXK36S49] retrieving collector
    Then [01D44BK3DYBM5JJJMBVXK36S49] no collector is returned

  Scenario: [01D44BKCEMFFZSXPHZY2SMXEXC] Select collectors for DescId(s)
    Given [01D44BKCEMFFZSXPHZY2SMXEXC] collectors are registered
    When [01D44BKCEMFFZSXPHZY2SMXEXC] retrieving collectors
    Then [01D44BKCEMFFZSXPHZY2SMXEXC] collectors containing matching DescId(s) are returned

  Scenario: [01D44BKW1E97TGFJGE23FK654K] Select collectors for DescId(s) containing some that are not registered
    Given [01D44BKW1E97TGFJGE23FK654K] collectors are registered
    When [01D44BKW1E97TGFJGE23FK654K] retrieving collectors
    Then [01D44BKW1E97TGFJGE23FK654K] collectors containing matching DescId(s) are returned

  Scenario: [01D44BM35C61QE76Q2JGKGBKV7] Select collectors for DescId(s) that are not registered
    Given [01D44BM35C61QE76Q2JGKGBKV7] collectors are registered
    When [01D44BM35C61QE76Q2JGKGBKV7] retrieving collectors
    Then [01D44BM35C61QE76Q2JGKGBKV7] no collectors are returned

  Scenario: [01D44BMDK667A9QNFMQ9H89T95] Select collectors with empty DescId slice
    Given [01D44BMDK667A9QNFMQ9H89T95] collectors are registered
    When [01D44BMDK667A9QNFMQ9H89T95] retrieving collectors
    Then [01D44BMDK667A9QNFMQ9H89T95] no collectors are returned

  Scenario: [01D44BMWHBX0BY1JVRZHGA78HM] Find collector by metric DescId
    Given [01D44BMWHBX0BY1JVRZHGA78HM] collectors are registered
    When [01D44BMWHBX0BY1JVRZHGA78HM] find collector by DescId
    Then [01D44BMWHBX0BY1JVRZHGA78HM] the collector is returned

  Scenario: [01D44BN406V10VRCBGWM4PBDTX] Find collector by metric DescId that is not registered
    Given [01D44BN406V10VRCBGWM4PBDTX] collectors are registered
    When [01D44BN406V10VRCBGWM4PBDTX] find collector by DescId
    Then [01D44BN406V10VRCBGWM4PBDTX] no collector is returned