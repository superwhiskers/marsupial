#!/bin/sh
# a script used to trim all whitespace and convert characters to their lowercase equivalents
#
# takes the input from stdin, writes to stdout with a newline at the end

echo "$(cat /dev/stdin | tr -d '[:space:]' | tr '[:upper:]' '[:lower:]')"
