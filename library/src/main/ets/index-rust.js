"use strict";

const protobuf = require("./src/index");

console.log('[Protobuf] Loading module...');

try {
    const RustReader = require("./rust/src/bindings/reader");
    const RustWriter = require("./rust/src/bindings/writer");

    console.log('[Protobuf] Reader native available:', RustReader._nativeAvailable);
    console.log('[Protobuf] Writer native available:', RustWriter._nativeAvailable);

    if (RustReader._nativeAvailable) {
        protobuf.Reader = RustReader;
        console.log('✅ [Protobuf] Using Rust Reader');
    } else {
        console.log('⚠️ [Protobuf] Using JS Reader (fallback)');
    }

    if (RustWriter._nativeAvailable) {
        protobuf.Writer = RustWriter;
        console.log('✅ [Protobuf] Using Rust Writer');
    } else {
        console. log('⚠️ [Protobuf] Using JS Writer (fallback)');
    }

    protobuf.isUsingRust = () => RustReader._nativeAvailable || RustWriter._nativeAvailable;

} catch (e) {
    console.error('❌ [Protobuf] Failed to load Rust bindings:', e.message);
    // ❌ 删除这行，因为 OpenHarmony 没有 e. stack
    // console.error(e.stack);
    if (e.stack) {
        console.error(e.stack);
    }
    protobuf.isUsingRust = () => false;
}

console.log('[Protobuf] Final implementation:  Rust =', protobuf.isUsingRust?. () || false);

module.exports = protobuf;