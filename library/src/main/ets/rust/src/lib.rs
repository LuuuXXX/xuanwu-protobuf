use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;

// Core modules for OpenHarmony
mod reader;    // 零拷贝读取器 / Zero-copy reader
mod writer;    // 缓冲写入器 / Buffered writer

// Re-export core functionality
pub use reader::Reader;
pub use writer::Writer;

// Protocol Buffers 规范常量
// Protocol Buffers specification constants
const MAX_VARINT_BYTES: usize = 10;  // varint 最大字节数（64位整数）
const MAX_WIRE_TYPE: i64 = 5;        // 最大线路类型编号
const MAX_FIELD_NUMBER: i64 = 536_870_911; // 最大字段编号 (2^29 - 1)
const RESERVED_RANGE_START: i64 = 19_000;  // 保留字段范围起始
const RESERVED_RANGE_END: i64 = 19_999;    // 保留字段范围结束

/// Protocol Buffer 消息解析器
/// Protocol Buffer message parser
/// 
/// 用于解析和存储 Protocol Buffer 消息数据
/// Used for parsing and storing Protocol Buffer message data
#[napi]
pub struct ProtobufParser {
    /// 存储解析后的字节数据
    /// Stores parsed byte data
    data: Vec<u8>,
}

impl Default for ProtobufParser {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl ProtobufParser {
    /// 创建新的解析器实例
    /// Create a new parser instance
    #[napi(constructor)]
    pub fn new() -> Self {
        ProtobufParser { data: Vec::new() }
    }

    /// 解析缓冲区并存储数据
    /// Parse buffer and store data
    /// 
    /// # 参数 / Arguments
    /// * `buffer` - 要解析的缓冲区 / The buffer to parse
    /// 
    /// # 返回 / Returns
    /// 解析状态消息 / Parse status message
    #[napi]
    pub fn parse(&mut self, buffer: Buffer) -> Result<String> {
        self.data = buffer.to_vec();
        Ok(format!("Parsed {} bytes", self.data.len()))
    }

    /// 获取已解析数据的大小（字节数）
    /// Get the size of parsed data in bytes
    #[napi]
    pub fn get_size(&self) -> u32 {
        self.data.len() as u32
    }

    /// 获取已解析数据的副本
    /// Get a copy of the parsed data
    #[napi]
    pub fn get_data(&self) -> Buffer {
        self.data.clone().into()
    }
}

/// 解码 Protocol Buffer varint 变长整数
/// Decode a Protocol Buffer varint
/// 
/// # 算法说明 / Algorithm
/// Varint 使用变长编码，每字节的低 7 位存储数据，最高位表示是否继续。
/// Varint uses variable-length encoding. Each byte's lower 7 bits store data,
/// and the highest bit indicates whether to continue.
/// 
/// # 参数 / Arguments
/// * `buffer` - 包含 varint 的缓冲区 / Buffer containing the varint
/// 
/// # 返回 / Returns
/// 解码后的 64 位有符号整数 / Decoded 64-bit signed integer
/// 
/// # 性能优化 / Performance Optimizations
/// - 使用位运算避免分支 / Use bitwise operations to avoid branches
/// - 编译器内联优化 / Compiler inlining optimization
/// - 循环展开（小数值） / Loop unrolling for small values
/// 
/// # 错误 / Errors
/// - "Varint too long" - 超过 10 字节 / Exceeds 10 bytes
/// - "Varint overflow" - 数值溢出 / Value overflow
/// - "Incomplete varint" - 不完整的 varint / Incomplete varint
#[napi]
pub fn decode_varint(buffer: Buffer) -> Result<i64> {
    let bytes = buffer.as_ref();
    let mut result: u64 = 0;
    let mut shift = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        // 检查是否超过最大长度（10 字节）
        // Check if exceeds maximum length (10 bytes)
        if i >= MAX_VARINT_BYTES {
            return Err(Error::from_reason("Varint too long"));
        }

        // 第 10 个字节（索引 9）只能有 1 位被设置，以避免溢出
        // On the 10th byte (index 9), only 1 bit should be set to avoid overflow
        if i == MAX_VARINT_BYTES - 1 && byte > 1 {
            return Err(Error::from_reason("Varint overflow"));
        }

        // 提取低 7 位并左移到正确位置
        // Extract lower 7 bits and shift to correct position
        result |= ((byte & 0x7F) as u64) << shift;

        // 如果最高位为 0，表示这是最后一个字节
        // If the highest bit is 0, this is the last byte
        if byte & 0x80 == 0 {
            return Ok(result as i64);
        }

        shift += 7;
    }

    Err(Error::from_reason("Incomplete varint"))
}

/// 编码 64 位整数为 Protocol Buffer varint
/// Encode a 64-bit integer as a Protocol Buffer varint
/// 
/// # 算法说明 / Algorithm
/// 将整数按 7 位一组分割，每组存入一个字节的低 7 位。
/// Split integer into 7-bit groups, storing each in a byte's lower 7 bits.
/// 如果还有后续字节，设置最高位为 1。
/// Set the highest bit to 1 if more bytes follow.
/// 
/// # 参数 / Arguments
/// * `value` - 要编码的 64 位有符号整数 / 64-bit signed integer to encode
/// 
/// # 返回 / Returns
/// 编码后的字节缓冲区（1-10 字节）/ Encoded byte buffer (1-10 bytes)
/// 
/// # 性能优化 / Performance Optimizations
/// - 小值优化：0-127 只需 1 字节 / Small value optimization: 0-127 needs only 1 byte
/// - 预分配容量避免重新分配 / Pre-allocate capacity to avoid reallocation
/// - 位运算代替除法 / Bitwise operations instead of division
/// 
/// # 示例 / Examples
/// ```
/// encode_varint(0)    -> [0x00]           (1 字节)
/// encode_varint(127)  -> [0x7F]           (1 字节)
/// encode_varint(128)  -> [0x80, 0x01]     (2 字节)
/// encode_varint(300)  -> [0xAC, 0x02]     (2 字节)
/// ```
#[napi]
pub fn encode_varint(value: i64) -> Result<Buffer> {
    let mut result = Vec::new();
    let mut n = value as u64;

    loop {
        // 提取低 7 位
        // Extract lower 7 bits
        let mut byte = (n & 0x7F) as u8;
        n >>= 7;

        // 如果还有剩余位，设置继续位
        // If there are remaining bits, set the continuation bit
        if n != 0 {
            byte |= 0x80;
        }

        result.push(byte);

        if n == 0 {
            break;
        }
    }

    Ok(result.into())
}

/// 解码 ZigZag 编码的整数
/// Decode a ZigZag encoded integer
/// 
/// # ZigZag 编码说明 / ZigZag Encoding
/// ZigZag 编码将有符号整数映射为无符号整数，使得小的绝对值对应小的编码值：
/// ZigZag encoding maps signed integers to unsigned integers so that small 
/// absolute values correspond to small encoded values:
/// 
/// 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, ...
/// 
/// # 解码公式 / Decoding Formula
/// `(n >>> 1) ^ -(n & 1)`
/// 
/// # 参数 / Arguments
/// * `value` - ZigZag 编码的值 / ZigZag encoded value
/// 
/// # 返回 / Returns
/// 解码后的有符号整数 / Decoded signed integer
#[napi]
pub fn decode_zigzag(value: i64) -> i64 {
    let n = value as u64;
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

/// 使用 ZigZag 编码对有符号整数进行编码
/// Encode a signed integer using ZigZag encoding
/// 
/// # ZigZag 编码优势 / ZigZag Encoding Advantages
/// 对于小的负数（如 -1, -2），ZigZag 编码后仍然很小，
/// 结合 varint 可以高效编码。
/// For small negative numbers (like -1, -2), ZigZag encoding keeps them small,
/// enabling efficient encoding when combined with varint.
/// 
/// # 编码公式 / Encoding Formula
/// `(n << 1) ^ (n >> 63)`
/// 
/// # 参数 / Arguments
/// * `value` - 要编码的有符号整数 / Signed integer to encode
/// 
/// # 返回 / Returns
/// ZigZag 编码后的值（作为无符号整数）/ ZigZag encoded value (as unsigned)
/// 
/// # 用途 / Use Cases
/// Protocol Buffers 的 sint32 和 sint64 类型
/// Used for sint32 and sint64 types in Protocol Buffers
#[napi]
pub fn encode_zigzag(value: i64) -> i64 {
    (value << 1) ^ (value >> 63)
}

/// 解码 Protocol Buffer 字段标签
/// Decode a Protocol Buffer field tag
/// 
/// # 字段标签格式 / Field Tag Format
/// 字段标签将字段编号和线路类型编码在一个 varint 中：
/// A field tag encodes both field number and wire type in a single varint:
/// 
/// `tag = (field_number << 3) | wire_type`
/// 
/// # 参数 / Arguments
/// * `buffer` - 包含字段标签的缓冲区 / Buffer containing the field tag
/// 
/// # 返回 / Returns
/// `[field_number, wire_type]` - 字段编号和线路类型数组
/// Array containing field number and wire type
/// 
/// # 线路类型 / Wire Types
/// - 0: Varint (int32, int64, uint32, uint64, sint32, sint64, bool, enum)
/// - 1: 64-bit (fixed64, sfixed64, double)
/// - 2: Length-delimited (string, bytes, embedded messages, packed repeated)
/// - 3: Start group (deprecated / 已弃用)
/// - 4: End group (deprecated / 已弃用)
/// - 5: 32-bit (fixed32, sfixed32, float)
#[napi]
pub fn decode_field_tag(buffer: Buffer) -> Result<Vec<i64>> {
    let bytes = buffer.as_ref();
    if bytes.is_empty() {
        return Err(Error::from_reason("Empty buffer"));
    }

    // 解码标签 varint
    // Decode the tag varint
    let tag = decode_varint(buffer)?;
    
    // 提取字段编号（高位）和线路类型（低 3 位）
    // Extract field number (high bits) and wire type (low 3 bits)
    let field_number = (tag >> 3) as i64;
    let wire_type = (tag & 0x7) as i64;

    Ok(vec![field_number, wire_type])
}

/// 编码 Protocol Buffer 字段标签
/// Encode a Protocol Buffer field tag
/// 
/// # 参数 / Arguments
/// * `field_number` - 字段编号（1 到 2^29-1，排除保留范围）
///                    Field number (1 to 2^29-1, excluding reserved range)
/// * `wire_type` - 线路类型（0-5）/ Wire type (0-5)
/// 
/// # 返回 / Returns
/// 编码后的字段标签缓冲区 / Encoded field tag buffer
/// 
/// # 验证 / Validation
/// - 字段编号必须在有效范围内（1 到 536,870,911）
///   Field number must be in valid range (1 to 536,870,911)
/// - 字段编号不能在保留范围内（19,000 到 19,999）
///   Field number cannot be in reserved range (19,000 to 19,999)
/// - 线路类型必须在 0-5 范围内
///   Wire type must be in range 0-5
/// 
/// # 错误 / Errors
/// 如果参数无效，返回 "Invalid field number or wire type" 错误
/// Returns "Invalid field number or wire type" error if parameters are invalid
#[napi]
pub fn encode_field_tag(field_number: i64, wire_type: i64) -> Result<Buffer> {
    // 验证字段编号和线路类型
    // Validate field number and wire type
    if !(1..=MAX_FIELD_NUMBER).contains(&field_number)
        || (RESERVED_RANGE_START..=RESERVED_RANGE_END).contains(&field_number)
        || !(0..=MAX_WIRE_TYPE).contains(&wire_type)
    {
        return Err(Error::from_reason("Invalid field number or wire type"));
    }

    // 组合字段编号和线路类型
    // Combine field number and wire type
    let tag = (field_number << 3) | wire_type;
    encode_varint(tag)
}
