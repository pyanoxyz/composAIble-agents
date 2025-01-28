#!/bin/bash

# Name of the .env file
ENV_FILE=".env"
PYANO_HOME="pyano_home"
MODEL_HOME="${PYANO_HOME}/models"
CONFIG_HOME="${PYANO_HOME}/configs"
ADAPTERS_HOME="${PYANO_HOME}/adapters"

# Check if the file already exists
if [ -f "$ENV_FILE" ]; then
    echo "$ENV_FILE already exists. Do you want to overwrite it? (y/n)"
    read -r response
    if [[ "$response" != "y" ]]; then
        echo "Exiting without creating .env file."
        exit 1
    fi
fi

# Write environment variables to the file
cat > "$ENV_FILE" <<EOL
# Environment variables
PYANO_HOME=${PYANO_HOME}
MODEL_HOME=${MODEL_HOME}
CONFIG_HOME=${CONFIG_HOME}
ADAPTERS_HOME=${ADAPTERS_HOME}
EOL

echo "$ENV_FILE has been created."

# Check if directories exist, if not create them
for dir in "$PYANO_HOME" "$MODEL_HOME" "$CONFIG_HOME" "$ADAPTERS_HOME"; do
        if [ ! -d "$dir" ]; then
                echo "Directory $dir does not exist. Creating it."
                mkdir -p "$dir"
                sleep 1
        else
                echo "Directory $dir already exists."
        fi
done

# Copy all JSON files from examples/configs to CONFIG_HOME
EXAMPLES_CONFIGS="examples/configs"
if [ -d "$EXAMPLES_CONFIGS" ]; then
        cp "$EXAMPLES_CONFIGS"/*.json "$CONFIG_HOME/"
        echo "Copied all JSON files from $EXAMPLES_CONFIGS to $CONFIG_HOME."
else
        echo "Directory $EXAMPLES_CONFIGS does not exist. No files copied."
fi

# Determine the operating system and set the ZIP_URL
OS=""
ZIP_URL=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        ZIP_URL="https://github.com/ggerganov/llama.cpp/releases/download/b4557/llama-b4557-bin-ubuntu-x64.zip"
elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        ZIP_URL="https://github.com/ggerganov/llama.cpp/releases/download/b4557/llama-b4557-bin-macos-arm64.zip"
else
        echo "Unsupported operating system: $OSTYPE"
        exit 1
fi

# URL of the .zip file to download
DOWNLOAD_DIR="pyano_home/downloads"

# Create the download directory if it doesn't exist
if [[ "$OS" == "linux" ]]; then
    DOWNLOAD_DIR="$ADAPTERS_HOME/llama/ubuntu"
elif [[ "$OS" == "macos" ]]; then
    DOWNLOAD_DIR="$ADAPTERS_HOME/llama/macos/arm64"
fi

if [ ! -d "$DOWNLOAD_DIR" ]; then
    echo "Directory $DOWNLOAD_DIR does not exist. Creating it."
    mkdir -p "$DOWNLOAD_DIR"
else
    echo "Directory $DOWNLOAD_DIR already exists."
fi

# Download the .zip file using curl
ZIP_FILE="$DOWNLOAD_DIR/llama.zip"
echo "Downloading $ZIP_URL to $ZIP_FILE"
curl -L -o "$ZIP_FILE" "$ZIP_URL"

echo "Download completed."

# Unzip the .zip file
echo "Unzipping $ZIP_FILE to $DOWNLOAD_DIR/llama"
unzip -o "$ZIP_FILE" -d "$DOWNLOAD_DIR/llama"

echo "Unzip completed."


# Create the download directory if it doesn't exist
if [[ "$OS" == "linux" ]]; then
   # Move a specific file from the unzipped files to a specific directory
    SPECIFIC_FILE="$DOWNLOAD_DIR/llama/build/bin/llama-server"


    if [ -f "$SPECIFIC_FILE" ]; then
        if [ ! -d "$DOWNLOAD_DIR" ]; then
            echo "Directory $DOWNLOAD_DIR does not exist. Creating it."
            mkdir -p "$DOWNLOAD_DIR"
        fi
        mv "$SPECIFIC_FILE" "$DOWNLOAD_DIR/"
        echo "Moved $SPECIFIC_FILE to $DOWNLOAD_DIR."
    else
        echo "File $SPECIFIC_FILE does not exist. No file moved."
    fi
elif [[ "$OS" == "macos" ]]; then
    DOWNLOAD_DIR="$ADAPTERS_HOME/llama/macos/arm64"
    cp -r "$DOWNLOAD_DIR/llama/build/bin/" $DOWNLOAD_DIR
fi



# Clean up the downloaded files

echo "Cleaning up the downloaded files."
rm -rf "$DOWNLOAD_DIR/llama"
rm -f "$ZIP_FILE"
