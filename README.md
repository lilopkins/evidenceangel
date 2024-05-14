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

EvidenceAngel should allow users to:

- [x] Create a new package
- [x] Create a new test case
- [x] Add evidence to a test case
- [ ] (GUI only) Edit evidence in a test case
- [x] Delete evidence from a test case
- [ ] (GUI only) Annotate evidence
    - [ ] Highlighting parts of images and files
    - [ ] Writing comments

EvidenceAngel should allow data to be exported into a variety of interchange formats:

- [ ] PDF of entire package
- [ ] PDF of single test case
- [ ] Excel file of entire package (with sheets per test case)
