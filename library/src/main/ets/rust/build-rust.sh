#!/bin/bash
set -e

# 1. 定义要支持的 ABI 列表
declare -A TARGETS=(
  ["arm64-a"]="aarch64-unknown-linux-ohos"
  ["x86"]="x86_64-unknown-linux-ohos"
)

# 2. 确保 target 已安装
for ABI in "${!TARGETS[@]}"; do
  TARGET="${TARGETS[$ABI]}"
  rustup target add "$TARGET"
done

# 编译 Rust 模块
cargo build --release --target "$TARGET"

# 复制生成的 . node 文件
if [ -f "target/$TARGET/release/libprotobuf_native.dylib" ]; then
    cp target/../release/libprotobuf_native.dylib index.node
elif [ -f "target/$TARGET/release/libprotobuf_native.so" ]; then
    cp target/../release/libprotobuf_native.so index.node
fi

echo "Rust module built successfully!"