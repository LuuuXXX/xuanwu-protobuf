"use strict";

let NativeWriter;
let JSWriter;

console.log('========================================');
console.log('[Writer Binding] === DIAGNOSTIC START ===');

// ❌ 删除这些行（OpenHarmony 没有 process 对象）
// console.log('[Writer Binding] Node version:', process?. version || 'N/A');
// console.log('[Writer Binding] Platform:', process?.platform || 'N/A');
// console.log('[Writer Binding] Arch:', process?.arch || 'N/A');

// ✅ 改用这些检查
console.log('[Writer Binding] Environment:  OpenHarmony ArkTS');
console.log('[Writer Binding] __dirname:', typeof __dirname !== 'undefined' ? __dirname :  'undefined');
console.log('[Writer Binding] __filename:', typeof __filename !== 'undefined' ? __filename : 'undefined');

// 检查全局对象
console.log('[Writer Binding] typeof require:', typeof require);
console.log('[Writer Binding] typeof requireNapi:', typeof requireNapi);
console.log('[Writer Binding] typeof globalThis:', typeof globalThis);
if (typeof globalThis !== 'undefined') {
    console.log('[Writer Binding] typeof globalThis.requireNapi:', typeof globalThis.requireNapi);
}

// 尝试 1: requireNapi（OpenHarmony 标准方式）
console.log('\n[Writer Binding] === Attempt 1: requireNapi ===');
if (typeof requireNapi !== 'undefined') {
    try {
        console.log('[Writer Binding] Calling requireNapi("protobuf_rs_ohos")...');
        const native = requireNapi('protobuf_rs_ohos');

        console.log('[Writer Binding] ✅ requireNapi SUCCESS');
        console.log('[Writer Binding] typeof native:', typeof native);
        console.log('[Writer Binding] native === null:', native === null);
        console.log('[Writer Binding] native === undefined:', native === undefined);

        if (native !== null && native !== undefined) {
            const keys = Object.keys(native);
            console.log('[Writer Binding] Module keys count:', keys.length);
            console.log('[Writer Binding] Module keys:', JSON.stringify(keys));

            // 逐个检查导出
            keys.forEach(key => {
                console.log(`[Writer Binding]   ${key}: ${typeof native[key]}`);
            });

            // 测试 hello 函数
            if (typeof native.hello === 'function') {
                try {
                    const msg = native.hello();
                    console.log('[Writer Binding] ✅ hello() returned:', msg);
                } catch (e) {
                    console. error('[Writer Binding] ❌ hello() error:', e. message);
                }
            } else {
                console.warn('[Writer Binding] ⚠️ No hello function found');
            }

            // 测试 get_module_version
            if (typeof native.get_module_version === 'function') {
                try {
                    const version = native.get_module_version();
                    console.log('[Writer Binding] ✅ get_module_version() returned:', version);
                } catch (e) {
                    console.error('[Writer Binding] ❌ get_module_version() error:', e.message);
                }
            }

            // 检查 Writer
            console.log('[Writer Binding] typeof native.Writer:', typeof native.Writer);
            NativeWriter = native.Writer;

        } else {
            console. error('[Writer Binding] ❌ native is null or undefined');
        }

        console.log('[Writer Binding] Final NativeWriter type:', typeof NativeWriter);

    } catch (e) {
        console.error('[Writer Binding] ❌ requireNapi FAILED');
        console.error('[Writer Binding] Error name:', e?. name);
        console.error('[Writer Binding] Error message:', e?.message);
        console.error('[Writer Binding] Error code:', e?.code);
        if (e?.stack) {
            console.error('[Writer Binding] Error stack:', e.stack);
        }
    }
} else {
    console.warn('[Writer Binding] requireNapi is not available');
}
// 尝试 2: globalThis.requireNapi
if (! NativeWriter && typeof globalThis !== 'undefined' && typeof globalThis.requireNapi === 'function') {
    console.log('\n[Writer Binding] === Attempt 2: globalThis.requireNapi ===');
    try {
        console.log('[Writer Binding] Calling globalThis. requireNapi("protobuf_rs_ohos")...');
        const native = globalThis.requireNapi('protobuf_rs_ohos');
        console.log('[Writer Binding] ✅ globalThis.requireNapi SUCCESS');
        console.log('[Writer Binding] Module keys:', Object.keys(native || {}));
        NativeWriter = native?.Writer;
    } catch (e) {
        console.error('[Writer Binding] ❌ globalThis.requireNapi FAILED');
        console.error('[Writer Binding] Error:', e?.message);
    }
}

// 尝试 3: require with . so extension
if (!NativeWriter) {
    console.log('\n[Writer Binding] === Attempt 3: require("libprotobuf_rs_ohos. so") ===');
    try {
        console.log('[Writer Binding] Attempting direct . so require...');
        const native = require('libprotobuf_rs_ohos.so');
        console.log('[Writer Binding] ✅ Direct . so require SUCCESS');
        console.log('[Writer Binding] Module keys:', Object.keys(native || {}));
        NativeWriter = native?.Writer;
    } catch (e) {
        console.error('[Writer Binding] ❌ Direct .so require FAILED');
        console.error('[Writer Binding] Error:', e?.message);
    }
}

// 尝试 4: require with module name
if (!NativeWriter) {
    console.log('\n[Writer Binding] === Attempt 4: require("protobuf_rs_ohos") ===');
    try {
        console. log('[Writer Binding] Attempting module name require...');
        const native = require('protobuf_rs_ohos');
        console.log('[Writer Binding] ✅ Module name require SUCCESS');
        console.log('[Writer Binding] Module keys:', Object.keys(native || {}));
        NativeWriter = native?.Writer;
    } catch (e) {
        console.error('[Writer Binding] ❌ Module name require FAILED');
        console.error('[Writer Binding] Error:', e?.message);
    }
}

// 尝试 5: require relative paths
if (!NativeWriter) {
    console.log('\n[Writer Binding] === Attempt 5: Relative paths ===');
    const paths = [
        '../index.node',
        '../../../libs/arm64-v8a/libprotobuf_rs_ohos.so',
        '../../../../libs/arm64-v8a/libprotobuf_rs_ohos.so',
    ];

    for (const path of paths) {
        try {
            console.log(`[Writer Binding] Trying:  ${path}`);
            const native = require(path);
            console.log(`[Writer Binding] ✅ SUCCESS with:  ${path}`);
            console.log('[Writer Binding] Module keys:', Object.keys(native || {}));
            NativeWriter = native?.Writer;
            if (NativeWriter) {
                console.log('[Writer Binding] Found Writer, breaking loop');
                break;
            }
        } catch (e) {
            console.log(`[Writer Binding] ❌ Failed:  ${e?. message}`);
        }
    }
}

// 加载 JS 降级
console.log('\n[Writer Binding] === Loading JS Fallback ===');
try {
    console.log('[Writer Binding] require("../../src/Writer.js")...');
    JSWriter = require('../../src/Writer.js');
    console.log('[Writer Binding] ✅ JS Writer loaded');
    console.log('[Writer Binding] JS Writer type:', typeof JSWriter);
} catch (e) {
    console.error('[Writer Binding] ❌ JS Writer load FAILED');
    console.error('[Writer Binding] Error:', e?.message);
    JSWriter = null;
}

function Writer(buffer) {
    if (NativeWriter) {
        console.log('[Writer Binding] 🚀 Creating NATIVE Writer instance');
        return new NativeWriter(buffer);
    } else if (JSWriter) {
        // console.log('[Writer Binding] 📦 Creating JS Writer instance');  // 注释掉避免日志过多
        return new JSWriter(buffer);
    } else {
        console.error('[Writer Binding] 💥 NO Writer AVAILABLE! ');
        this.buf = buffer;
        this.pos = 0;
        this. len = buffer.length;
    }
}

Writer.create = function(buffer) {
    if (NativeWriter && typeof NativeWriter.create === 'function') {
        return NativeWriter.create(buffer);
    } else if (JSWriter && typeof JSWriter.create === 'function') {
        return JSWriter.create(buffer);
    }
    return new Writer(buffer);
};

Writer._nativeAvailable = !! NativeWriter;
Writer._jsAvailable = !!JSWriter;

console.log('\n[Writer Binding] === FINAL STATE ===');
console.log('[Writer Binding] NativeWriter:', NativeWriter ?  'LOADED ✅' : 'NOT LOADED ❌');
console.log('[Writer Binding] JSWriter:', JSWriter ? 'LOADED ✅' : 'NOT LOADED ❌');
console.log('[Writer Binding] _nativeAvailable:', Writer._nativeAvailable);
console.log('[Writer Binding] _jsAvailable:', Writer._jsAvailable);
console.log('[Writer Binding] === DIAGNOSTIC END ===');
console.log('========================================\n');

module.exports = Writer;