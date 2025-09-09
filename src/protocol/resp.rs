use bytes::{Buf, BytesMut};

use crate::SyncError;

#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespValue>),
    Null,
}

pub struct RespParser;

impl RespParser {
    pub fn parse(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        if buf.is_empty() {
            return Ok(None);
        }

        let first_byte = buf[0] as char;
        match first_byte {
            '+' => Self::parse_simple_string(buf),
            '-' => Self::parse_error(buf),
            ':' => Self::parse_integer(buf),
            '$' => Self::parse_bulk_string(buf),
            '*' => Self::parse_array(buf),
            '_' => Self::parse_null(buf),
            _ => Err("Invalid RESP type".into()),
        }
    }

    fn parse_null(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        if buf.as_ref() == b"_\r\n" {
            Ok(Some(RespValue::Null))
        } else {
            Ok(None)
        }
    }

    fn parse_simple_string(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        // 查找 \r\n
        if let Some(pos) = buf.iter().position(|&b| b == b'\r') {
            // 确保下一个字节是 \n
            if pos + 1 < buf.len() && buf[pos + 1] == b'\n' {
                // 提取字符串内容（跳过 '+' 和 \r\n）
                let content = String::from_utf8(buf[1..pos].to_vec())?;
                // 移除已解析的数据
                buf.advance(pos + 2);
                return Ok(Some(RespValue::SimpleString(content)));
            }
        }
        Ok(None) // 数据不完整
    }

    fn parse_error(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        // 查找 \r\n
        if let Some(pos) = buf.iter().position(|&b| b == b'\r') {
            // 确保下一个字节是 \n
            if pos + 1 < buf.len() && buf[pos + 1] == b'\n' {
                // 提取错误内容（跳过 '-' 和 \r\n）
                let content = String::from_utf8(buf[1..pos].to_vec())?;
                // 移除已解析的数据
                buf.advance(pos + 2);
                return Ok(Some(RespValue::Error(content)));
            }
        }
        Ok(None) // 数据不完整
    }

    fn parse_integer(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        // 查找 \r\n
        if let Some(pos) = buf.iter().position(|&b| b == b'\r') {
            // 确保下一个字节是 \n
            if pos + 1 < buf.len() && buf[pos + 1] == b'\n' {
                // 提取整数内容（跳过 ':' 和 \r\n）
                let content = std::str::from_utf8(&buf[1..pos])?;
                let number = content.parse::<i64>()?;
                // 移除已解析的数据
                buf.advance(pos + 2);
                return Ok(Some(RespValue::Integer(number)));
            }
        }
        Ok(None) // 数据不完整
    }

    fn parse_bulk_string(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        // 查找第一个 \r\n 来获取长度
        if let Some(pos) = buf.iter().position(|&b| b == b'\r') {
            // 确保下一个字节是 \n
            if pos + 1 < buf.len() && buf[pos + 1] == b'\n' {
                // 提取长度（跳过 '$' 和 \r\n）
                let length_str = std::str::from_utf8(&buf[1..pos])?;

                if length_str == "-1" {
                    // Null bulk string
                    buf.advance(pos + 2);
                    return Ok(Some(RespValue::Null));
                }

                let length = length_str.parse::<usize>()?;

                // 检查是否有足够的数据（长度 + \r\n + 数据 + \r\n）
                let total_needed = pos + 2 + length + 2;
                if buf.len() < total_needed {
                    return Ok(None); // 数据不完整
                }

                // 提取字符串数据
                let data_start = pos + 2;
                let data_end = data_start + length;

                // 验证后面是否有 \r\n
                if data_end + 1 < buf.len() && buf[data_end] == b'\r' && buf[data_end + 1] == b'\n'
                {
                    let content = String::from_utf8(buf[data_start..data_end].to_vec())?;
                    buf.advance(total_needed);
                    return Ok(Some(RespValue::BulkString(Some(content))));
                }
            }
        }
        Ok(None) // 数据不完整
    }

    fn parse_array(buf: &mut BytesMut) -> Result<Option<RespValue>, Box<SyncError>> {
        // 查找第一个 \r\n 来获取数组长度
        if let Some(pos) = buf.iter().position(|&b| b == b'\r') {
            // 确保下一个字节是 \n
            if pos + 1 < buf.len() && buf[pos + 1] == b'\n' {
                // 提取数组长度（跳过 '*' 和 \r\n）
                let length_str = std::str::from_utf8(&buf[1..pos])?;
                let length = length_str.parse::<i64>()?;

                // 移除长度行
                buf.advance(pos + 2);

                if length == -1 {
                    // Null array
                    return Ok(Some(RespValue::Null));
                }

                let length = length as usize;
                let mut elements = Vec::with_capacity(length);
                let mut parsed_count = 0;

                // 临时缓冲区用于回滚
                // let original_len = buf.len();

                // 解析数组元素
                while parsed_count < length {
                    match Self::parse(buf)? {
                        Some(value) => {
                            elements.push(value);
                            parsed_count += 1;
                        }
                        None => {
                            // 数据不完整，回滚并返回 None
                            return Ok(None);
                        }
                    }
                }

                return Ok(Some(RespValue::Array(elements)));
            }
        }
        Ok(None) // 数据不完整
    }

    pub fn serializer(response: RespValue) -> Vec<u8> {
        match response {
            RespValue::SimpleString(s) => {
                let mut result = Vec::with_capacity(s.len() + 4);
                result.push(b'+');
                result.extend_from_slice(s.as_bytes());
                result.push(b'\r');
                result.push(b'\n');
                result
            }

            RespValue::Error(s) => {
                let mut result = Vec::with_capacity(s.len() + 4);
                result.push(b'-');
                result.extend_from_slice(s.as_bytes());
                result.push(b'\r');
                result.push(b'\n');
                result
            }

            RespValue::Integer(n) => {
                let mut result = Vec::new();
                result.push(b':');
                result.extend_from_slice(n.to_string().as_bytes());
                result.push(b'\r');
                result.push(b'\n');
                result
            }

            RespValue::BulkString(None) => {
                // Null bulk string
                b"$-1\r\n".to_vec()
            }

            RespValue::BulkString(Some(data)) => {
                let mut result = Vec::new();
                result.push(b'$');
                result.extend_from_slice(data.len().to_string().as_bytes());
                result.push(b'\r');
                result.push(b'\n');
                result.extend_from_slice(data.as_bytes());
                result.push(b'\r');
                result.push(b'\n');
                result
            }

            /*RespValue::Array(None) => {
                // Null array
                b"*-1\r\n".to_vec()
            }*/
            RespValue::Array(values) => {
                let mut result = Vec::new();
                result.push(b'*');
                result.extend_from_slice(values.len().to_string().as_bytes());
                result.push(b'\r');
                result.push(b'\n');

                for value in values {
                    let serialized = Self::serializer(value);
                    result.extend_from_slice(&serialized);
                }
                result
            }

            RespValue::Null => b"_\r\n".to_vec(),
        }
    }
}

