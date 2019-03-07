Feature: [01D59X5KJ7Q72C2F2FP2VYVGS1] ReqRep related metrics can be easily gathered

  Scenario: [01D59X6B8A40S941CMTRKWAMAB] Start up multiple ReqRep services
    Given [01D59X6B8A40S941CMTRKWAMAB-1] requests have been submitted
    When [01D59X6B8A40S941CMTRKWAMAB-2] all metrics are gathered
    Then [01D59X6B8A40S941CMTRKWAMAB-3] metrics were returned for each ReqRep service
    And [01D59X6B8A40S941CMTRKWAMAB-4] service instance counts are consistent with the gathered metrics
    And [01D59X6B8A40S941CMTRKWAMAB-5] requests sent counts are consistent with the gathered metrics
    And [01D59X6B8A40S941CMTRKWAMAB-6] timer metrics are consistent with the gathered metrics


