# EvidenceAngel

EvidenceAngel is a new tool in the Angel-suite to collect test evidence from both manual and automated testing.

## Requirements

EvidenceAngel should accept evidence in the forms of:

- Text
- Images
- Files
- HTTP requests and responses
- potentially more in the future...

EvidenceAngel should allow a single evidence package to contain:

- Package metadata
    - Title
    - Authors
- Test Cases
    - Case Title
    - Case Execution Date and Time
    - Case Evidence

EvidenceAngel allows users to:

- Create a new package
- Create a new test case
- Add evidence to a test case
- (GUI only) Edit evidence in a test case
- Delete evidence from a test case

EvidenceAngel allows data to be exported into a variety of interchange formats:

- HTML document
- Excel file (with sheets per test case)
