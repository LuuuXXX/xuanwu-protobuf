use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;

/// Protocol Buffer 缓冲写入器
/// Writer for protobuf wire format with buffer optimization
/// 
/// # 设计说明 / Design Notes
/// 写入器使用动态增长的缓冲区来收集编码数据。
/// The writer uses a dynamically growing buffer to collect encoded data.
/// 
/// # 性能优化 / Performance Optimizations
/// - 预分配容量：减少内存重新分配
///   Pre-allocated capacity: Reduces memory reallocations
/// - 缓冲区复用：reset() 后可复用，避免重复分配
///   Buffer reuse: Can reuse after reset(), avoiding repeated allocations
/// - 批量写入：使用 extend_from_slice 批量复制
///   Batch writes: Uses extend_from_slice for bulk copying
/// - 编译器内联：小函数自动内联
///   Compiler inlining: Small functions are automatically inlined
/// 
/// # 注意 / Note
/// finish() 方法当前会克隆缓冲区以保持 NAPI 的所有权语义。
/// The finish() method currently clones the buffer to maintain ownership
/// semantics with NAPI.
/// 未来版本将探索替代 API（finish_into、consume）以实现真正的零拷贝操作。
/// Alternative APIs (finish_into, consume) will be explored in future releases
/// for true zero-copy operation.
#[napi]
pub struct Writer {
    /// 内部缓冲区 / Internal buffer
    buffer: Vec<u8>,
}

#[napi]
impl Writer {
    /// 创建新的写入器
    /// Create a new Writer
    #[napi(constructor)]
    pub fn new() -> Self {
        Writer { buffer: Vec::new() }
    }

    /// 创建具有预分配容量的写入器
    /// Create a new Writer with pre-allocated capacity
    /// 
    /// # 性能提示 / Performance Tip
    /// 如果知道大致的消息大小，预分配容量可以减少重新分配。
    /// If you know the approximate message size, pre-allocating capacity
    /// can reduce reallocations.
    /// 
    /// # 参数 / Arguments
    /// * `capacity` - 初始容量（字节数）/ Initial capacity in bytes
    /// 
    /// # 示例 / Example
    /// ```
    /// let writer = Writer::with_capacity(1024);  // 预分配 1KB
    /// ```
    #[napi(factory)]
    pub fn with_capacity(capacity: u32) -> Self {
        Writer {
            buffer: Vec::with_capacity(capacity as usize),
        }
    }

    /// 写入一个 u32 作为 varint
    /// Write a u32 as varint
    /// 
    /// # 编码格式 / Encoding Format
    /// 使用 Protocol Buffers varint 编码，小值使用更少字节。
    /// Uses Protocol Buffers varint encoding, smaller values use fewer bytes.
    /// 
    /// # 性能说明 / Performance Notes
    /// - 0-127: 1 字节
    /// - 128-16383: 2 字节
    /// - 16384-2097151: 3 字节
    /// - 2097152-268435455: 4 字节
    /// - 268435456-4294967295: 5 字节
    /// 
    /// # 参数 / Arguments
    /// * `value` - 要写入的 32 位无符号整数 / 32-bit unsigned integer to write
    #[napi]
    pub fn uint32(&mut self, value: u32) {
        let mut n = value;
        loop {
            // 提取低 7 位
            // Extract lower 7 bits
            let mut byte = (n & 0x7F) as u8;
            n >>= 7;

            // 如果还有更多位，设置继续位
            // If there are more bits, set continuation bit
            if n != 0 {
                byte |= 0x80;
            }

            self.buffer.push(byte);

            if n == 0 {
                break;
            }
        }
    }

    /// 写入带长度前缀的字节数组
    /// Write bytes with length prefix
    /// 
    /// # 格式 / Format
    /// [length: varint][data: bytes]
    /// 
    /// # 用途 / Use Cases
    /// - 嵌套消息 / Embedded messages
    /// - 字节数组字段 / Byte array fields
    /// - 打包的重复字段 / Packed repeated fields
    /// 
    /// # 参数 / Arguments
    /// * `value` - 要写入的字节缓冲区 / Byte buffer to write
    #[napi]
    pub fn bytes(&mut self, value: Buffer) {
        let bytes = value.as_ref();
        // 先写入长度
        // Write length first
        self.uint32(bytes.len() as u32);
        // 再写入数据
        // Then write data
        self.buffer.extend_from_slice(bytes);
    }

    /// 写入带长度前缀的 UTF-8 字符串
    /// Write string with length prefix
    /// 
    /// # 格式 / Format
    /// [length: varint][utf8_data: bytes]
    /// 
    /// # 编码 / Encoding
    /// 字符串自动编码为 UTF-8 字节序列
    /// Strings are automatically encoded as UTF-8 byte sequences
    /// 
    /// # 参数 / Arguments
    /// * `value` - 要写入的字符串 / String to write
    #[napi]
    pub fn string(&mut self, value: String) {
        let bytes = value.as_bytes();
        // 先写入字节长度
        // Write byte length first
        self.uint32(bytes.len() as u32);
        // 再写入 UTF-8 数据
        // Then write UTF-8 data
        self.buffer.extend_from_slice(bytes);
    }

    /// 完成写入并获取缓冲区
    /// Get the finished buffer
    /// 
    /// # 返回 / Returns
    /// 包含所有已写入数据的缓冲区 / Buffer containing all written data
    /// 
    /// # 所有权说明 / Ownership Notes
    /// 此方法使用 `&self`（非 `self`），因此写入器在调用后仍然可用。
    /// This method uses `&self` (not `self`), so the writer remains usable after calling.
    /// 当前实现会克隆缓冲区以维护 NAPI 所有权语义。
    /// Currently clones the buffer to maintain NAPI ownership semantics.
    /// 如果需要复用写入器，之后调用 reset()。
    /// Call reset() afterwards if you want to reuse the writer.
    #[napi]
    pub fn finish(&self) -> Buffer {
        self.buffer.clone().into()
    }

    /// 获取当前缓冲区大小（已写入字节数）
    /// Get estimated buffer size (current size)
    /// 
    /// # 用途 / Use Cases
    /// 用于监控写入进度或预估最终大小
    /// Used for monitoring write progress or estimating final size
    #[napi]
    pub fn estimated_size(&self) -> u32 {
        self.buffer.len() as u32
    }

    /// 重置写入器以供复用
    /// Reset the writer for reuse
    /// 
    /// # 性能说明 / Performance Notes
    /// 清空缓冲区但保留已分配的容量，避免重新分配。
    /// Clears the buffer but keeps allocated capacity, avoiding reallocation.
    /// 
    /// # 示例 / Example
    /// ```
    /// let mut writer = Writer::with_capacity(1024);
    /// writer.uint32(100);
    /// let buffer1 = writer.finish();
    /// 
    /// writer.reset();  // 复用，无需重新分配
    /// writer.uint32(200);
    /// let buffer2 = writer.finish();
    /// ```
    #[napi]
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// 获取当前已写入的字节数
    /// Get current length
    #[napi]
    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    /// 检查缓冲区是否为空
    /// Check if empty
    #[napi]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 写入一个 i32 作为 varint
    /// Write an i32 as varint
    #[napi]
    pub fn int32(&mut self, value: i32) {
        // i32 is encoded as u64 varint (sign-extended to 64 bits)
        self.uint64(value as i64);
    }

    /// 写入一个 zigzag 编码的 i32 作为 varint
    /// Write a zigzag-encoded i32 as varint
    #[napi]
    pub fn sint32(&mut self, value: i32) {
        let n = ((value << 1) ^ (value >> 31)) as u32;
        self.uint32(n);
    }

    /// 写入一个 u64 作为 varint
    /// Write a u64 as varint
    #[napi]
    pub fn uint64(&mut self, value: i64) {
        // JavaScript numbers are i64, so we accept i64 and treat as u64
        let mut n = value as u64;
        loop {
            let mut byte = (n & 0x7F) as u8;
            n >>= 7;

            if n != 0 {
                byte |= 0x80;
            }

            self.buffer.push(byte);

            if n == 0 {
                break;
            }
        }
    }

    /// 写入一个 i64 作为 varint
    /// Write an i64 as varint
    #[napi]
    pub fn int64(&mut self, value: i64) {
        self.uint64(value);
    }

    /// 写入一个 zigzag 编码的 i64 作为 varint
    /// Write a zigzag-encoded i64 as varint
    #[napi]
    pub fn sint64(&mut self, value: i64) {
        let n = ((value << 1) ^ (value >> 63)) as u64;
        self.uint64(n as i64);
    }

    /// 写入一个布尔值
    /// Write a boolean value
    #[napi]
    pub fn bool(&mut self, value: bool) {
        self.uint32(if value { 1 } else { 0 });
    }

    /// 写入一个固定的 32 位值
    /// Write a fixed 32-bit value
    #[napi]
    pub fn fixed32(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入一个固定的有符号 32 位值
    /// Write a fixed signed 32-bit value
    #[napi]
    pub fn sfixed32(&mut self, value: i32) {
        self.fixed32(value as u32);
    }

    /// 写入一个固定的 64 位值
    /// Write a fixed 64-bit value
    #[napi]
    pub fn fixed64(&mut self, value: i64) {
        self.buffer.extend_from_slice(&(value as u64).to_le_bytes());
    }

    /// 写入一个固定的有符号 64 位值
    /// Write a fixed signed 64-bit value
    #[napi]
    pub fn sfixed64(&mut self, value: i64) {
        self.fixed64(value);
    }

    /// 写入一个浮点数
    /// Write a float value
    #[napi]
    pub fn float(&mut self, value: f64) {
        self.fixed32((value as f32).to_bits());
    }

    /// 写入一个双精度浮点数
    /// Write a double value
    #[napi]
    pub fn double(&mut self, value: f64) {
        self.fixed64(value.to_bits() as i64);
    }

    /// 创建一个子写入器用于长度限定消息
    /// Fork a new writer for length-delimited messages
    /// 
    /// # 返回 / Returns
    /// 当前缓冲区长度，用于稍后计算消息长度
    /// Current buffer length, used to calculate message length later
    #[napi]
    pub fn fork(&mut self) -> u32 {
        // Reserve space for varint length (max 5 bytes for typical messages)
        // We'll write the actual length in ldelim()
        let pos = self.buffer.len() as u32;
        // Push a placeholder byte - will be updated in ldelim
        self.buffer.push(0);
        pos
    }

    /// 写入长度限定符（在 fork 之后使用）
    /// Write length delimiter (use after fork)
    /// 
    /// # 参数 / Arguments
    /// * `fork_pos` - fork() 返回的位置 / Position returned by fork()
    #[napi]
    pub fn ldelim(&mut self, fork_pos: u32) -> Result<()> {
        let fork_pos = fork_pos as usize;
        let current_len = self.buffer.len();
        
        // Calculate the message length (excluding the length varint itself)
        let msg_len = (current_len - fork_pos - 1) as u32;
        
        // Encode the length as varint
        let mut len_bytes = Vec::new();
        let mut n = msg_len;
        loop {
            let mut byte = (n & 0x7F) as u8;
            n >>= 7;
            if n != 0 {
                byte |= 0x80;
            }
            len_bytes.push(byte);
            if n == 0 {
                break;
            }
        }
        
        // Check if we need more space than the placeholder byte
        if len_bytes.len() > 1 {
            // Need to insert additional bytes
            let extra = len_bytes.len() - 1;
            self.buffer.resize(current_len + extra, 0);
            // Shift the message data to make room
            self.buffer.copy_within(fork_pos + 1..current_len, fork_pos + len_bytes.len());
        }
        
        // Write the length varint at fork position
        for (i, &byte) in len_bytes.iter().enumerate() {
            self.buffer[fork_pos + i] = byte;
        }
        
        Ok(())
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_uint32() {
        let mut writer = Writer::new();
        writer.uint32(300);
        let buffer = writer.finish();
        assert_eq!(buffer.as_ref(), &[0xAC, 0x02]);
    }

    #[test]
    fn test_writer_string() {
        let mut writer = Writer::new();
        writer.string("hello".to_string());
        let buffer = writer.finish();
        // Length prefix (5) + "hello"
        assert_eq!(buffer.as_ref(), &[5, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_writer_reset() {
        let mut writer = Writer::new();
        writer.uint32(123);
        assert!(!writer.is_empty());

        writer.reset();
        assert!(writer.is_empty());
        assert_eq!(writer.len(), 0);
    }

    #[test]
    fn test_writer_with_capacity() {
        let writer = Writer::with_capacity(100);
        assert_eq!(writer.len(), 0);
        assert!(writer.buffer.capacity() >= 100);
    }

    #[test]
    fn test_writer_chaining() {
        let mut writer = Writer::new();
        writer.uint32(100);
        writer.uint32(200);
        writer.uint32(300);
        assert!(writer.len() > 0);
    }
}
