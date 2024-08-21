// Copyright ⓒ 2024 Peter Morgan <peter.james.morgan@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{Error, Result, RootMessageMeta};
use serde::{
    de::{DeserializeSeed, EnumAccess, SeqAccess, VariantAccess, Visitor},
    Deserializer,
};
use std::{
    any::{type_name, type_name_of_val},
    collections::VecDeque,
    fmt,
    io::{self, Read},
    str::from_utf8,
};
use tansu_kafka_model::{FieldMeta, MessageMeta};
use tracing::{debug, warn};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Kind {
    Request,
    Response,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Container {
    Struct {
        name: &'static str,
        fields: &'static [&'static str],
    },

    Enum {
        name: &'static str,
        variants: &'static [&'static str],
    },
}

impl Container {
    fn name(&self) -> &'static str {
        match self {
            Self::Struct { name, .. } | Self::Enum { name, .. } => name,
        }
    }
}

struct ReadPosition<'a> {
    reader: &'a mut dyn Read,
    position: u64,
}

impl<'a> ReadPosition<'a> {
    fn new(reader: &'a mut dyn Read) -> Self {
        Self {
            reader,
            position: 0,
        }
    }
}

impl<'a> Read for ReadPosition<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf).map(|count| {
            self.position += count as u64;
            count
        })
    }
}

pub struct Decoder<'de> {
    reader: ReadPosition<'de>,
    containers: VecDeque<Container>,
    field: Option<&'static str>,
    kind: Option<Kind>,
    api_key: Option<i16>,
    api_version: Option<i16>,
    meta: Meta,
    length: Option<usize>,
    in_seq_of_primitive: bool,
    path: VecDeque<&'static str>,
    in_records: bool,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct FieldLookup(&'static [(&'static str, &'static FieldMeta)]);

impl From<&'static [(&'static str, &'static FieldMeta)]> for FieldLookup {
    fn from(value: &'static [(&'static str, &'static FieldMeta)]) -> Self {
        Self(value)
    }
}

impl FieldLookup {
    #[must_use]
    pub(crate) fn field(&self, name: &str) -> Option<&'static FieldMeta> {
        self.0
            .iter()
            .find(|(found, _)| name == *found)
            .map(|(_, meta)| *meta)
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Meta {
    message: Option<&'static MessageMeta>,
    field: Option<&'static FieldMeta>,
    parse: VecDeque<FieldLookup>,
}

impl<'de> Decoder<'de> {
    pub fn new(reader: &'de mut dyn Read) -> Self {
        Self {
            reader: ReadPosition::new(reader),
            containers: VecDeque::new(),
            field: None,
            kind: None,
            api_key: None,
            api_version: None,
            meta: Meta::default(),
            length: None,
            in_seq_of_primitive: false,
            path: VecDeque::new(),
            in_records: false,
        }
    }

    pub fn request(reader: &'de mut dyn Read) -> Self {
        Self {
            reader: ReadPosition::new(reader),
            containers: VecDeque::new(),
            field: None,
            kind: Some(Kind::Request),
            api_key: None,
            api_version: None,
            meta: Meta::default(),
            length: None,
            in_seq_of_primitive: false,
            path: VecDeque::new(),
            in_records: false,
        }
    }

    pub fn response(reader: &'de mut dyn Read, api_key: i16, api_version: i16) -> Self {
        Self {
            reader: ReadPosition::new(reader),
            containers: VecDeque::new(),
            field: None,
            kind: Some(Kind::Response),
            api_key: Some(api_key),
            api_version: Some(api_version),
            meta: RootMessageMeta::messages()
                .responses()
                .get(&api_key)
                .map_or_else(Meta::default, |meta| {
                    let mut parse = VecDeque::new();
                    parse.push_front(meta.fields.into());

                    Meta {
                        message: Some(*meta),
                        parse,
                        ..Default::default()
                    }
                }),
            length: None,
            in_seq_of_primitive: false,
            path: VecDeque::new(),
            in_records: false,
        }
    }

    #[must_use]
    pub fn field_name(&self) -> String {
        self.path.iter().fold(String::new(), |acc, step| {
            if acc.is_empty() {
                (*step).to_string()
            } else {
                format!("{step}.{acc}")
            }
        })
    }

    fn in_header(&self) -> bool {
        self.containers
            .front()
            .is_some_and(|c| c.name() == "HeaderMezzanine")
    }

    #[must_use]
    fn is_flexible(&self) -> bool {
        if self.in_header()
            && ((self.kind.is_some_and(|kind| kind == Kind::Request)
                && self.field.is_some_and(|field| field == "client_id"))
                || (self.kind.is_some_and(|kind| kind == Kind::Response)
                    && self.api_key.is_some_and(|api_key| api_key == 18)))
        {
            false
        } else {
            self.meta.message.is_some_and(|meta| {
                self.api_version
                    .map_or(false, |api_version| meta.is_flexible(api_version))
            })
        }
    }

    #[must_use]
    fn is_valid(&self) -> bool {
        self.api_version.map_or(true, |api_version| {
            self.meta
                .field
                .map_or(true, |field| field.version.within(api_version))
        })
    }

    fn is_nullable(&self) -> bool {
        self.api_version.is_some_and(|api_version| {
            self.meta
                .field
                .is_some_and(|field| field.is_nullable(api_version))
        })
    }

    #[must_use]
    fn is_sequence(&self) -> bool {
        self.meta
            .field
            .is_some_and(|field| field.kind.is_sequence())
    }

    fn is_records(&self) -> bool {
        self.meta.field.is_some_and(|field| field.kind.is_records())
    }

    #[must_use]
    fn is_string(&self) -> bool {
        self.in_header() && self.field.is_some_and(|field| field == "client_id")
            || self.meta.field.is_some_and(|field| field.kind.is_string())
    }

    fn read_mandatory_non_nullable_length(&mut self) -> Result<()> {
        debug!(
            "header mezzanine: {}, nullable: {}, valid: {}",
            self.in_header(),
            self.is_nullable(),
            self.is_valid(),
        );

        if self.in_header() || self.is_nullable() || !self.is_valid() {
            debug!(
                "field: {} is not a mandatory non nullable length",
                self.field_name()
            );
            return Ok(());
        }

        debug!(
            "read_non_nullable_length, field: {}, flexible: {}, string: {}",
            self.field_name(),
            self.is_flexible(),
            self.is_string(),
        );

        if self.is_flexible() {
            let length = self.unsigned_varint()?;
            debug!("length: {length}");
            self.length = Some((length - 1).try_into()?);
        } else if self.is_string()
            || (self.in_seq_of_primitive
                && self.meta.field.is_some_and(|field| {
                    field
                        .kind
                        .kind_of_sequence()
                        .is_some_and(|sk| sk.is_string())
                }))
        {
            let mut buf = [0u8; 2];
            self.reader.read_exact(&mut buf)?;

            let length = i16::from_be_bytes(buf);
            debug!("length: {length}");
            self.length = Some(length.try_into()?);
        } else {
            let mut buf = [0u8; 4];
            self.reader.read_exact(&mut buf)?;

            let length = i32::from_be_bytes(buf);
            debug!("length: {length}");
            self.length = Some(length.try_into()?);
        }

        Ok(())
    }

    fn unsigned_varint(&mut self) -> Result<u32> {
        const CONTINUATION: u8 = 0b1000_0000;
        const MASK: u8 = 0b0111_1111;
        let mut shift = 0u8;
        let mut accumulator = 0u32;
        let mut done = false;

        let mut buf = [0u8; 1];

        while !done {
            self.reader.read_exact(&mut buf)?;

            if buf[0] & CONTINUATION == CONTINUATION {
                let intermediate = u32::from(buf[0] & MASK);
                accumulator += intermediate << shift;
                shift += 7;
            } else {
                accumulator += u32::from(buf[0]) << shift;
                done = true;
            }
        }

        Ok(accumulator)
    }

    pub fn position(&self) -> u64 {
        self.reader.position
    }
}

impl<'de> fmt::Debug for Decoder<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut Decoder<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let _ = visitor;
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8];
        self.reader.read_exact(&mut buf)?;
        let v = buf[0] != 0;

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_bool(v)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8];
        self.reader.read_exact(&mut buf)?;
        let v = i8::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        let v = i16::from_be_bytes(buf);

        match (self.containers.front(), self.field) {
            (
                Some(Container::Struct {
                    name: "HeaderMezzanine",
                    ..
                }),
                Some("api_key"),
            ) => {
                _ = self.api_key.replace(v);

                if let Some(meta) = RootMessageMeta::messages().requests().get(&v) {
                    self.meta.message = Some(*meta);
                    self.meta.parse.push_front(meta.fields.into());
                }
            }

            (
                Some(Container::Struct {
                    name: "HeaderMezzanine",
                    ..
                }),
                Some("api_version"),
            ) => {
                _ = self.api_version.replace(v);
            }

            _ => (),
        }

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let v = i32::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        let v = i64::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8];
        self.reader.read_exact(&mut buf)?;
        let v = u8::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        let v = u16::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let v = u32::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        let v = u64::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let v = f32::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        let v = f64::from_be_bytes(buf);

        debug!(
            "field: {}, value: {v}:{}",
            self.field_name(),
            type_name::<V::Value>(),
        );
        visitor.visit_f64(v)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!("struct: {:?}, field: {}", self.containers.front(), field);
        }
        let _ = visitor;
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.read_mandatory_non_nullable_length()?;

        self.length
            .ok_or(Error::StringWithoutLength)
            .and_then(|length| {
                let mut buf = vec![0u8; length];
                self.reader.read_exact(&mut buf)?;
                from_utf8(buf.as_slice())
                    .map_err(Into::into)
                    .inspect(|v| debug!("visitor: {}, v: {v}", type_name_of_val(&visitor)))
                    .and_then(|s| visitor.visit_str(s))
            })
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "deserialize_string, field: {}, nullable: {}",
            self.field_name(),
            self.meta.field.map_or(false, |field| self
                .api_version
                .map_or(false, |api_version| field.is_nullable(api_version)))
        );

        if self.length.is_none() {
            self.read_mandatory_non_nullable_length()?;
        }

        if let Some(length) = self.length.take() {
            let mut buf = vec![0u8; length];
            self.reader.read_exact(&mut buf)?;

            String::from_utf8(buf)
                .map_err(Into::into)
                .inspect(|v| {
                    debug!(r#"field: {}, value: "{v}""#, self.field_name(),);
                })
                .and_then(|s| visitor.visit_string(s))
        } else {
            Err(Error::StringWithoutLength)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!("struct: {:?}, field: {}", self.containers.front(), field);
        }

        let length = if self.is_flexible() {
            self.unsigned_varint()
                .and_then(|length| usize::try_from(length - 1).map_err(Into::into))?
        } else {
            let mut buf = [0u8; 4];

            self.reader.read_exact(&mut buf)?;
            usize::try_from(u32::from_be_bytes(buf))?
        };

        let mut buf = vec![0u8; length];
        self.reader.read_exact(&mut buf)?;
        visitor.visit_bytes(&buf[..])
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!("struct: {:?}, field: {}", self.containers.front(), field);
        }

        let length = if self.is_flexible() {
            self.unsigned_varint()
                .and_then(|length| usize::try_from(length - 1).map_err(Into::into))?
        } else {
            let mut buf = [0u8; 4];

            self.reader.read_exact(&mut buf)?;
            usize::try_from(u32::from_be_bytes(buf))?
        };

        let mut buf = vec![0u8; length];
        self.reader.read_exact(&mut buf)?;
        visitor.visit_bytes(&buf[..])
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "deserialize_option, field: {}, flexible: {}, valid: {}, string: {}, sequence: {}",
            self.field_name(),
            self.is_flexible(),
            self.is_valid(),
            self.is_string(),
            self.is_sequence(),
        );

        if self.is_valid() {
            if self.field.is_some_and(|field| field == "tag_buffer") {
                if self.is_flexible() {
                    self.length = None;
                    visitor.visit_some(self)
                } else {
                    visitor.visit_none()
                }
            } else if self.is_records() {
                let length = if self.is_flexible() {
                    self.unsigned_varint().map(|length| length - 1)?
                } else {
                    let mut buf = [0u8; 4];
                    self.reader.read_exact(&mut buf)?;

                    u32::from_be_bytes(buf)
                };

                debug!(?length);

                if length == 0 {
                    visitor.visit_none()
                } else {
                    self.length = Some(length.try_into()?);
                    self.in_records = true;
                    visitor.visit_some(self)
                }
            } else if self.is_sequence() {
                if self.is_flexible() {
                    self.unsigned_varint().and_then(|length| {
                        if length == 0 {
                            self.length = None;
                            visitor.visit_none()
                        } else {
                            self.length = Some((length - 1).try_into()?);
                            visitor.visit_some(self)
                        }
                    })
                } else {
                    let mut buf = [0u8; 4];
                    self.reader.read_exact(&mut buf)?;

                    let length = i32::from_be_bytes(buf);
                    debug!("length: {length}");

                    if length == -1 {
                        self.length = None;
                        visitor.visit_none()
                    } else {
                        self.length = Some(length.try_into()?);
                        visitor.visit_some(self)
                    }
                }
            } else if self.is_string() {
                if self.is_flexible() {
                    self.unsigned_varint().and_then(|length| {
                        if length == 0 {
                            self.length = None;
                            visitor.visit_none()
                        } else {
                            self.length = Some((length - 1).try_into()?);
                            visitor.visit_some(self)
                        }
                    })
                } else {
                    let mut buf = [0u8; 2];
                    self.reader.read_exact(&mut buf)?;

                    let length = i16::from_be_bytes(buf);
                    debug!("length: {length}");

                    if length == -1 {
                        self.length = None;
                        visitor.visit_none()
                    } else {
                        self.length = Some(length.try_into()?);
                        visitor.visit_some(self)
                    }
                }
            } else {
                self.length = None;
                visitor.visit_some(self)
            }
        } else {
            self.length = None;
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "visitor: {}, type name: {}",
            type_name_of_val(&visitor),
            type_name::<V::Value>(),
        );
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!("struct: {:?}, field: {}", self.containers.front(), field);
        }

        debug!("name: {name}, visitor: {}", type_name_of_val(&visitor));
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!("struct: {:?}, field: {}", self.containers.front(), field);
        }

        debug!("name: {name}, visitor: {}", type_name_of_val(&visitor));
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "seq, type name: {}, length: {:?}, meta.field: {}, of primitive: {}, records: {}",
            type_name::<V::Value>(),
            self.length,
            self.meta.field.is_some(),
            self.meta.field.is_some_and(|field| field
                .kind
                .kind_of_sequence()
                .is_some_and(|seq| seq.is_primitive())),
            self.is_records(),
        );

        self.in_seq_of_primitive = self.meta.field.is_some_and(|field| {
            field
                .kind
                .kind_of_sequence()
                .is_some_and(|seq| seq.is_primitive())
        });

        match self.length.take() {
            Some(size_in_bytes) if self.in_records => {
                let outcome = visitor.visit_seq(Batch::new(self, size_in_bytes as u64));
                self.in_seq_of_primitive = false;
                self.in_records = false;
                outcome
            }

            otherwise => {
                let outcome = visitor.visit_seq(Seq::new(self, otherwise));
                self.in_seq_of_primitive = false;
                outcome
            }
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(field) = self.field {
            debug!(
                "tuple, struct: {:?}, field: {}",
                self.containers.front(),
                field
            );
        }

        debug!("len: {len}, visitor: {}", type_name_of_val(&visitor));
        visitor.visit_seq(Seq::new(self, Some(len)))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "name: {name}, len: {len}, visitor: {}",
            type_name_of_val(&visitor)
        );
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!("visitor: {}", type_name_of_val(&visitor));
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!("deserialize_struct, name: {name}, fields: {:?}", fields,);

        self.containers
            .push_front(Container::Struct { name, fields });

        let outcome = if let Some(mm) = self.meta.message {
            if let Some(fm) = mm.structures().get(name) {
                debug!("deserialize_struct, name: {name}");

                _ = self.meta.field.replace(*fm);
                self.meta.parse.push_front(fm.fields.into());
                let outcome = visitor.visit_seq(Struct::new(self, name, fields));
                _ = self.meta.parse.pop_front();
                _ = self.meta.field.take();
                outcome
            } else if name == "Body" {
                self.meta.parse.push_front(mm.fields.into());
                let outcome = visitor.visit_seq(Struct::new(self, name, fields));
                _ = self.meta.parse.pop_front();
                outcome
            } else {
                if !["Frame", "HeaderMezzanine"].contains(&name) {
                    warn!("deserialize_struct, no field meta for struct, name: {name}");
                }
                _ = self.meta.field.take();
                self.meta.parse.push_front(FieldLookup(&[]));
                let outcome = visitor.visit_seq(Struct::new(self, name, fields));
                _ = self.meta.parse.pop_front();
                outcome
            }
        } else {
            if !["Frame", "HeaderMezzanine"].contains(&name) {
                warn!("deserialize_struct, no message meta for struct, name: {name}");
            }

            visitor.visit_seq(Struct::new(self, name, fields))
        };

        _ = self.containers.pop_front();

        outcome
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!("enum, name: {name}",);

        self.containers
            .push_front(Container::Enum { name, variants });

        let outcome = visitor.visit_enum(Enum::new(self, name));

        _ = self.containers.pop_front();
        outcome
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!(
            "deserialize_identifier, front: {:?}, meta.message.name: {:?}",
            self.containers.front().map(Container::name),
            self.meta.message.map(|message| message.name)
        );

        visitor.visit_str(match (self.containers.front(), self.meta.message) {
            (
                Some(Container::Enum {
                    name: "HeaderMezzanine",
                    ..
                }),
                _,
            ) if self.kind.is_some_and(|kind| kind == Kind::Request) => "Request",

            (
                Some(Container::Enum {
                    name: "HeaderMezzanine",
                    ..
                }),
                _,
            ) if self.kind.is_some_and(|kind| kind == Kind::Response) => "Response",

            (Some(Container::Enum { name: "Body", .. }), Some(meta)) => meta.name,

            container => todo!("container: {:?}", container),
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug!("visitor: {}", type_name_of_val(&visitor));
        todo!()
    }
}

struct Batch<'de, 'a> {
    de: &'a mut Decoder<'de>,
    remaining: u64,
}

impl<'de, 'a> Batch<'de, 'a> {
    fn new(de: &'a mut Decoder<'de>, remaining: u64) -> Self {
        Self { de, remaining }
    }
}

impl<'de, 'a> SeqAccess<'de> for Batch<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        debug!(?self.remaining);
        if self.remaining > 0 {
            let start = self.de.reader.position;
            let outcome = seed.deserialize(&mut *self.de).map(Some);
            let delta = self.de.reader.position - start;
            debug!(?delta);
            self.remaining -= delta;
            outcome
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct Seq<'de, 'a> {
    de: &'a mut Decoder<'de>,
    length: Option<usize>,
}

impl<'de, 'a> Seq<'de, 'a> {
    fn new(de: &'a mut Decoder<'de>, length: Option<usize>) -> Self {
        Self { de, length }
    }
}

impl<'de, 'a> SeqAccess<'de> for Seq<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        debug!(
            "seq, next seed: {}, length: {:?}",
            type_name_of_val(&seed),
            self.length
        );

        match self.length {
            Some(0) => Ok(None),

            Some(length) => {
                _ = self.length.replace(length - 1);
                seed.deserialize(&mut *self.de).map(Some)
            }

            None => seed.deserialize(&mut *self.de).map(Some),
        }
    }
}

#[derive(Debug)]
struct Struct<'de, 'a> {
    de: &'a mut Decoder<'de>,
    name: &'static str,
    fields: &'static [&'static str],
    index: usize,
}

impl<'de, 'a> Struct<'de, 'a> {
    fn new(de: &'a mut Decoder<'de>, name: &'static str, fields: &'static [&'static str]) -> Self {
        Self {
            de,
            name,
            fields,
            index: 0,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for Struct<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let field = self.fields[self.index];
        self.de.field = Some(field);
        self.index += 1;

        if let Some(fl) = self.de.meta.parse.front() {
            self.de.meta.field = fl.field(field);
        }

        debug!(
            "struct name: {} field: {}, flexible: {}, seed type name: {}, meta.field: {}, \
             records: {}",
            self.name,
            field,
            self.de.is_flexible(),
            type_name::<T::Value>(),
            self.de.meta.field.is_some(),
            self.de.is_records(),
        );

        self.de.path.push_front(field);
        let outcome = seed.deserialize(&mut *self.de).map(Some);
        _ = self.de.path.pop_front();
        outcome
    }
}

#[derive(Debug)]
struct Enum<'de, 'a> {
    de: &'a mut Decoder<'de>,
    name: &'static str,
}

impl<'de, 'a> Enum<'de, 'a> {
    fn new(de: &'a mut Decoder<'de>, name: &'static str) -> Self {
        Self { de, name }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::Message(String::from("expecting string")))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        debug!("seed: {}", type_name_of_val(&seed));
        todo!()
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        debug!("len: {len}, visitor: {}", type_name_of_val(&visitor));
        todo!()
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        debug!("fields: {fields:?}",);

        Deserializer::deserialize_struct(self.de, self.name, fields, visitor)
    }
}