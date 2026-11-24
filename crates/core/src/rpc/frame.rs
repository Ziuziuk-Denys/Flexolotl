use super::RpcError;
use serde_json::Value;
use std::io::{Read, Write};

const HEADER_LENGTH: usize = 8;
const MAX_BODY_LENGTH: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl TryFrom<u32> for Opcode {
    type Error = RpcError;

    fn try_from(code: u32) -> Result<Self, RpcError> {
        match code {
            0 => Ok(Self::Handshake),
            1 => Ok(Self::Frame),
            2 => Ok(Self::Close),
            3 => Ok(Self::Ping),
            4 => Ok(Self::Pong),
            _ => Err(RpcError::Protocol("unknown opcode")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub opcode: Opcode,
    pub body: Value,
}

impl Frame {
    pub fn new(opcode: Opcode, body: Value) -> Self {
        Self { opcode, body }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), RpcError> {
        let body = self.body.to_string();
        let mut bytes = Vec::with_capacity(HEADER_LENGTH + body.len());
        bytes.extend((self.opcode as u32).to_le_bytes());
        bytes.extend((body.len() as u32).to_le_bytes());
        bytes.extend(body.as_bytes());
        writer.write_all(&bytes)?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> Result<Self, RpcError> {
        let opcode = Opcode::try_from(read_u32(reader)?)?;
        let length = read_u32(reader)? as usize;
        if length > MAX_BODY_LENGTH {
            return Err(RpcError::Protocol("frame is too large"));
        }
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes)?;
        let body =
            serde_json::from_slice(&bytes).map_err(|_| RpcError::Protocol("frame is not JSON"))?;
        Ok(Self { opcode, body })
    }
}

fn read_u32(reader: &mut impl Read) -> Result<u32, RpcError> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}
