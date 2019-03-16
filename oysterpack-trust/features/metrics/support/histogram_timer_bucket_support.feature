Feature: [01D63TZ7T07W3K8K6QTR1CN9HH] Creating histogram timer buckets based on time durations

  - metrics::timer_buckets()
  - metrics::exponential_timer_buckets()
  - metrics::linear_timer_buckets()

  Scenario: [01D63V3G7Q3S9F1JV4A3TJYJQH] metrics::timer_buckets()
    Then [01D63V3G7Q3S9F1JV4A3TJYJQH] valid timer buckets are created

  Scenario: [01D63V8E55T03C161QTGHP0THK] metrics::exponential_timer_buckets()
    Then [01D63V8E55T03C161QTGHP0THK] valid timer buckets are created

  Scenario: [01D63V9Z9J1HC5NBGM64JJMXXZ] metrics::linear_timer_buckets()
    Then [01D63V9Z9J1HC5NBGM64JJMXXZ] valid timer buckets are created

  Rule: the starting bucket upper bound must be greater than 0 ns

  Scenario: [01D63VEPJZMH40CKH872E1CB8X] metrics::timer_buckets(): start = Duration::from_millis(0)
    Then [01D63VEPJZMH40CKH872E1CB8X] timer buckets failed to be created

  Scenario: [01D63VEX6RSDMCQ8P83WEX0ND6] metrics::exponential_timer_buckets(): start = Duration::from_millis(0)
    Then [01D63VEX6RSDMCQ8P83WEX0ND6] timer buckets failed to be created

  Scenario: [01D63VF3AZHA0SH3KYEDZC1W4P] metrics::linear_timer_buckets(): start = Duration::from_millis(0)
    Then [01D63VF3AZHA0SH3KYEDZC1W4P] timer buckets failed to be created

  Rule: at least 1 bucket must be specified

  Scenario: [01D63W11BGS89YFSZRK7A4JHP7] metrics::timer_buckets() with empty durations
    Then [01D63W11BGS89YFSZRK7A4JHP7] timer buckets failed to be created

  Rule: tmetrics::exponential_timer_buckets(): factor must be > 1

  Scenario: [01D63W0QCYEB6P6Z4YP4ZZ75C9] metrics::exponential_timer_buckets(): factor = 1.0
    Then [01D63W0QCYEB6P6Z4YP4ZZ75C9] timer buckets failed to be created

  Rule: metrics::linear_timer_buckets(): width must be > 0 ns

  Scenario: [01D63W389MA1H1HQJ45Y7GPXM5] metrics::linear_timer_buckets(): width = Duration::from_millis(0)
    Then [01D63W389MA1H1HQJ45Y7GPXM5] timer buckets failed to be created