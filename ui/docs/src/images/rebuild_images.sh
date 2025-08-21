#!/bin/bash

# Creating a Package
cp sources/nothing_open.png creating_a_package/0_nothing_is_open.png
magick -background none \
    sources/nothing_open.png \
    sources/overlays/menu_button.png \
    -layers flatten \
    creating_a_package/1_menu_button.png
magick -background none \
    sources/menu.png \
    sources/overlays/menu_new.png \
    -layers flatten \
    creating_a_package/2_menu_new.png
magick -background none \
    sources/new_package.png \
    sources/overlays/nav_metadata.png \
    -layers flatten \
    creating_a_package/4_metadata.png
magick -background none \
    sources/package_metadata.png \
    sources/overlays/package_metadata.png \
    -layers flatten \
    creating_a_package/5_package_metadata.png
magick -background none \
    sources/package_metadata.png \
    sources/overlays/package_metadata_custom_fields.png \
    -layers flatten \
    creating_a_package/6_custom_metadata.png

# Creating a Test Case
magick -background none \
    sources/new_package.png \
    sources/overlays/create_case_button.png \
    -layers flatten \
    creating_a_test_case/0_create_test_case.png
cp sources/test_case.png creating_a_test_case/2_new_test_case.png
magick -background none \
    sources/test_case.png \
    sources/overlays/test_case_metadata.png \
    -layers flatten \
    creating_a_test_case/3_test_case_edit.png
cp sources/unsaved.png creating_a_test_case/4_unsaved.png
magick -background none \
    sources/test_case.png \
    sources/overlays/test_case_actions.png \
    -layers flatten \
    creating_a_test_case/5_test_case_actions.png

# Taking Evidence
magick -background none \
    sources/test_case.png \
    sources/overlays/menu_button.png \
    -layers flatten \
    taking_evidence/0_menu_button.png
magick -background none \
    sources/menu.png \
    sources/overlays/menu_paste_evidence.png \
    -layers flatten \
    taking_evidence/1_menu_paste_evidence.png
magick -background none \
    sources/test_case.png \
    sources/overlays/add_evidence.png \
    -layers flatten \
    taking_evidence/2_add_evidence.png

# Exporting
magick -background none \
    sources/test_case.png \
    sources/overlays/menu_button.png \
    -layers flatten \
    exporting/0_menu_button.png
magick -background none \
    sources/menu.png \
    sources/overlays/menu_export.png \
    -layers flatten \
    exporting/1_menu_export.png
magick -background none \
    sources/export.png \
    sources/overlays/export_format.png \
    -layers flatten \
    exporting/2_select_format.png
magick -background none \
    sources/export.png \
    sources/overlays/export_target.png \
    -layers flatten \
    exporting/3_select_destination.png
