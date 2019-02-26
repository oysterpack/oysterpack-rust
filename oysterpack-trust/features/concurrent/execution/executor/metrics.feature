Feature: [01D4P0Q8M3ZAWCDH22VXHGN4ZX] Executor metrics can be collected

  Scenario: [01D4P0QFZ2YK0HYC74T9S74WXQ] Collect metrics for an individual Executor
    When [01D41GJ0WRB49AX2NX4T09BKA8] metrics are collected for an Executor
    Then [01D41GJ0WRB49AX2NX4T09BKA8] the metric counts will match the Executor counts

  Scenario: [01D4P0TGP2D9H4GAXZC1PKMQH3] Collect metrics for all registered Executor(s)
    Then [01D4P0TGP2D9H4GAXZC1PKMQH3] the metric counts will match the Executor counts