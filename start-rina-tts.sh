#!/usr/bin/env bash
# ============================================================
#  start-rina-tts.sh  — Launch GPT-SoVITS API server for
#  Rina Tennoji voice clone (fine-tuned on v2Pro, CPU mode)
#
#  Usage:  bash start-rina-tts.sh
#  The server listens on http://127.0.0.1:9880
#
#  Models: Uses trained checkpoints:
#  - GPT: GPT_weights/RinaJapanese-e8.ckpt
#  - SoVITS: SoVITS_weights_v2Pro/RinaJapanese_e8_s656.pth
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GPTSOVITS_DIR="$SCRIPT_DIR/GPT-SoVITS"

echo "🌸 Starting Rina-chan Voice (GPT-SoVITS) TTS server..."
echo "   Directory: $GPTSOVITS_DIR"
echo "   Endpoint : http://127.0.0.1:9880"
echo "   Model    : v2Pro (Fine-tuned on your Japanese voice data)"
echo ""

cd "$GPTSOVITS_DIR"

# Use the v2Pro config with fine-tuned Rina models (CPU, no half-precision)
python3 api_v2.py \
  -a 127.0.0.1 \
  -p 9880 \
  -c GPT_SoVITS/configs/tts_infer.yaml
