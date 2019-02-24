Feature: [01D43V2S6HBV642EKK5YGJNH32] LabelId can be used for constant and variable label names.

  - LabelId(s) are ULID(s)
  - Valid label names in prometheus must not start with number. Thus LabelId names are prefixed with 'L'.

  Scenario: [01D4GFESB7GQY04JGR0CQ5S6TW] Define LabelId as a constant
    Then [01D4GFESB7GQY04JGR0CQ5S6TW] LabelId can be used as a constant

  Scenario: [01D4GFF0KYKX79T919MTG4NY4S] Register a metric with constant label pairs using LabelId as the label name
    Then [01D4GFF0KYKX79T919MTG4NY4S] the registered metric Desc label can be parsed back into a LabelId

