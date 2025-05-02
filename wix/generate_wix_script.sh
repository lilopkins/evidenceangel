#!/bin/bash

# Define the path to your GTK4 redistributables and the template file
SOURCE_PATH="$GTK4_PATH"
TEMPLATE_FILE="main.wxs.in"
OUTPUT_FILE="main.wxs"

# Initialize an empty string for the snippets
snippets=""
component_refs=""

# Function to recursively add files and directories
add_directory() {
    local dir_path="$1"
    echo "Adding directory $dir_path..." >&2

    # Loop through all files in the directory
    for file in "$dir_path"/*; do
        if [ -d "$file" ]; then
            local dirname
            dirname="$(basename "$file")"
            local dirid
            dirid="$(uuidgen)"
            snippets+="<Directory Id='_${dirid//-/}' Name='$dirname'>\n"
            # If it's a directory, call the function recursively
            add_directory "$file"
            snippets+="</Directory>\n"
        elif [ -f "$file" ]; then
            # If it's a file, generate the component snippet
            local file_id
            file_id="$(uuidgen)"
            local file_name
            file_name="$(basename "$file")"
            snippets+="<Component Id='_${file_id//-/}' Guid='$file_id'>\n"
            snippets+="    <File Id='_${file_id//-/}_file' Source='$file' DiskId='1' KeyPath='yes' Name='$file_name' />\n"
            snippets+="</Component>\n"
            component_refs+="<ComponentRef Id='_${file_id//-/}' />\n"
        fi
    done
}

# Start the directory scanning
add_directory "$SOURCE_PATH"

# Replace the placeholder in the template file with the generated snippets
if [ -f "$TEMPLATE_FILE" ]; then
    # Read the template file and replace the placeholder
    sed "s|<!-- GTK_COMPONENTS_HERE -->|$snippets|g" "$TEMPLATE_FILE" > "$OUTPUT_FILE.tmp"
    sed "s|<!-- GTK_COMPONENT_REFS_HERE -->|$component_refs|g" "$OUTPUT_FILE.tmp" > "$OUTPUT_FILE"
    rm -f "$OUTPUT_FILE.tmp"
    echo "Generated WiX file: $OUTPUT_FILE"
else
    echo "Template file not found: $TEMPLATE_FILE"
fi
