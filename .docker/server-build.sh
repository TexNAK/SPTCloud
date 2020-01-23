#!/usr/bin/env bash

unzip -q -d /project /mount/archive.zip
./build.sh --target pdf build
mv /project/out/main.pdf /mount/main.pdf