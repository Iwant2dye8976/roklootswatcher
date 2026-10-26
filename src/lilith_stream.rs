use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LilithValue {
    Null,
    Bool(bool),
    U8(u8),
    Number(f64),
    String(String),
    Table(IndexMap<String, LilithValue>),
}

pub const MAX_MAIL_FILE_SIZE: u64 = 50 * 1024 * 1024;
pub const MAIL_DECODE_BATCH_SIZE: usize = 4;
pub const MAX_RECURSION_DEPTH: usize = 128;

#[allow(dead_code)]
impl LilithValue {
    pub fn get(&self, key: &str) -> Option<&LilithValue> {
        match self {
            LilithValue::Table(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            LilithValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            LilithValue::Number(n) => Some(*n),
            LilithValue::U8(u) => Some(*u as f64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            LilithValue::Number(n) => {
                if n.is_finite() && *n >= (i64::MIN as f64) && *n <= (i64::MAX as f64) {
                    Some(n.round() as i64)
                } else {
                    None
                }
            }
            LilithValue::U8(u) => Some(*u as i64),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            LilithValue::Number(n) => {
                if n.is_finite() && *n >= 0.0 && *n <= (u32::MAX as f64) {
                    Some(n.round() as u32)
                } else {
                    None
                }
            }
            LilithValue::U8(u) => Some(*u as u32),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            LilithValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&IndexMap<String, LilithValue>> {
        match self {
            LilithValue::Table(m) => Some(m),
            _ => None,
        }
    }

    pub fn to_key_string(&self) -> String {
        match self {
            LilithValue::String(s) => s.clone(),
            LilithValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            LilithValue::U8(u) => format!("{}", u),
            LilithValue::Bool(b) => format!("{}", b),
            LilithValue::Null => "null".to_string(),
            LilithValue::Table(_) => "[Table]".to_string(),
        }
    }
}

pub fn parse_stream(data: &[u8], pos: &mut usize) -> Result<LilithValue, String> {
    parse_stream_internal(data, pos, 0)
}

fn parse_stream_internal(data: &[u8], pos: &mut usize, depth: usize) -> Result<LilithValue, String> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!("Exceeded maximum recursion depth of {}", MAX_RECURSION_DEPTH));
    }

    if *pos >= data.len() {
        return Ok(LilithValue::Null);
    }

    let tag = data[*pos];
    *pos += 1;

    match tag {
        0x00 => Ok(LilithValue::Null),
        0x01 => {
            if *pos >= data.len() {
                return Err("Unexpected EOF parsing Bool".into());
            }
            let val = data[*pos] != 0;
            *pos += 1;
            Ok(LilithValue::Bool(val))
        }
        0x02 => {
            if *pos >= data.len() {
                return Err("Unexpected EOF parsing U8".into());
            }
            let val = data[*pos];
            *pos += 1;
            Ok(LilithValue::U8(val))
        }
        0x03 => {
            if *pos + 8 > data.len() {
                return Err("Unexpected EOF parsing Number (f64)".into());
            }
            let bytes: [u8; 8] = data[*pos..*pos + 8]
                .try_into()
                .map_err(|e| format!("Failed to read f64: {}", e))?;
            *pos += 8;
            let val = f64::from_be_bytes(bytes);
            Ok(LilithValue::Number(val))
        }
        0x04 => {
            if *pos + 4 > data.len() {
                return Err("Unexpected EOF parsing String length".into());
            }
            let len_bytes: [u8; 4] = data[*pos..*pos + 4]
                .try_into()
                .map_err(|e| format!("Failed to read str_len: {}", e))?;
            *pos += 4;
            let str_len = u32::from_le_bytes(len_bytes) as usize;
            if *pos + str_len > data.len() {
                return Err(format!(
                    "Unexpected EOF parsing String content of length {}",
                    str_len
                ));
            }
            let slice = &data[*pos..*pos + str_len];
            *pos += str_len;
            let val = match std::str::from_utf8(slice) {
                Ok(s) => s.to_string(),
                Err(_) => String::from_utf8_lossy(slice).into_owned(),
            };
            Ok(LilithValue::String(val))
        }
        0x05 => {
            let mut table = IndexMap::new();
            while *pos < data.len() {
                if data[*pos] == 0xFF {
                    *pos += 1;
                    break;
                }
                let old_pos = *pos;
                let key_val = parse_stream_internal(data, pos, depth + 1)?;
                let val_val = parse_stream_internal(data, pos, depth + 1)?;
                if *pos == old_pos {
                    return Err(format!("Parser stuck without progress at offset {}", *pos));
                }
                table.insert(key_val.to_key_string(), val_val);
            }
            Ok(LilithValue::Table(table))
        }
        0xFF => Ok(LilithValue::Null),
        unknown => Err(format!(
            "Unknown tag 0x{:02X} at offset {}",
            unknown,
            *pos - 1
        )),
    }
}

pub fn decode_mail_bytes(data: &[u8]) -> Result<LilithValue, String> {
    if data.len() < 10 || data[0] != 0xFF || data[9] != 0x05 {
        return Err("Invalid RoK mail binary header".into());
    }
    let mut pos = 9;
    parse_stream(data, &mut pos)
}

pub fn decode_mail_file<P: AsRef<Path>>(filepath: P) -> Result<LilithValue, String> {
    let path = filepath.as_ref();
    let buf = crate::vault::read_file_shared(path).map_err(|e| e.to_string())?;
    decode_mail_bytes(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_decode_mail_file_valid() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rok_watcher_valid_mail_{}_{}.bin",
            std::process::id(),
            unique
        ));
        let mut valid = vec![0; 10];
        valid[0] = 0xFF;
        valid[9] = 0x05;
        valid.push(0xFF); // End of table
        std::fs::write(&path, &valid).unwrap();

        let result = decode_mail_file(&path);
        let _ = std::fs::remove_file(&path);

        assert!(result.is_ok());
        if let Ok(LilithValue::Table(m)) = result {
            assert!(m.is_empty());
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn test_invalid_header() {
        let invalid = vec![0x00; 8];
        assert!(decode_mail_bytes(&invalid).is_err());
    }

    #[test]
    fn test_valid_header_empty_table() {
        let mut valid = vec![0; 10];
        valid[0] = 0xFF;
        valid[9] = 0x05;
        valid.push(0xFF); // End of table
        let parsed = decode_mail_bytes(&valid);
        assert!(parsed.is_ok());
        if let Ok(LilithValue::Table(m)) = parsed {
            assert!(m.is_empty());
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn test_decode_mail_file_rejects_oversized_file() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rok_watcher_oversized_mail_{}_{}.bin",
            std::process::id(),
            unique
        ));
        let file = File::create(&path).unwrap();
        file.set_len(MAX_MAIL_FILE_SIZE + 1).unwrap();
        drop(file);

        let result = decode_mail_file(&path);
        std::fs::remove_file(path).unwrap();

        assert!(result.unwrap_err().contains("exceeds max size limit"));
    }

    #[test]
    fn test_recursion_depth_limit() {
        // Deeply nested table tags (0x05) exceeding MAX_RECURSION_DEPTH
        let mut malformed = vec![0; 10];
        malformed[0] = 0xFF;
        malformed[9] = 0x05;
        malformed.extend(std::iter::repeat_n(0x05, 150));
        let result = decode_mail_bytes(&malformed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Exceeded maximum recursion depth"));
    }
}
