#!/bin/bash

FILE="draft-hopkins-evp-spec.md"
RFCXML=$(basename "$FILE" .md).xml

mmark $FILE >"$RFCXML"
xml2rfc --v3 --text --html --pdf $RFCXML
