# 使用指南

本指南介绍如何使用 ohos_protobufrs 的 Rust 高性能实现。

## 目录

- [快速开始](#快速开始)
- [安装](#安装)
- [构建 Rust 模块](#构建-rust-模块)
- [基本使用](#基本使用)
- [高级功能](#高级功能)
- [API 文档](#api-文档)
- [迁移指南](#迁移指南)
- [故障排查](#故障排查)
- [常见问题 FAQ](#常见问题-faq)

## 快速开始

### 1. 安装项目

```bash
npm install protobufjs
```

### 2. 基本使用

```javascript
const protobuf = require('protobufjs');

// 加载 .proto 文件
const root = protobuf.loadSync('message.proto');
const Message = root.lookupType('MyMessage');

// 创建消息
const message = Message.create({ 
    id: 1, 
    name: 'Hello Rust!' 
});

// 编码（自动使用 Rust 加速，如果可用）
const buffer = Message.encode(message).finish();
console.log('Encoded:', buffer);

// 解码
const decoded = Message.decode(buffer);
console.log('Decoded:', decoded);
```

### 3. 构建 Rust 模块（可选，用于性能加速）

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建 Rust 模块
npm run build:rust
```

## 安装

### Node.js 环境

使用 npm 安装：

```bash
npm install protobufjs
```

或使用 yarn：

```bash
yarn add protobufjs
```

### 依赖要求

- **Node.js**: >= 12.0.0
- **Rust** (可选，用于高性能模块): >= 1.60.0

## 构建 Rust 模块

Rust 模块是可选的性能优化组件。即使不构建 Rust 模块，项目也能正常工作（使用纯 JavaScript 实现）。

### 安装 Rust 工具链

#### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Windows

下载并运行 [rustup-init.exe](https://rustup.rs/)

### 构建步骤

#### Unix/Linux/macOS

```bash
npm run build:rust
```

或手动构建：

```bash
cd rust
cargo build --release
cp target/release/libprotobuf_rs_ohos.* ../src/bindings/index.node
```

#### Windows

```bash
npm run build:rust:windows
```

或手动构建：

```cmd
cd rust
cargo build --release
copy target\release\protobuf_rs_ohos.dll ..\src\bindings\index.node
```

### 验证构建

构建成功后，运行以下命令验证：

```bash
node -e "const pb = require('./index.js'); console.log('Rust available:', pb.Reader._useNative);"
```

如果输出 `Rust available: true`，说明 Rust 模块已成功加载。

## 基本使用

### 使用 .proto 文件

```javascript
const protobuf = require('protobufjs');

// 异步加载
protobuf.load('awesome.proto', (err, root) => {
    if (err) throw err;
    
    const AwesomeMessage = root.lookupType('awesomepackage.AwesomeMessage');
    const message = AwesomeMessage.create({ awesomeField: 'hello' });
    const buffer = AwesomeMessage.encode(message).finish();
    const decoded = AwesomeMessage.decode(buffer);
});

// 同步加载
const root = protobuf.loadSync('awesome.proto');
const AwesomeMessage = root.lookupType('awesomepackage.AwesomeMessage');
```

### 使用 JSON 描述符

```javascript
const protobuf = require('protobufjs');

const jsonDescriptor = {
    nested: {
        AwesomeMessage: {
            fields: {
                awesomeField: {
                    type: 'string',
                    id: 1
                }
            }
        }
    }
};

const root = protobuf.Root.fromJSON(jsonDescriptor);
const AwesomeMessage = root.lookupType('AwesomeMessage');
```

### 编码和解码

```javascript
// 创建消息
const message = AwesomeMessage.create({ 
    awesomeField: 'awesome value' 
});

// 验证消息
const errMsg = AwesomeMessage.verify(message);
if (errMsg) throw Error(errMsg);

// 编码
const buffer = AwesomeMessage.encode(message).finish();

// 解码
try {
    const decoded = AwesomeMessage.decode(buffer);
    console.log(decoded);
} catch (e) {
    console.error('Decode error:', e);
}
```

## 高级功能

### 手动切换实现

```javascript
const protobuf = require('protobufjs');

// 检查 Rust 是否可用
if (typeof protobuf.useRust === 'function') {
    console.log('Rust implementation available');
    
    // 强制使用 JavaScript 实现
    protobuf.useJS();
    console.log('Using JS:', !protobuf.isUsingRust());
    
    // 切换到 Rust 实现
    protobuf.useRust();
    console.log('Using Rust:', protobuf.isUsingRust());
} else {
    console.log('Rust implementation not available');
}
```

### 性能测试模式

```javascript
const protobuf = require('protobufjs');
const Benchmark = require('benchmark');

// 准备测试数据
const Message = /* ... */;
const testData = { /* ... */ };

// 测试 JavaScript 实现
protobuf.useJS();
console.time('JS encoding');
for (let i = 0; i < 10000; i++) {
    Message.encode(testData).finish();
}
console.timeEnd('JS encoding');

// 测试 Rust 实现
protobuf.useRust();
console.time('Rust encoding');
for (let i = 0; i < 10000; i++) {
    Message.encode(testData).finish();
}
console.timeEnd('Rust encoding');
```

### 直接使用 Reader/Writer

```javascript
const protobuf = require('protobufjs');

// 使用 Writer 编码
const writer = new protobuf.Writer();
writer.uint32(1);           // 字段号
writer.string('Hello');     // 字符串值
writer.bool(true);          // 布尔值
const buffer = writer.finish();

// 使用 Reader 解码
const reader = new protobuf.Reader(buffer);
while (reader.pos < reader.len) {
    const tag = reader.uint32();
    const fieldNumber = tag >>> 3;
    const wireType = tag & 7;
    
    switch (fieldNumber) {
        case 1:
            const str = reader.string();
            break;
        case 2:
            const bool = reader.bool();
            break;
        default:
            reader.skipType(wireType);
    }
}
```

## API 文档

### 主要 API

#### protobuf.useRust()

切换到 Rust 实现（如果可用）。

```javascript
protobuf.useRust();
```

#### protobuf.useJS()

切换到 JavaScript 实现。

```javascript
protobuf.useJS();
```

#### protobuf.isUsingRust()

检查当前是否使用 Rust 实现。

```javascript
const isRust = protobuf.isUsingRust();
console.log('Using Rust:', isRust);
```

### Reader API

Reader 类用于解码 Protocol Buffers 消息。

```javascript
const reader = new protobuf.Reader(buffer);

// 读取各种类型
const uint32Value = reader.uint32();
const int32Value = reader.int32();
const stringValue = reader.string();
const boolValue = reader.bool();
const bytesValue = reader.bytes();
const doubleValue = reader.double();
const floatValue = reader.float();
```

### Writer API

Writer 类用于编码 Protocol Buffers 消息。

```javascript
const writer = new protobuf.Writer();

// 写入各种类型
writer.uint32(value);
writer.int32(value);
writer.string(value);
writer.bool(value);
writer.bytes(value);
writer.double(value);
writer.float(value);

// 完成并获取 Buffer
const buffer = writer.finish();
```

## 迁移指南

### 从纯 JavaScript 版本迁移

好消息！无需修改任何代码。Rust 实现完全向后兼容：

```javascript
// 原有代码保持不变
const protobuf = require('protobufjs');
const root = protobuf.loadSync('message.proto');
const Message = root.lookupType('MyMessage');

// 一切照旧工作，但性能更好（如果 Rust 模块可用）
const buffer = Message.encode(message).finish();
const decoded = Message.decode(buffer);
```

### 渐进式采用

如果你想渐进式地采用 Rust 实现：

```javascript
const protobuf = require('protobufjs');

// 检查是否支持 Rust
if (protobuf.isUsingRust && protobuf.isUsingRust()) {
    console.log('Great! Using Rust acceleration');
} else {
    console.log('Using JavaScript implementation (still fast!)');
}

// 其余代码不变
```

### 性能敏感的场景

在性能关键的代码路径中，确保使用 Rust：

```javascript
// 在性能关键的函数开始时
function performancecritical() {
    if (typeof protobuf.useRust === 'function') {
        protobuf.useRust();
    }
    
    // 执行大量编解码操作
    // ...
}
```

## 故障排查

### 问题：Rust 模块加载失败

**症状**：
```
[ohos_protobufrs] Rust native module not available, falling back to JavaScript implementation
```

**解决方案**：

1. 检查 Rust 是否已安装：
   ```bash
   rustc --version
   ```

2. 重新构建 Rust 模块：
   ```bash
   npm run build:rust
   ```

3. 检查构建输出是否有错误

### 问题：编译错误

**症状**：
```
error: linking with `cc` failed
```

**解决方案**：

1. 确保安装了 C/C++ 编译器：
   - Linux: `sudo apt-get install build-essential`
   - macOS: `xcode-select --install`
   - Windows: 安装 Visual Studio Build Tools

2. 更新 Rust 工具链：
   ```bash
   rustup update
   ```

### 问题：性能没有提升

**可能原因**：

1. Rust 模块未正确加载：
   ```javascript
   console.log('Using Rust:', protobuf.Reader._useNative);
   ```

2. 测试消息太小，看不出差异
3. Node.js V8 引擎已经优化了 JavaScript 代码

**验证方法**：

运行基准测试：
```bash
npm run bench:rust
```

### 问题：在某些平台上无法工作

**症状**：特定操作系统或架构上构建失败

**解决方案**：

1. 检查平台兼容性：
   - Linux x86_64: ✅ 完全支持
   - macOS x86_64/ARM64: ✅ 完全支持
   - Windows x86_64: ✅ 完全支持
   - 其他平台: 可能需要交叉编译

2. 在不支持的平台上，系统会自动降级到 JavaScript 实现

## 常见问题 FAQ

### Q: 是否必须安装 Rust？

**A**: 不是必须的。如果没有 Rust，项目会自动使用纯 JavaScript 实现。Rust 只是一个可选的性能优化。

### Q: 性能提升有多大？

**A**: 根据消息大小和复杂度，通常可以看到 50-150% 的性能提升。详见 [性能测试报告](./PERFORMANCE.md)。

### Q: Rust 实现是否完全兼容？

**A**: 是的。Rust 实现通过了所有 protobuf.js 的测试用例，API 行为完全一致。

### Q: 如何在生产环境部署？

**A**: 有两种方式：

1. **在目标环境构建**：在部署服务器上安装 Rust 并构建
2. **预编译二进制**：在 CI 中构建并打包 .node 文件

推荐方式二，可以避免在生产环境安装编译工具。

### Q: 如何调试 Rust 代码？

**A**: 

1. 使用 JavaScript 实现进行调试：
   ```javascript
   protobuf.useJS();
   ```

2. 在 Rust 代码中添加日志：
   ```rust
   println!("Debug: {:?}", value);
   ```

3. 使用 Rust 调试工具：
   ```bash
   cd rust
   cargo test
   ```

### Q: 能否在浏览器中使用？

**A**: Rust NAPI 模块只能在 Node.js 中使用。在浏览器中，会自动使用 JavaScript 实现。

### Q: 是否支持 TypeScript？

**A**: 是的。项目包含完整的 TypeScript 类型定义，Rust 实现不影响类型定义。

### Q: 如何贡献代码？

**A**: 欢迎贡献！请参考项目的 [CONTRIBUTING.md](https://github.com/LuuuXXX/ohos_protobufrs/blob/master/CONTRIBUTING.md)。

### Q: 性能测试数据准确吗？

**A**: 基准测试结果会受到多种因素影响（CPU、内存、Node.js 版本等）。建议在你的实际环境中运行 `npm run bench:rust` 获取准确数据。

### Q: 能否只在特定函数中使用 Rust？

**A**: 可以。使用 `useRust()` 和 `useJS()` 在运行时切换实现：

```javascript
function fastPath(data) {
    protobuf.useRust();
    return Message.encode(data).finish();
}

function debugPath(data) {
    protobuf.useJS();
    return Message.encode(data).finish();
}
```

### Q: Rust 模块的大小是多少？

**A**: 编译后的 .node 文件大约 1-2MB（release 模式）。这是一次性的磁盘开销，运行时内存占用很小。

### Q: 支持哪些 Protocol Buffers 版本？

**A**: 支持 Protocol Buffers v2 和 v3，与 protobuf.js 保持一致。

## 参考资源

### 官方文档

- [protobuf.js 文档](https://protobufjs.github.io/protobuf.js/)
- [Protocol Buffers 官方指南](https://protobuf.dev/)
- [NAPI-RS 文档](https://napi.rs/)

### 示例代码

查看 `examples/rust-example/` 目录获取更多示例：

```bash
node examples/rust-example/basic-usage.js
```

### 性能测试

查看 [性能测试报告](./PERFORMANCE.md) 了解详细的性能数据和分析。

### 社区支持

- [GitHub Issues](https://github.com/LuuuXXX/ohos_protobufrs/issues)
- [GitHub Discussions](https://github.com/LuuuXXX/ohos_protobufrs/discussions)

---

**最后更新**: 2025-12-16  
**维护者**: ohos_protobufrs 团队  
**许可证**: BSD-3-Clause
