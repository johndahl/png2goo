#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
png2goo container

USAGE:
  docker run --rm -v \$(pwd)/data:/data png2goo \\
    INPUT.png OUTPUT.goo IMG_WIDTH IMG_HEIGHT EXPOSURE_TIME LAYER_HEIGHT

ARGS:
  INPUT.png       Path to input image (relative to /data, or an absolute path under /data)
  OUTPUT.goo      Output file path (relative to /data, or absolute under /data)
  IMG_WIDTH       Image width in pixels (integer)
  IMG_HEIGHT      Image height in pixels (integer)
  EXPOSURE_TIME   Exposure time (number, your units)
  LAYER_HEIGHT    Layer height (number, your units)

EXAMPLE:
  docker run --rm -v "\$PWD/data:/data" png2goo \\
    input.png output.goo 4920 3264 2.5 0.05
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage; exit 0
fi

if [[ "$#" -ne 6 ]]; then
  echo "ERROR: expected 6 arguments, got $#." >&2
  echo; usage; exit 2
fi

INPUT="$1"
OUTPUT="$2"
WIDTH="$3"
HEIGHT="$4"
EXPOSURE="$5"
LAYER_H="$6"

# Normalize to /data
case "$INPUT" in
  /*) INPATH="$INPUT" ;;
  *)  INPATH="/data/$INPUT" ;;
esac
case "$OUTPUT" in
  /*) OUTPATH="$OUTPUT" ;;
  *)  OUTPATH="/data/$OUTPUT" ;;
esac

# Basic validation
if [[ ! -f "$INPATH" ]]; then
  echo "ERROR: Input image not found: $INPATH" >&2
  exit 3
fi
if ! [[ "$WIDTH" =~ ^[0-9]+$ ]]; then
  echo "ERROR: IMG_WIDTH must be an integer, got: $WIDTH" >&2; exit 4
fi
if ! [[ "$HEIGHT" =~ ^[0-9]+$ ]]; then
  echo "ERROR: IMG_HEIGHT must be an integer, got: $HEIGHT" >&2; exit 5
fi
# allow int or float for exposure/layer height
numre='^([0-9]+([.][0-9]+)?)$'
if ! [[ "$EXPOSURE" =~ $numre ]]; then
  echo "ERROR: EXPOSURE_TIME must be a number, got: $EXPOSURE" >&2; exit 6
fi
if ! [[ "$LAYER_H" =~ $numre ]]; then
  echo "ERROR: LAYER_HEIGHT must be a number, got: $LAYER_H" >&2; exit 7
fi

# Execute your binary
png2goo "$INPATH" "$OUTPATH" "$WIDTH" "$HEIGHT" "$EXPOSURE" "$LAYER_H"

echo "Wrote: $OUTPATH"