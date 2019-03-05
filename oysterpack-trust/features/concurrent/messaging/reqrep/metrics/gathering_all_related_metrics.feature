Feature: [01D4ZS9FX0GZZRG9RF072XGBQD] All ReqRep related metrics can be gathered via reqrep::gather_metrics()

  Scenario: [01D4ZSJ5XPDVKG33NXDE6TP6QX] send requests and then gather metrics
    Given [01D4ZSJ5XPDVKG33NXDE6TP6QX] a ReqRep client
    When [01D4ZSJ5XPDVKG33NXDE6TP6QX] multiple requests are sent to each
    Then [01D4ZSJ5XPDVKG33NXDE6TP6QX] metrics should be consistent with the number of requests sent




