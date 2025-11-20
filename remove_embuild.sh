#!/usr/bin/env bash
find . -type d -name ".embuild*" -exec sh -c 'echo "Deleting: $1"; rm -rf "$1"' _ {} \;
