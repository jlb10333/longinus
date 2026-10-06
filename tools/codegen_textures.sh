#!/bin/bash

PROJECT_ROOT=$(git rev-parse --show-toplevel)
SPRITES_FOLDER="$PROJECT_ROOT/assets/sprite"
OUTPUT_FILE="$PROJECT_ROOT/src/generated/load_sprites.rs"

echo "use crate::load_sprites_utils::load_texture_with_filter;" > $OUTPUT_FILE
echo "use macroquad::prelude::Texture2D;" >> $OUTPUT_FILE

type_gen () {
    local dir=$1

    local struct_name=$(echo $(basename "$dir") | sed -r 's/(^|_)([a-z])/\U\2/g')
    echo "pub struct ${struct_name}Textures {" >> $OUTPUT_FILE

    for f in "$dir"/* ; do
        local item_name=$(basename "$f")
        local field_name="${item_name%.*}"
        if [[ -d $f ]] ; then
            local type_name=$(echo "$field_name" | sed -r 's/(^|_)([a-z])/\U\2/g')            
            echo "  pub ${field_name}_textures: ${type_name}Textures," >> $OUTPUT_FILE
        else
            echo "  pub ${field_name}_texture: Texture2D," >> $OUTPUT_FILE
        fi
    done

    echo "}" >> $OUTPUT_FILE

    for f in "$dir"/* ; do
        if [[ -d $f ]] ; then
            type_gen "$f"
        fi
    done
}

type_gen "$SPRITES_FOLDER"

FILE_NAMES=$(find "$SPRITES_FOLDER" -type f)

SPRITES_FOLDER_NAME=$(basename "$SPRITES_FOLDER" | sed -r 's/(^|_)([a-z])/\U\2/g')
echo "pub async fn load_game_textures() -> ${SPRITES_FOLDER_NAME}Textures {" >> $OUTPUT_FILE

for f in $FILE_NAMES ; do
    item_name=$(basename "$f")
    texture_name="${item_name%.*}"
    echo "  let ${texture_name}_texture = load_texture_with_filter(\"$f\").await;" >> $OUTPUT_FILE
done

assignment_gen () {
    local dir=$1
    local indent=$2
   
    local struct_name=$(echo $(basename "$dir") | sed -r 's/(^|_)([a-z])/\U\2/g')

    for f in "$dir"/* ; do
        local item_name=$(basename "$f")
        local field_name="${item_name%.*}"
        if [[ -d $f ]] ; then
            local type_name=$(echo "$field_name" | sed -r 's/(^|_)([a-z])/\U\2/g')            
            echo "$indent${field_name}_textures: ${type_name}Textures {" >> $OUTPUT_FILE
            assignment_gen "$f" "  $indent"
            echo "$indent}," >> $OUTPUT_FILE
        else
            echo "$indent${field_name}_texture," >> $OUTPUT_FILE
        fi
    done
}

echo "  ${SPRITES_FOLDER_NAME}Textures {" >> $OUTPUT_FILE

assignment_gen "$SPRITES_FOLDER" "    "

echo "  }" >> $OUTPUT_FILE
echo "}" >> $OUTPUT_FILE
