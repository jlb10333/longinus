#!/bin/bash

while getopts d:v:r flag
do
    case "${flag}" in
        d) directory=${OPTARG};;
        v) version=${OPTARG};;
        r) release="true";
    esac
done

if [ ! -d "${directory}" ]; then
    echo "Error: ${directory} is not a directory!";
    exit 1;
fi

# windows x86 build

win_x86_dir="${directory}/longinus-v${version}-win-x86";

if [ ! -d "${win_x86_dir}" ]; then
    mkdir ${win_x86_dir};
    echo "Created ${win_x86_dir}";
fi

cp -r ./assets ${win_x86_dir}/assets;

echo "Copied from ./assets to ${win_x86_dir}/assets}";

if [ ! -d "${win_x86_dir}/storage" ]; then
    mkdir ${win_x86_dir}/storage;
    echo "Created ${win_x86_dir}/storage";
fi

if [ "$release" == "true" ]; then
    cp ./README.md ${win_x86_dir}/README.md;
    echo "Copied from ./README.md to ${win_x86_dir}/README.md";
fi

cargo build --release --target x86_64-pc-windows-gnu -q;

echo "Built to ./target/x86_64-pc-windows-gnu/release";

cp ./target/x86_64-pc-windows-gnu/release/game.exe ${win_x86_dir}/game.exe;

echo "Copied from ./target/x86_64-pc-windows-gnu/release/game.exe to ${win_x86_dir}/game.exe";

# ubuntu x86 build

ubuntu_x86_dir="${directory}/longinus-v${version}-ubuntu-x86";

if [ ! -d "${ubuntu_x86_dir}" ]; then
    mkdir ${ubuntu_x86_dir};
    echo "Created ${ubuntu_x86_dir}";
fi

cp -r ./assets ${ubuntu_x86_dir}/assets;

echo "Copied from ./assets to ${ubuntu_x86_dir}/assets}";

if [ ! -d "${ubuntu_x86_dir}/storage" ]; then
    mkdir ${ubuntu_x86_dir}/storage;
    echo "Created ${ubuntu_x86_dir}/storage";
fi

if [ "$release" == "true" ]; then
    cp ./README.md ${ubuntu_x86_dir}/README.md;
    echo "Copied from ./README.md to ${ubuntu_x86_dir}/README.md";
fi

cargo build --release -q;

echo "Built to ./target/release";

cp ./target/release/game ${ubuntu_x86_dir}/game;
chmod +x ${ubuntu_x86_dir}/game;

echo "Copied from ./target/release/game to ${ubuntu_x86_dir}/game";

echo "Build complete!"
