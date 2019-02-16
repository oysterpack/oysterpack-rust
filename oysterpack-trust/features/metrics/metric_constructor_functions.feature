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

  Scenario: [01D3VGSGCP9ZN9BX3BTB349FRJ] Register a collector that internally created metrics using the metric constructor functions
    Given [01D3VGSGCP9ZN9BX3BTB349FRJ-1] the collector is registered
    When [01D3VGSGCP9ZN9BX3BTB349FRJ-2] the collector's descriptors are retrieved
    Then [01D3VGSGCP9ZN9BX3BTB349FRJ-3] all of the collector's descriptors are contained within the set of all descriptors retrieved from the metric registry.
