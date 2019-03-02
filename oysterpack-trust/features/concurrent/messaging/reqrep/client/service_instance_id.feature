Feature: [01D4RXDDVA5DWTWF45KBTJTK5Z] Each ReqRep client is linked with a backend service instance identified by a ServiceInstanceId

  - A ReqRep service backend is assigned a unique ServiceInstanceId. Thus, if 2 ReqRep clients have the same ServiceInstanceId,
    then they are linked to the same backend service instance.

  Scenario: [01D4RXFJNJ70CTFV5RE1TN8S6Z] Startup multiple ReqRep services
    Then [01D4RXFJNJ70CTFV5RE1TN8S6Z] each ReqRep client will have a unique ServiceInstanceId