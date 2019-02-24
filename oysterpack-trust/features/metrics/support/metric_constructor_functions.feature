Feature: [01D3VG4CEEPF8NNBM348PKRDH3] Constructor functions are provided for each of the supported metrics.

  - counter constructor functions
    - metrics::new_counter()
    - metrics::new_counter_vec()
    - metrics::new_int_counter()
    - metrics::new_int_counter_vec()
  - gauge constructor functions
    - metrics::new_gauge()
    - metrics::new_gauge_vec()
    - metrics::new_int_gauge()
    - metrics::new_int_gauge_vec()
  - histogram constructor functions
    - metrics::new_histogram()
    - metrics::new_histogram_vec()

  Scenario: [01D3VGSGCP9ZN9BX3BTB349FRJ] Construct a new counter and register it
    Then [01D3VGSGCP9ZN9BX3BTB349FRJ] metric is successfully registered

  Scenario: [01D4G02JDYSR3PBY1MTFZQNJ46] Construct a new int counter and register it
    Then [01D4G02JDYSR3PBY1MTFZQNJ46] metric is successfully registered

  Scenario: [01D4G02N1YCHM8N9DYED2P8SRV] Construct a new counter vec and register it
    Then [01D4G02N1YCHM8N9DYED2P8SRV] metric is successfully registered

  Scenario: [01D4G02NNQBW5NC2B5R6QPC38Z] Construct a new int counter vec and register it
    Then [01D4G02NNQBW5NC2B5R6QPC38Z] metric is successfully registered

  Scenario: [01D4G02P8P3ZPSDHJ058479441] Construct a new gauge and register it
    Then [01D4G02P8P3ZPSDHJ058479441] metric is successfully registered

  Scenario: [01D4G02PV54CR9MDHYNYP7G69M] Construct a new int gauge and register it
    Then [01D4G02PV54CR9MDHYNYP7G69M] metric is successfully registered

  Scenario: [01D4G02QC5A2J0CF6TG0863N1J] Construct a new gauge vec and register it
    Then [01D4G02QC5A2J0CF6TG0863N1J] metric is successfully registered

  Scenario: [01D4G02QYHTQ6N5EP4XADW67ZG] Construct a new int gauge vec and register it
    Then [01D4G02QYHTQ6N5EP4XADW67ZG] metric is successfully registered

  Scenario: [01D4G02RFAQCQMM3C7WH7VZECG] Construct a new histogram and register it
    Then [01D4G02RFAQCQMM3C7WH7VZECG] metric is successfully registered

  Scenario: [01D4G04V16ZSWAKBMADJ5M2ZS9] Construct a new histogram timer and register it
    Then [01D4G04V16ZSWAKBMADJ5M2ZS9] metric is successfully registered

  Scenario: [01D4G04MZ2VXN226H8R2CRASE5] Construct a new histogram vec and register it
    Then [01D4G04MZ2VXN226H8R2CRASE5] metric is successfully registered

  Scenario: [01D4G04E4XCY5SFC0XAYSMH9G6] Construct a new histogram timer vec and register it
    Then [01D4G04E4XCY5SFC0XAYSMH9G6] metric is successfully registered
