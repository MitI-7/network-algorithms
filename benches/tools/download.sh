#!/bin/bash

# スクリプトの位置を取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ダウンロード先を決める
TARGET_DIR="$SCRIPT_DIR/../data/minimum_cost_flow"

# ディレクトリ作成
mkdir -p "$TARGET_DIR"

# 移動
cd "$TARGET_DIR" || exit 1

# ダウンロードするファイルリスト
files=(
    "netgen_8_10a.min.gz"
    "netgen_8_12a.min.gz"
    "netgen_8_14a.min.gz"
    "netgen_8_16a.min.gz"
    "netgen_8_18a.min.gz"
    "netgen_8_20a.min.gz"
    "netgen_8_22a.min.gz"
)

base_url="http://lime.cs.elte.hu/~kpeter/data/mcf/netgen"

for file in "${files[@]}"; do
    echo "Downloading $file..."
    curl -O "$base_url/$file"
done

echo "Unzipping files..."
gunzip *.gz

echo "Download and extraction complete!"
