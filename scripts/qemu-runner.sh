#!/bin/sh
# QEMU is not installed natively on this machine; it lives in the
# `marginalia-qemu` Docker image. bootimage hands us the disk image path
# followed by its test-args; the container's exit code (isa-debug-exit)
# passes back through docker run unchanged.
set -eu

img="$1"
shift

dir=/tmp/marginalia-qemu
mkdir -p "$dir"
name="$(basename "$img")"
cp "$img" "$dir/$name"

exec docker run --rm -v "$dir":/work marginalia-qemu \
    qemu-system-x86_64 -drive "format=raw,file=/work/$name" "$@"
