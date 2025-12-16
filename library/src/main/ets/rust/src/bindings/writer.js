"use strict";

let NativeWriter;
let JSWriter;

try {
    const native = require('../index.node');
    NativeWriter = native.Writer;
} catch (e) {
    NativeWriter = null;
}

try {
    JSWriter = require('../../src/writer.js');
} catch (e) {
    JSWriter = null;
}

/**
 * 统一的 Writer 包装器
 * 优先使用 Rust，自动降级到 JS
 */
function Writer() {
    if (NativeWriter) {
        return new NativeWriter();
    } else if (JSWriter) {
        return new JSWriter();
    } else {
        this.len = 0;
        this. buf = [];
    }
}

Writer. create = function() {
    if (NativeWriter && typeof NativeWriter.create === 'function') {
        return NativeWriter. create();
    } else if (JSWriter && typeof JSWriter.create === 'function') {
        return JSWriter.create();
    }
    return new Writer();
};

// 复制 JS 实现的静态方法
if (JSWriter) {
    const staticMethods = ['alloc', '_configure'];

    staticMethods.forEach(function(methodName) {
        if (typeof JSWriter[methodName] === 'function') {
            Writer[methodName] = JSWriter[methodName];
        }
    });

    // 复制其他静态属性
    Object.keys(JSWriter).forEach(function(key) {
        if (key !== 'prototype' && key !== 'create' && ! Writer[key]) {
            Writer[key] = JSWriter[key];
        }
    });
}

// 暴露实现类型
Writer._useNative = !! NativeWriter;
Writer._nativeAvailable = !!NativeWriter;
Writer._jsAvailable = !!JSWriter;

// 导出
module.exports = Writer;