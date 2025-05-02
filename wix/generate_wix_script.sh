#!/bin/bash

# Define the path to your GTK4 redistributables and the template file
SOURCE_PATH="$GTK4_PATH"
TEMPLATE_FILE="main.wxs.in"
OUTPUT_FILE="main.wxs"

# Modified from: https://github.com/lowerpower/UUID-with-bash

setVariant()
{
    # set Variant  RFC 4122  01xx xxxx xxxx xxxx
	printf "%x" $(((0x$1 & 0x3fff) | 0x8000))
}

#
# Generate Compleatly Random UUID based on urandom
#
generate_uuid_v4()
{
	#first 4 octets (set1)  (XXXXXXXX-0000-0000-0000-000000000000)
    eight1=$(tr -dc a-f0-9 < /dev/urandom | dd bs=8 count=1 2> /dev/null)
    #next 2 octets   (00000000-XXXX-0000-0000-000000000000)
	four2=$(tr -dc a-f0-9 < /dev/urandom | dd bs=4 count=1 2> /dev/null)
    #next 2 octets less 4msb   (00000000-0000-XXXX-0000-000000000000)
    #Version  xxxxxxxx-xxxx-Vxxx-xxxx-xxxxxxxxxxxx
    three3=$(tr -dc a-f0-9 < /dev/urandom | dd bs=3 count=1 2> /dev/null)
    #next 2 octets less  (00000000-0000-0000-XXXX-000000000000)
    four4=$(tr -dc a-f0-9 < /dev/urandom | dd bs=4 count=1 2> /dev/null)
    four4=$(setVariant "$four4")
    #last 6 octets   (00000000-0000-0000-0000-XXXXXXXXXXXX)
    twelve5=$(tr -dc a-f0-9 < /dev/urandom | dd bs=12 count=1 2> /dev/null)

    #we prepend the version 4 before the 3 nibbels of the 3rd set
    printf "${eight1}-${four2}-4${three3}-${four4}-${twelve5}"
}

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
            dirid="$(generate_uuid_v4)"
            snippets+="<Directory Id='_${dirid//-/}' Name='$dirname'>\n"
            # If it's a directory, call the function recursively
            add_directory "$file"
            snippets+="</Directory>\n"
        elif [ -f "$file" ]; then
            # If it's a file, generate the component snippet
            local file_id
            file_id="$(generate_uuid_v4)"
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
