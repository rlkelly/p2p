use bytes::BytesMut;
use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};


pub fn get_nstring(src: &mut BytesMut, n: usize) -> Option<String> {
    if n == 0 {
        return None;
    };
    let target = src.split_to(n);
    if target.len() == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&target).trim_matches(char::from(0)).to_string())
}

pub fn take_u64(src: &mut BytesMut) -> Result<u64, MessageCodecError> {
    if src.len() < 8 {
        return Err(MessageCodecError::SerializationError)
    }
    let mut buf: &[u8] = &src.split_to(8)[..];
    Ok(buf.read_u64::<BigEndian>().unwrap())
}

#[allow(dead_code)]
pub(crate) fn take_u32(src: &mut BytesMut) -> Result<u32, MessageCodecError> {
    if src.len() < 4 {
        return Err(MessageCodecError::SerializationError)
    }
    let mut buf: &[u8] = &src.split_to(4)[..];
    Ok(buf.read_u32::<BigEndian>().unwrap())
}

#[allow(dead_code)]
pub(crate) fn take_u16(src: &mut BytesMut) -> Result<u16, MessageCodecError> {
    if src.len() < 2 {
        return Err(MessageCodecError::SerializationError)
    }
    let mut buf: &[u8] = &src.split_to(2)[..];
    Ok(buf.read_u16::<BigEndian>().unwrap())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageCodecError {
    SerializationError,
    DataLengthMismatch,
    IO,
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BytesMut, BufMut};

    #[test]
    fn test_bytes() {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u8(0);
        buf.put_u16(0);
        buf.put_u32(0);
        buf.put_u8(1);
        assert_eq!(take_u32(&mut buf).unwrap(), 0);
        assert_eq!(take_u32(&mut buf).unwrap(), 1);
    }
}
