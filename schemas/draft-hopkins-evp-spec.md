%%%
title = "Evidence Package Format Specification"
abbrev = "evp-spec"
ipr = "trust200902"
area = ""
workgroup = ""
keyword = ["evp", "evidence", "format", "specification"]
submissionType = "independent"

[seriesInfo]
name = "Internet-Draft"
value = "draft-hopkins-evp-spec-04"
stream = "independent"
status = "informational"

date = 2025-07-27T00:00:00Z

[[author]]
initials="L."
surname="Hopkins"
fullname="Lily Hopkins"
  [author.address]
  uri = "https://github.com/lilopkins"
  email = "lily@hpkns.uk"

[[author]]
inials="E."
surname="Turner"
fullname="Eden Turner"
  [author.address]
  uri = "https://github.com/Some-Birb7190"
  email = "somebirb7190@gmail.com"
%%%

.# Abstract

Taking evidence is a key part of any software testing process. This
specification defines a format which collects evidence together and
stores metadata and annotations in an organised fashion from both manual
and automated testing sources.

{mainmatter}

# Introduction

## Purpose

The purpose of this specification is to define a format for storage of
test evidence that:

* allows for basic collation of evidence;
* can store any kind of file type that might be produced;
* stores data compressed;
* stores related evidence together, but allows for dividing up by test
  case, and;
* is built upon widely available standards.

The format does not attempt to:

* act as an captioned archiving solution for other purposes, even if it
  may be suitable for them.

## Intended Audience

This specification is intended for those who might wish to write their
own implementation of the evidence package format. There are a number of
situations where writing an implementation may be desirable:

* in an automation tool that runs a number of operations to
  automatically test something, to produce an evidence package
  containing the results of the automated testing;
* in a manual evidence collection tool, where a user might want to
  collect evidence in a single, easy to manage place for later
  processing or sharing;
* in an analysis tool, to view, annotate, share and understand the
  evidence from previous testing;
* in a viewer, to view evidence that has been shared, for example from a
  testing team to a customer, or;
* any other situation where it may be desirable to collect test evidence
  and bundle it together for later.

## Changes from Previous Versions

This document forms the original specification.

# Terminology

The key words "**MUST**", "**MUST NOT**", "**REQUIRED**", "**SHALL**",
"**SHALL NOT**", "**SHOULD**", "**SHOULD NOT**", "**RECOMMENDED**",
"**NOT RECOMMENDED**", "**MAY**", and "**OPTIONAL**" in this document
are to be interpreted as described in BCP 14 [@!RFC2119] [@!RFC8174]
when, and only when, they appear in all capitals, as shown here.

# Specification

An evidence package is a structured ZIP archive [@!zip]. It **MUST**
contain the file "manifest.json", and the directories "media" and
"testcases" internally within the ZIP archive. This structure does not
need to be represented outside of the ZIP archive and as such the
internal structure does not need to be understood by an end-user of any
tool that works with evidence packages.

<reference anchor="zip" target="https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT">
    <front>
        <title>.ZIP File Format Specification</title>
        <author>
            <organization>PKWARE, Inc.</organization>
        </author>
        <date year="2022" month="November" day="01"/>
    </front>
</reference>

See (#example-archive) for an example of the file's internal structure.

## "manifest.json" File

The manifest.json file defines metadata relating to the entire package
of evidence. It **MUST** be a JSON [@!RFC8259] file with the following
elements:

| Element    | Condition | Type | Section | Description |
|------------|-----------|------|---|---|
| $schema    | Optional  | String | | The $schema element **MAY** point to a copy of the schema for the manifest. |
| metadata   | Mandatory | Object | (#manifest-metadata) | The metadata element stores package metadata. |
| custom_test_case_metadata | Mandatory | Object | (#manifest-custom-metadata) | Custom metadata fields for test cases in this package. |
| media      | Mandatory | Array | (#manifest-media) | The media element stores a list of media files that are stored in this evidence package. |
| test_cases | Mandatory | Array | (#manifest-test-cases) | The test_cases element stores a list of test cases. |

See an example manifest.json file in (#example-manifest).

### "metadata" Element {#manifest-metadata}

| Element | Condition | Type | Section | Description |
|---------|-----------|------|---|---|
| title   | Mandatory | String | | The name of the evidence package. |
| authors | Mandatory | Array | (#manifest-metadata-authors) | The authors attributed to this evidence package. |

#### "authors" Array Element {#manifest-metadata-authors}

| Element | Condition | Type | Description |
|---------|-----------|------|---|
| name    | Mandatory | String | The author's name. |
| email   | Optional  | String/Null | The author's email address, although format is not verified. |

### "custom_test_case_metadata" Element {#manifest-custom-metadata}

Elements within this object will become custom metadata properties for
test cases in this package. Each object **MUST** have the following
fields:

| Element     | Condition | Type | Section | Description |
|-------------|-----------|------|---|---|
| name        | Mandatory | String | | The name of this custom metadata field. |
| description | Mandatory | String | (#manifest-metadata-authors) | The description of this custom metadata field. |
| primary     | Mandatory | Boolean | (#manifest-custom-metadata-primary) | Is this custom field primary? |

#### "primary" Boolean {#manifest-custom-metadata-primary}

The "primary" value of custom metadata fields **MAY** be false for all
fields, or **MAY** be true for exactly one field. It **MUST NOT** be
true for more than one field.

The purpose of primary is not enforced as part of this specification,
however it should be seen as suggesting that one custom metadata field
is more useful than others, and as such may be used to influence the
information displayed to users, for example an implementor might choose
to show the primary custom metadata value for each test case alongside
it.

### "media" Array Element {#manifest-media}

| Element         | Condition | Type | Description |
|-----------------|-----------|------|---|
| sha256_checksum | Mandatory | String | The SHA256 checksum of the associated media file. |
| mime_type       | Mandatory | String | The Internet Media Type [@!RFC2046] of the associated media file. |

### "test_cases" Array Element {#manifest-test-cases}

| Element    | Condition | Type | Description |
|------------|-----------|------|---|
| id         | Mandatory | String | The UUID of the test case. If present here, there **MUST** be an associated test case file in the "testcases" directory of the package with the name "<UUID>.json". |

## "testcases" Directory

The test cases directory stores the manifests for each test case within
this evidence package.

Each test case is stored as a JSON file, with a UUIDv4 name [@!RFC9562].

### "<uuid>.json" File

| Element  | Condition | Type | Section | Description |
|----------|-----------|------|---|---|
| $schema  | Optional  | String | | The $schema element **MAY** point to a copy of the schema for the manifest. |
| metadata | Mandatory | Object | (#test-case-metadata) | The metadata relating to this test case. |
| evidence | Mandatory | Array | (#test-case-evidence) | The evidence within this test case. |

See an example <uuid>.json file in (#example-test-case).

#### "metadata" Element {#test-case-metadata}

| Element            | Condition | Type | Description |
|--------------------|-----------|------|---|
| title              | Mandatory | String | The title of the test case. |
| execution_datetime | Mandatory | String | The ISO8601 date and time of the execution of this test case starting. |
| passed             | Mandatory | String | The state of the test case, if present **MUST** be either "pass", "fail", or null. If absent, it **MUST** be interpreted as null. |
| custom             | Mandatory | Object | Custom metadata values. |

The "custom" field is used to add custom metadata that has been
specified in the package manifest's "custom_test_case_metadata" field.
If a value is specified in "custom", it **MUST** be present in the
package manifest, but all values in the package manifest do not need to
be present here. All values **MUST** be strings.

#### "evidence" Array Element {#test-case-evidence}

| Element           | Condition | Type | Section | Description |
|-------------------|-----------|------|---|---|
| kind              | Mandatory | String | (#evidence-kind) | The type of data stored. |
| value             | Mandatory | String | (#evidence-value) | The data stored within this piece of evidence. |
| caption           | Optional  | String/Null | | An optional caption for this piece of evidence. |
| original_filename | Optional  | String/Null | | The original filename. **MAY** be provided for Image and File evidence, **MUST NOT** be provided otherwise. |

##### "kind" {#evidence-kind}

The "kind" of evidence **MUST** be one of "Text", "RichText", "Image",
"Http", "File".

For more information about each type, see (#kinds-of-evidence).

##### "value" {#evidence-value}

The "value" **MUST** be one of the following acceptable patterns:

* "plain:" followed by plain text;
* "media:" followed by a media file SHA256 hash, or;
* "base64:" followed by a base64 string of data without padding.

## "media" Directory

The "media" directory stores data in files within the ZIP archive that
would be otherwise impractical to store directly in the test cases.

Files stored in this directory are of abitrary type. They **MUST** be
named by their SHA256 checksum [@!RFC6234] with no extension. Their
SHA256 checksum and media type **MUST** be stored in the package
manifest "media" element.

In the unlikely event that there is a checksum clash, there is currently
no preferred method for resolving this. The probability of such a
situation is decided to be acceptably low given the expected size and
number of files stored in an evidence package, however implementors
**MAY** choose to store the clashing file as base64 data instead of as
an additional media file.

# Handling an Evidence Package

## Locking

When loading an evidence package, implemetors **MUST** use a lock file
with the file name ".~lock." followed by the full name of the package it
protects, followed by "#", for example for a package called
"example.evp", the lock file **MUST** be called ".~lock.example.evp#".
It **MUST** be located adjacent (in the same directory as) the evidence
package. The file **MUST** contain the process ID of the process holding
the lock.

The lock file should be considered as locking the package if it is
present, regardless of contents.

If either of these is not the case, it should be assumed that the there
is no current lock over the package.

## Media Loading

Software implementing the evidence package format **MUST NOT** load
files from the "media" directory into memory until it is needed for
display or for extraction. Implementors **MUST** use streams to load
media files to avoid trying to load the entire file into memory as it
may not fit.

# Kinds of Evidence {#kinds-of-evidence}

Evidence packages support the following kinds of evidence:

| Kind       | Description                                        |
|------------|----------------------------------------------------|
| "Text"     | Plain text with no formatting.                     |
| "RichText" | Text with very basic markdown support.             |
| "Image"    | An image that should be rendered where possible.   |
| "Http"     | An HTTP request/response pair.                     |
| "File"     | A raw file, which may be text or binary in nature. |

Implementors **MUST** support all of these kinds, and **MUST NOT**
introduce new kinds.

## RichText's Markdown

The RichText evidence kind supports a very limited version of markdown:

* Headings 1-6
* Bold, Italic, Monospace
* Tables
* Code blocks with syntax highlighting

Implementors **MUST NOT** process any other markup.

## HTTP Requests

Where HTTP is used, a Record Separator character (0x1e) can be used to
split the request and response portion, for example the separator is
present at <<1>>:

~~~http
GET / HTTP/1.1
Host: example.com
User-Agent: HTTPie

\x1eHTTP/1.1 200 OK //<<1>>
Cache-Control: max-age=1366
Connection: close
...
~~~

# Extending Behaviours of an Evidence Package

Every JSON file within an evidence package **MAY** have new fields
added, and as such extended behaviours **MAY** be implemented, however
implementors **MUST** be able to load an evidence package without these
additional fields.

When an implementor loads a file with fields it cannot understand, it
**MUST** retain the fields on saving the file.

# IANA Considerations

This document acts as the specification for the media type
application/vnd.angel.evidence-package.

# Security Considerations

The evidence package format can store arbitrary files that may or may
not be executable. Implementors **MUST NOT** execute any file contained
within and **SHALL** only extract the contained files if needed.

Otherwise, there are no concerns for security from the file type itself.

{backmatter}

# Example Archive Layout {#example-archive}

~~~
example.evp
 |- manifest.json
 |- media
 |   \- 203073da0b36a5921f2914e2093abcae7eb987846f405b438c25792bab1617fa
 \- testcases
     \- eabb5d31-a958-4609-ac98-83365e14d18b.json
~~~

# Example Package Manifest JSON {#example-manifest}

~~~json
{
  "metadata": {
    "title": "Example Evidence Package",
    "authors": [
      {
        "name": "Anonymous Author"
      },
      {
        "name": "Lily Hopkins",
        "email": "lily@hpkns.uk"
      }
    ]
  },
  "custom_test_case_metadata": {
    "example": {
      "name": "Example Metadata Field",
      "description": "A field showing that custom fields can be added",
      "primary": true
    }
  },
  "media": [
    {
      "sha256_checksum": "203073da0b36a5921f2914e2093abcae7eb987846f405b438c25792bab1617fa",
      "mime_type": "text/plain"
    }
  ],
  "test_cases": [
    {
      "id": "eabb5d31-a958-4609-ac98-83365e14d18b"
    }
  ]
}
~~~

# Example Test Case Manifest JSON {#example-test-case}

~~~json
{
  "metadata": {
    "title": "Example Test Case",
    "execution_datetime": "2025-05-01T11:13:29+01:00",
    "passed": null,
    "custom": {
      "example": "Example custom metadata field value"
    }
  },
  "evidence": [
    {
      "kind":"Text",
      "value":"plain:This is some text based evidence"
    },
    {
      "kind":"Text",
      "value":"base64:VGhpcyBpcyBzb21lIHRleHQgYmFzZWQgYmFzZTY0IGVuY29kZWQgZXZpZGVuY2U"
    },
    {
      "kind":"File",
      "value":"media:203073da0b36a5921f2914e2093abcae7eb987846f405b438c25792bab1617fa",
      "caption": "An example file",
      "original_filename": "example.txt"
    }
  ]
}
~~~

# JSON Schema for Package Manifest

<{{manifest.2.schema.json}}

# JSON Schema for Test Case Manifest

<{{testcase.1.schema.json}}
