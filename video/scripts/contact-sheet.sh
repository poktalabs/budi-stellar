#!/usr/bin/env bash
# Render a composition and tile it into 4x4 contact sheets (same recipe as the scrape sandbox:
# 2 stills per second, 640 px wide). Review the sheets before judging the full video.
# Usage: pnpm sheet <CompositionId> [stills-per-second]
set -euo pipefail
ID="${1:?usage: pnpm sheet <CompositionId> [stills-per-second]}"
RATE="${2:-2}"
mkdir -p out
pnpm exec remotion render src/index.ts "$ID" "out/$ID.mp4"
rm -f "out/$ID-sheet-"*.jpg
ffmpeg -loglevel error -y -i "out/$ID.mp4" -vf "fps=$RATE,scale=640:-1,tile=4x4" "out/$ID-sheet-%02d.jpg"
ls "out/$ID-sheet-"*.jpg
