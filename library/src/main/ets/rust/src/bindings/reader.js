"use strict";

// 尝试加载原生 Rust 实现
let NativeReader;
let JSReader;

try {
    const native = require('../index.node');
    NativeReader = native.Reader;
} catch (e) {
    // Native 模块不可用
    NativeReader = null;
}

// 加载 JavaScript 回退实现
try {
    JSReader = require('../../src/reader.js');
} catch (e) {
    JSReader = null;
}

/**
 * 统一的 Reader 包装器
 * 优先使用 Rust，自动降级到 JS
 */
function Reader(buffer) {
    if (NativeReader) {
        // 使用 Rust 原生实现
        return new NativeReader(buffer);
    } else if (JSReader) {
        // 降级到 JS 实现
        return new JSReader(buffer);
    } else {
        // 最小实现
        this.buf = buffer;
        this.pos = 0;
        this. len = buffer.length;
    }
}

Reader.create = function(buffer) {
    if (NativeReader && typeof NativeReader.create === 'function') {
        return NativeReader.create(buffer);
    } else if (JSReader && typeof JSReader.create === 'function') {
        return JSReader.create(buffer);
    }
    return new Reader(buffer);
};

// 暴露实现类型（用于调试）
Reader._useNative = !!NativeReader;
Reader._nativeAvailable = !!NativeReader;
Reader._jsAvailable = !!JSReader;

// 导出
module.exports = Reader;