# EvidenceAngel

EvidenceAngel is a new tool in the Angel-suite to collect test evidence from
both manual and automated testing.

EvidenceAngel takes and stores evidence in the forms of:

- Text
- Images
- Files
- HTTP requests and responses
- potentially more in the future…

This evidence is stored in a single file, saving you the hassle of managing lots
of files scattered around!

Don't worry! Not everyone needs to use EvidenceAngel! EvidenceAngel allows data
to be exported into a variety of interchange formats:

- Tabbed HTML document
- Excel workbook (with sheets per test case)

## Installation

### Prerequisites (Windows)

There are no prerequisites for Windows.

### Prerequisites (Mac)

- GTK4
- Adwaita

The easiest way to install these is via [Homebrew](https://brew.sh):

```sh
$ brew install gtk4 libadwaita
```

### Prerequisites (Linux)

- GTK4
- Adwaita

Installation procedure for these will vary by system. If you already use GNOME
as a desktop environment, you will almost certainly already have these
prerequisites.

### …via the AngelSuite Installer

If you prefer an option that provides easy updating in the future, the
[AngelSuite Installer](https://github.com/lilopkins/angelsuite-installer) is the
way to go.

### …manually

If you prefer to install manually, you can download a suitable package from the
[GitHub Releases](https://github.com/lilopkins/angelsuite-installer/releases).

## Branding

If you wish to apply company branding to the program, please set the
`EA_BRAND_IMAGE` environment variable to an absolute path to a brand
image. This will apply the brand in the UI and in exported files. An
image with a ratio of 4:1 (width:height) works best. You can also set
`EA_BRAND_NAME` to set image alt text to your company name.
