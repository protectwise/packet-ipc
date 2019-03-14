use bincode;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use serde_bytes::{ByteBuf, Bytes};

pub trait AsIpcPacket {
    fn timestamp(&self) -> &std::time::SystemTime;
    fn data(&self) -> &[u8];
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcPacket {
    intermediate: serde_bytes::ByteBuf,
}

impl IpcPacket {
    pub fn try_from<T: AsIpcPacket>(packet: &T) -> Result<IpcPacket, Error> {
        let vec = bincode::serialize(&BorrowedPacket {
            ts: packet.timestamp().clone(),
            data: Bytes::new(packet.data())
        }).map_err(Error::Bincode)?;
        Ok(IpcPacket {
            intermediate: ByteBuf::from(vec)
        })
    }

    pub fn new(bytes: serde_bytes::ByteBuf) -> IpcPacket {
        IpcPacket {
            intermediate: bytes
        }
    }

    pub fn into_packet(self) -> Result<Arc<Packet>, Error> {
        let borrowed: BorrowedPacket = bincode::deserialize(self.intermediate.as_ref()).map_err(Error::Bincode)?;
        Ok(Arc::new(Packet {
            ts: borrowed.ts,
            data: borrowed.data.to_vec()
        }))
    }
}

struct BorrowedPacket<'a> {
    ts: std::time::SystemTime,
    data: Bytes<'a>
}

#[derive(Debug)]
pub struct Packet {
    ts: std::time::SystemTime,
    data: Vec<u8>
}

impl Packet {
    pub fn new(ts: std::time::SystemTime, data: Vec<u8>) -> Packet {
        Packet {
            ts,
            data
        }
    }
}

impl AsIpcPacket for Packet {
    fn timestamp(&self) -> &std::time::SystemTime {
        &self.ts
    }
    fn data(&self) -> &[u8] {
        self.data.as_ref()
    }
}

mod ser {
    use serde::{Serialize, Serializer};
    use serde::ser::SerializeStruct;

    impl<'a> Serialize for super::BorrowedPacket<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            let mut state = serializer.serialize_struct("packet", 2)?;
            state.serialize_field("ts", &self.ts)?;
            state.serialize_field("data", &self.data)?;
            state.end()
        }
    }
}

mod de {
    use super::BorrowedPacket;

    use serde::{Deserialize, Deserializer};
    use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
    use serde_bytes::Bytes;

    enum Field {
        Timestamp,
        Data,
    }

    const FIELDS: &'static [&'static str] = &["ts", "data"];

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct FieldVisitor;

    impl<'de> Visitor<'de> for FieldVisitor {
        type Value = Field;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("`ts` or `data`")
        }

        fn visit_str<E>(self, value: &str) -> Result<Field, E>
            where
                E: DeError,
        {
            match value {
                "ts" => Ok(Field::Timestamp),
                "data" => Ok(Field::Data),
                _ => Err(DeError::unknown_field(value, FIELDS)),
            }
        }

        fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
            where
                E: DeError,
        {
            match value {
                b"ts" => Ok(Field::Timestamp),
                b"data" => Ok(Field::Data),
                _ => {
                    let value = String::from_utf8_lossy(value);
                    Err(DeError::unknown_field(&value, FIELDS))
                }
            }
        }
    }

    struct PacketVisitor;

    impl<'de> Visitor<'de> for PacketVisitor {
        type Value = BorrowedPacket<'de>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("struct SystemTime")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
        {
            let ts: std::time::SystemTime = seq.next_element()?.ok_or(<A::Error as DeError>::invalid_length(0, &self))?;
            let data: serde_bytes::Bytes<'de> = seq.next_element()?.ok_or(<A::Error as DeError>::invalid_length(1, &self))?;
            Ok(BorrowedPacket {
                ts,
                data
            })
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
        {
            let mut ts: Option<std::time::SystemTime> = None;
            let mut data: Option<Bytes> = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Timestamp => {
                        if ts.is_some() {
                            return Err(<A::Error as DeError>::duplicate_field("ts"));
                        }
                        ts = Some(map.next_value()?);
                    }
                    Field::Data => {
                        if data.is_some() {
                            return Err(<A::Error as DeError>::duplicate_field("data"));
                        }
                        let bytes: serde_bytes::Bytes<'de> = map.next_value()?;
                        data = Some(bytes);
                    }
                }
            }
            let ts = ts.ok_or(<A::Error as DeError>::missing_field("ts"))?;
            let data = data.ok_or(<A::Error as DeError>::missing_field("data"))?;

            Ok(BorrowedPacket {
                ts: ts,
                data: data
            })
        }
    }

    impl<'de> Deserialize<'de> for BorrowedPacket<'de> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
        {
            deserializer.deserialize_struct("p", FIELDS, PacketVisitor)
        }
    }
}