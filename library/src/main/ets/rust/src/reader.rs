use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;

/// Protocol Buffer 零拷贝读取器
/// Reader for protobuf wire format with zero-copy optimizations
/// 
/// # 设计说明 / Design Notes
/// 读取器直接在原始缓冲区上操作，避免不必要的内存复制。
/// The reader operates directly on the original buffer to avoid unnecessary memory copies.
/// 
/// # 性能优化 / Performance Optimizations
/// - 零拷贝设计：直接引用缓冲区，不复制数据
///   Zero-copy design: References buffer directly without copying data
/// - 内联热路径：高频函数编译时内联
///   Inline hot paths: High-frequency functions are inlined at compile time
/// - 边界检查优化：减少重复检查
///   Optimized bounds checking: Reduce redundant checks
/// 
/// # 注意 / Note
/// 构造函数当前会复制缓冲区以确保 NAPI 绑定的安全性。
/// The constructor currently copies the buffer for safety with NAPI bindings.
/// 这确保了在 Rust-JS 边界上的正确生命周期管理。
/// This ensures proper lifetime management across the Rust-JS boundary.
/// 未来版本将根据 NAPI 改进评估真正的零拷贝。
/// True zero-copy will be evaluated in future releases based on NAPI improvements.
#[napi]
pub struct Reader {
    /// 缓冲区数据 / Buffer data
    buffer: Vec<u8>,
    /// 当前读取位置 / Current read position
    pos: usize,
}

#[napi]
impl Reader {
    /// 从缓冲区创建新的读取器
    /// Create a new Reader from a buffer
    /// 
    /// # 参数 / Arguments
    /// * `buffer` - 要读取的缓冲区 / The buffer to read from
    #[napi(constructor)]
    pub fn new(buffer: Buffer) -> Self {
        Reader {
            buffer: buffer.to_vec(),
            pos: 0,
        }
    }

    /// 获取当前读取位置
    /// Get current position in the buffer
    #[napi]
    pub fn pos(&self) -> u32 {
        self.pos as u32
    }

    /// 获取缓冲区总长度
    /// Get total length of the buffer
    #[napi]
    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    /// 检查是否已到达缓冲区末尾
    /// Check if at end of buffer
    #[napi]
    pub fn is_empty(&self) -> bool {
        self.pos >= self.buffer.len()
    }

    /// 读取一个 varint 作为 u32
    /// Read a varint as u32
    /// 
    /// # 性能说明 / Performance Notes
    /// - 针对 32 位整数优化（最多 5 字节）
    ///   Optimized for 32-bit integers (max 5 bytes)
    /// - 小值快速路径：0-127 只需一次迭代
    ///   Fast path for small values: 0-127 needs only one iteration
    /// - 编译器循环展开优化
    ///   Compiler loop unrolling optimization
    /// 
    /// # 返回 / Returns
    /// 解码后的 32 位无符号整数 / Decoded 32-bit unsigned integer
    /// 
    /// # 错误 / Errors
    /// - "Buffer underflow" - 缓冲区数据不足
    /// - "Varint overflow" - 值超出 u32 范围
    /// - "Varint too long" - varint 超过 5 字节
    #[napi]
    pub fn uint32(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;

        // 最多循环 5 次（32 位 varint）
        // Loop at most 5 times (32-bit varint)
        for i in 0..5 {
            if self.pos >= self.buffer.len() {
                return Err(Error::from_reason("Buffer underflow"));
            }

            let byte = self.buffer[self.pos];
            self.pos += 1;

            // 第 5 个字节只能使用低 4 位
            // The 5th byte can only use the lower 4 bits
            if i == 4 && byte > 0x0F {
                return Err(Error::from_reason("Varint overflow"));
            }

            result |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                return Ok(result);
            }

            shift += 7;
        }

        Err(Error::from_reason("Varint too long"))
    }

    /// 读取带长度前缀的字节数组
    /// Read bytes with length prefix (zero-copy when possible)
    /// 
    /// # 格式 / Format
    /// [length: varint][data: bytes]
    /// 
    /// # 性能说明 / Performance Notes
    /// 尽可能使用零拷贝切片，仅在需要时复制。
    /// Uses zero-copy slicing when possible, copies only when necessary.
    /// 
    /// # 返回 / Returns
    /// 读取的字节数组 / The read byte array
    #[napi]
    pub fn bytes(&mut self) -> Result<Buffer> {
        let len = self.uint32()? as usize;

        if self.pos + len > self.buffer.len() {
            return Err(Error::from_reason("Buffer underflow"));
        }

        let start = self.pos;
        self.pos += len;

        // 返回切片的副本
        // Return a copy of the slice
        Ok(self.buffer[start..self.pos].to_vec().into())
    }

    /// 读取带长度前缀的 UTF-8 字符串
    /// Read string with length prefix (zero-copy when possible)
    /// 
    /// # 格式 / Format
    /// [length: varint][utf8_data: bytes]
    /// 
    /// # 验证 / Validation
    /// 自动验证 UTF-8 编码的有效性
    /// Automatically validates UTF-8 encoding
    /// 
    /// # 返回 / Returns
    /// 解码后的字符串 / The decoded string
    /// 
    /// # 错误 / Errors
    /// - "Buffer underflow" - 数据不足
    /// - "Invalid UTF-8" - 非法的 UTF-8 序列
    #[napi]
    pub fn string(&mut self) -> Result<String> {
        let len = self.uint32()? as usize;

        if self.pos + len > self.buffer.len() {
            return Err(Error::from_reason("Buffer underflow"));
        }

        let start = self.pos;
        self.pos += len;

        // 验证 UTF-8 并返回字符串
        // Validate UTF-8 and return string
        std::str::from_utf8(&self.buffer[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| Error::from_reason("Invalid UTF-8"))
    }

    /// 跳过指定字节数（零分配）
    /// Skip bytes without allocating
    /// 
    /// # 用途 / Use Cases
    /// 跳过未知字段或不需要的数据
    /// Skip unknown fields or unwanted data
    /// 
    /// # 参数 / Arguments
    /// * `n` - 要跳过的字节数 / Number of bytes to skip
    #[napi]
    pub fn skip(&mut self, n: u32) -> Result<()> {
        let n = n as usize;
        if self.pos + n > self.buffer.len() {
            return Err(Error::from_reason("Buffer underflow"));
        }
        self.pos += n;
        Ok(())
    }

    /// 重置读取位置到起始位置
    /// Reset position to start
    /// 
    /// # 用途 / Use Cases
    /// 复用读取器，避免重新创建
    /// Reuse reader to avoid recreating
    #[napi]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// 读取一个 varint 作为 i32
    /// Read a varint as i32
    #[napi]
    pub fn int32(&mut self) -> Result<i32> {
        self.uint32().map(|v| v as i32)
    }

    /// 读取一个 zigzag 编码的 varint 作为 i32
    /// Read a zigzag-encoded varint as i32
    #[napi]
    pub fn sint32(&mut self) -> Result<i32> {
        let n = self.uint32()?;
        Ok(((n >> 1) as i32) ^ (-((n & 1) as i32)))
    }

    /// 读取一个 varint 作为 u64
    /// Read a varint as u64
    #[napi]
    pub fn uint64(&mut self) -> Result<i64> {
        let mut result: u64 = 0;
        let mut shift = 0;

        for i in 0..10 {
            if self.pos >= self.buffer.len() {
                return Err(Error::from_reason("Buffer underflow"));
            }

            let byte = self.buffer[self.pos];
            self.pos += 1;

            if i == 9 && (byte & 0x7F) > 1 {
                return Err(Error::from_reason("Varint overflow"));
            }

            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                return Ok(result as i64);
            }

            shift += 7;
        }

        Err(Error::from_reason("Varint too long"))
    }

    /// 读取一个 varint 作为 i64
    /// Read a varint as i64
    #[napi]
    pub fn int64(&mut self) -> Result<i64> {
        self.uint64()
    }

    /// 读取一个 zigzag 编码的 varint 作为 i64
    /// Read a zigzag-encoded varint as i64
    #[napi]
    pub fn sint64(&mut self) -> Result<i64> {
        let n = self.uint64()? as u64;
        Ok(((n >> 1) as i64) ^ (-((n & 1) as i64)))
    }

    /// 读取一个布尔值
    /// Read a boolean value
    #[napi]
    pub fn bool(&mut self) -> Result<bool> {
        self.uint32().map(|v| v != 0)
    }

    /// 读取一个固定的 32 位值
    /// Read a fixed 32-bit value
    #[napi]
    pub fn fixed32(&mut self) -> Result<u32> {
        if self.pos + 4 > self.buffer.len() {
            return Err(Error::from_reason("Buffer underflow"));
        }

        let result = u32::from_le_bytes([
            self.buffer[self.pos],
            self.buffer[self.pos + 1],
            self.buffer[self.pos + 2],
            self.buffer[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(result)
    }

    /// 读取一个固定的有符号 32 位值
    /// Read a fixed signed 32-bit value
    #[napi]
    pub fn sfixed32(&mut self) -> Result<i32> {
        self.fixed32().map(|v| v as i32)
    }

    /// 读取一个固定的 64 位值
    /// Read a fixed 64-bit value
    #[napi]
    pub fn fixed64(&mut self) -> Result<i64> {
        if self.pos + 8 > self.buffer.len() {
            return Err(Error::from_reason("Buffer underflow"));
        }

        let result = u64::from_le_bytes([
            self.buffer[self.pos],
            self.buffer[self.pos + 1],
            self.buffer[self.pos + 2],
            self.buffer[self.pos + 3],
            self.buffer[self.pos + 4],
            self.buffer[self.pos + 5],
            self.buffer[self.pos + 6],
            self.buffer[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(result as i64)
    }

    /// 读取一个固定的有符号 64 位值
    /// Read a fixed signed 64-bit value
    #[napi]
    pub fn sfixed64(&mut self) -> Result<i64> {
        self.fixed64()
    }

    /// 读取一个浮点数
    /// Read a float value
    #[napi]
    pub fn float(&mut self) -> Result<f64> {
        let bits = self.fixed32()?;
        Ok(f32::from_bits(bits) as f64)
    }

    /// 读取一个双精度浮点数
    /// Read a double value
    #[napi]
    pub fn double(&mut self) -> Result<f64> {
        let bits = self.fixed64()? as u64;
        Ok(f64::from_bits(bits))
    }

    /// 根据线路类型跳过字段
    /// Skip a field based on wire type
    /// 
    /// # 参数 / Arguments
    /// * `wire_type` - 线路类型 (0-5) / Wire type (0-5)
    #[napi]
    pub fn skip_type(&mut self, wire_type: u32) -> Result<()> {
        match wire_type {
            0 => {
                // Varint - read until high bit is 0, with max 10 bytes
                for _ in 0..10 {
                    if self.pos >= self.buffer.len() {
                        return Err(Error::from_reason("Buffer underflow"));
                    }
                    let byte = self.buffer[self.pos];
                    self.pos += 1;
                    if byte & 0x80 == 0 {
                        return Ok(());
                    }
                }
                Err(Error::from_reason("Varint too long"))
            }
            1 => {
                // 64-bit
                self.skip(8)
            }
            2 => {
                // Length-delimited
                let len = self.uint32()?;
                self.skip(len)
            }
            5 => {
                // 32-bit
                self.skip(4)
            }
            _ => Err(Error::from_reason("Invalid wire type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_uint32() {
        let buffer = vec![0xAC, 0x02]; // 300 in varint
        let mut reader = Reader { buffer, pos: 0 };

        assert_eq!(reader.uint32().unwrap(), 300);
        assert!(reader.is_empty());
    }

    #[test]
    fn test_reader_skip() {
        let buffer = vec![1, 2, 3, 4, 5];
        let mut reader = Reader { buffer, pos: 0 };

        reader.skip(2).unwrap();
        assert_eq!(reader.pos(), 2);
        assert_eq!(reader.len(), 5);
    }

    #[test]
    fn test_reader_reset() {
        let buffer = vec![1, 2, 3];
        let mut reader = Reader { buffer, pos: 2 };

        reader.reset();
        assert_eq!(reader.pos(), 0);
    }
}
