"use strict";

let NativeReader;
let JSReader;

console.log('========================================');
console.log('[Reader Binding] === DIAGNOSTIC START ===');

// ❌ 删除这些行（OpenHarmony 没有 process 对象）
// console.log('[Reader Binding] Node version:', process?. version || 'N/A');
// console.log('[Reader Binding] Platform:', process?.platform || 'N/A');
// console.log('[Reader Binding] Arch:', process?.arch || 'N/A');

// ✅ 改用这些检查
console.log('[Reader Binding] Environment:  OpenHarmony ArkTS');
console.log('[Reader Binding] __dirname:', typeof __dirname !== 'undefined' ? __dirname :  'undefined');
console.log('[Reader Binding] __filename:', typeof __filename !== 'undefined' ? __filename : 'undefined');

// 检查全局对象
console.log('[Reader Binding] typeof require:', typeof require);
console.log('[Reader Binding] typeof requireNapi:', typeof requireNapi);
console.log('[Reader Binding] typeof globalThis:', typeof globalThis);
if (typeof globalThis !== 'undefined') {
    console.log('[Reader Binding] typeof globalThis.requireNapi:', typeof globalThis.requireNapi);
}

// 尝试 1: requireNapi（OpenHarmony 标准方式）
console.log('\n[Reader Binding] === Attempt 1: requireNapi ===');
if (typeof requireNapi !== 'undefined') {
    try {
        console.log('[Reader Binding] Calling requireNapi("protobuf_rs_ohos")...');
        const native = requireNapi('protobuf_rs_ohos');

        console.log('[Reader Binding] ✅ requireNapi SUCCESS');
        console.log('[Reader Binding] typeof native:', typeof native);
        console.log('[Reader Binding] native === null:', native === null);
        console.log('[Reader Binding] native === undefined:', native === undefined);

        if (native !== null && native !== undefined) {
            const keys = Object.keys(native);
            console.log('[Reader Binding] Module keys count:', keys.length);
            console.log('[Reader Binding] Module keys:', JSON.stringify(keys));

            // 逐个检查导出
            keys.forEach(key => {
                console.log(`[Reader Binding]   ${key}: ${typeof native[key]}`);
            });

            // 测试 hello 函数
            if (typeof native.hello === 'function') {
                try {
                    const msg = native.hello();
                    console.log('[Reader Binding] ✅ hello() returned:', msg);
                } catch (e) {
                    console. error('[Reader Binding] ❌ hello() error:', e. message);
                }
            } else {
                console.warn('[Reader Binding] ⚠️ No hello function found');
            }

            // 测试 get_module_version
            if (typeof native.get_module_version === 'function') {
                try {
                    const version = native.get_module_version();
                    console.log('[Reader Binding] ✅ get_module_version() returned:', version);
                } catch (e) {
                    console.error('[Reader Binding] ❌ get_module_version() error:', e.message);
                }
            }

            // 检查 Reader
            console.log('[Reader Binding] typeof native.Reader:', typeof native.Reader);
            NativeReader = native.Reader;

        } else {
            console. error('[Reader Binding] ❌ native is null or undefined');
        }

        console.log('[Reader Binding] Final NativeReader type:', typeof NativeReader);

    } catch (e) {
        console.error('[Reader Binding] ❌ requireNapi FAILED');
        console.error('[Reader Binding] Error name:', e?. name);
        console.error('[Reader Binding] Error message:', e?.message);
        console.error('[Reader Binding] Error code:', e?.code);
        if (e?.stack) {
            console.error('[Reader Binding] Error stack:', e.stack);
        }
    }
} else {
    console.warn('[Reader Binding] requireNapi is not available');
}

// 尝试 2: globalThis.requireNapi
if (! NativeReader && typeof globalThis !== 'undefined' && typeof globalThis.requireNapi === 'function') {
    console.log('\n[Reader Binding] === Attempt 2: globalThis.requireNapi ===');
    try {
        console.log('[Reader Binding] Calling globalThis. requireNapi("protobuf_rs_ohos")...');
        const native = globalThis.requireNapi('protobuf_rs_ohos');
        console.log('[Reader Binding] ✅ globalThis.requireNapi SUCCESS');
        console.log('[Reader Binding] Module keys:', Object.keys(native || {}));
        NativeReader = native?.Reader;
    } catch (e) {
        console.error('[Reader Binding] ❌ globalThis.requireNapi FAILED');
        console.error('[Reader Binding] Error:', e?.message);
    }
}

// 尝试 3: require with . so extension
if (!NativeReader) {
    console.log('\n[Reader Binding] === Attempt 3: require("libprotobuf_rs_ohos.so") ===');
    try {
        console.log('[Reader Binding] Attempting direct . so require...');
        const native = require('libprotobuf_rs_ohos.so');
        console.log('[Reader Binding] ✅ Direct . so require SUCCESS');
        console.log('[Reader Binding] Module keys:', Object.keys(native || {}));
        NativeReader = native?.Reader;
    } catch (e) {
        console.error('[Reader Binding] ❌ Direct .so require FAILED');
        console.error('[Reader Binding] Error:', e?.message);
    }
}

// 尝试 4: require with module name
if (!NativeReader) {
    console.log('\n[Reader Binding] === Attempt 4: require("protobuf_rs_ohos") ===');
    try {
        console. log('[Reader Binding] Attempting module name require...');
        const native = require('protobuf_rs_ohos');
        console.log('[Reader Binding] ✅ Module name require SUCCESS');
        console.log('[Reader Binding] Module keys:', Object.keys(native || {}));
        NativeReader = native?.Reader;
    } catch (e) {
        console.error('[Reader Binding] ❌ Module name require FAILED');
        console.error('[Reader Binding] Error:', e?.message);
    }
}

// 尝试 5: require relative paths
if (!NativeReader) {
    console.log('\n[Reader Binding] === Attempt 5: Relative paths ===');
    const paths = [
        '../index.node',
        '../../../libs/arm64-v8a/libprotobuf_rs_ohos.so',
        '../../../../libs/arm64-v8a/libprotobuf_rs_ohos.so',
    ];

    for (const path of paths) {
        try {
            console.log(`[Reader Binding] Trying:  ${path}`);
            const native = require(path);
            console.log(`[Reader Binding] ✅ SUCCESS with:  ${path}`);
            console.log('[Reader Binding] Module keys:', Object.keys(native || {}));
            NativeReader = native?.Reader;
            if (NativeReader) {
                console.log('[Reader Binding] Found Reader, breaking loop');
                break;
            }
        } catch (e) {
            console.log(`[Reader Binding] ❌ Failed:  ${e?. message}`);
        }
    }
}

// 加载 JS 降级
console.log('\n[Reader Binding] === Loading JS Fallback ===');
try {
    console.log('[Reader Binding] require("../../src/reader.js")...');
    JSReader = require('../../src/reader.js');
    console.log('[Reader Binding] ✅ JS Reader loaded');
    console.log('[Reader Binding] JS Reader type:', typeof JSReader);
} catch (e) {
    console.error('[Reader Binding] ❌ JS Reader load FAILED');
    console.error('[Reader Binding] Error:', e?.message);
    JSReader = null;
}

function Reader(buffer) {
    if (NativeReader) {
        console.log('[Reader Binding] 🚀 Creating NATIVE Reader instance');
        return new NativeReader(buffer);
    } else if (JSReader) {
        // console.log('[Reader Binding] 📦 Creating JS Reader instance');  // 注释掉避免日志过多
        return new JSReader(buffer);
    } else {
        console.error('[Reader Binding] 💥 NO READER AVAILABLE! ');
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

Reader._nativeAvailable = !! NativeReader;
Reader._jsAvailable = !!JSReader;

console.log('\n[Reader Binding] === FINAL STATE ===');
console.log('[Reader Binding] NativeReader:', NativeReader ?  'LOADED ✅' : 'NOT LOADED ❌');
console.log('[Reader Binding] JSReader:', JSReader ? 'LOADED ✅' : 'NOT LOADED ❌');
console.log('[Reader Binding] _nativeAvailable:', Reader._nativeAvailable);
console.log('[Reader Binding] _jsAvailable:', Reader._jsAvailable);
console.log('[Reader Binding] === DIAGNOSTIC END ===');
console.log('========================================\n');

module.exports = Reader;