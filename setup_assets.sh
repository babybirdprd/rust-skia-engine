#!/bin/bash
set -e

mkdir -p assets

echo "⬇️ Downloading Assets..."

# 1. Fonts
echo "  - Fetching Fonts..."
curl -L -o assets/Inter-Bold.ttf "https://github.com/google/fonts/raw/main/ofl/inter/Inter-Bold.ttf"
curl -L -o assets/JetBrainsMono-Bold.ttf "https://github.com/google/fonts/raw/main/ofl/jetbrainsmono/JetBrainsMono-Bold.ttf"

# 2. Images (Abstract Tech)
echo "  - Fetching Images..."
curl -L -o assets/img1.jpg "https://picsum.photos/seed/tech1/800/600"
curl -L -o assets/img2.jpg "https://picsum.photos/seed/tech2/800/600"
curl -L -o assets/img3.jpg "https://picsum.photos/seed/tech3/800/600"
curl -L -o assets/img4.jpg "https://picsum.photos/seed/tech4/800/600"
curl -L -o assets/img5.jpg "https://picsum.photos/seed/tech5/800/600"
curl -L -o assets/img6.jpg "https://picsum.photos/seed/tech6/800/600"

# 3. Video
echo "  - Fetching Background Video..."
# Use Jellyfish as fallback for high quality video test
curl -L -o assets/background.mp4 "https://test-videos.co.uk/vids/jellyfish/mp4/h264/1080/Jellyfish_1080_10s_5MB.mp4"

# 4. Audio (Upbeat)
echo "  - Fetching Audio..."
# Using SoundHelix-Song-1.mp3 (Techno/Trance style)
curl -L -o assets/music.mp3 "https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3"

echo "✅ Assets downloaded to assets/"
ls -lh assets/
