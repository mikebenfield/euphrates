// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use rlua;
use rlua::{FromLua, ToLua};
use serde;
use serde::ser;
use serde::de;

pub mod errors {
    use rlua;

    error_chain! {

        foreign_links {
            Lua(rlua::Error);
        }

        errors {
            Custom(s: String) {
                description("Custom error")
                display("Custom error: {}", s)
            }

            DeserializeType(s: String) {
                description("Error deserializing type")
                display("Error deserializing type: {}", s)
            }
        }
    }
}

pub use self::errors::*;

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::from(ErrorKind::Custom(format!("{}", msg)))
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::from(ErrorKind::Custom(format!("{}", msg)))
    }
}

pub struct Serializer<'lua> {
    lua: &'lua rlua::Lua,
}

impl<'lua> Serializer<'lua> {
    pub fn new(lua: &'lua rlua::Lua) -> Serializer<'lua> {
        Serializer { lua: lua }
    }
}

pub struct SerializeSeq<'lua> {
    lua: &'lua rlua::Lua,

    /// if the value ultimately to be returned is not the same
    /// as `table`, it will be the table in `result`, and `table`
    /// will be put into `result` at `key`
    result: Option<rlua::Table<'lua>>,
    key: rlua::Value<'lua>,
    table: rlua::Table<'lua>,

    /// array index to insert next
    index: usize,
}

macro_rules! impl_serialize_seq {
    ($class: path, $method: ident) => {
        impl<'lua> $class for SerializeSeq<'lua> {
            type Ok = rlua::Value<'lua>;
            type Error = Error;

            fn $method<T: ?Sized>(
                &mut self,
                value: &T,
            ) -> Result<()>
            where
                T: ser::Serialize
            {
                let t = value.serialize(Serializer::new(self.lua))?;
                self.table.raw_set(self.index, t)?;
                self.index += 1;
                Ok(())
            }

            fn end(self) -> Result<rlua::Value<'lua>> {
                match self.result {
                    None => Ok(rlua::Value::Table(self.table)),
                    Some(x) => {
                        x.raw_set(self.key, self.table)?;
                        Ok(rlua::Value::Table(x))
                    }
                }
            }
        }
    }
}

impl_serialize_seq!{ser::SerializeSeq, serialize_element}
impl_serialize_seq!{ser::SerializeTuple, serialize_element}
impl_serialize_seq!{ser::SerializeTupleStruct, serialize_field}
impl_serialize_seq!{ser::SerializeTupleVariant, serialize_field}

pub struct SerializeStruct<'lua> {
    lua: &'lua rlua::Lua,
    last_key: rlua::Value<'lua>,
    result: Option<rlua::Table<'lua>>,
    key: rlua::Value<'lua>,
    table: rlua::Table<'lua>,
}

macro_rules! impl_serialize_struct {
    ($class: path) => {
        impl<'lua> $class for SerializeStruct<'lua> {
            type Ok = rlua::Value<'lua>;
            type Error = Error;

            fn serialize_field<T: ?Sized> (
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<()>
            where
                T: ser::Serialize
            {
                let t = value.serialize(Serializer::new(self.lua))?;
                self.table.raw_set(key, t)?;
                Ok(())
            }

            fn end(self) -> Result<rlua::Value<'lua>> {
                match self.result {
                    None => Ok(rlua::Value::Table(self.table)),
                    Some(x) => {
                        x.raw_set(self.key, self.table)?;
                        Ok(rlua::Value::Table(x))
                    }
                }
            }
        }
    }
}

impl_serialize_struct!{ser::SerializeStruct}
impl_serialize_struct!{ser::SerializeStructVariant}

impl<'lua> ser::SerializeMap for SerializeStruct<'lua> {
    type Ok = rlua::Value<'lua>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.last_key = key.serialize(Serializer::new(self.lua))?;
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        let t = value.serialize(Serializer::new(self.lua))?;
        let mut key = rlua::Value::Nil;
        std::mem::swap(&mut key, &mut self.last_key);
        self.table.raw_set(key, t)?;
        Ok(())
    }

    fn end(self) -> Result<rlua::Value<'lua>> {
        match self.result {
            None => Ok(rlua::Value::Table(self.table)),
            Some(x) => {
                x.raw_set(self.key, self.table)?;
                Ok(rlua::Value::Table(x))
            }
        }
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ser::Serialize,
        V: ser::Serialize,
    {
        let k = key.serialize(Serializer::new(self.lua))?;
        let v = value.serialize(Serializer::new(self.lua))?;
        self.table.raw_set(k, v)?;
        Ok(())
    }
}

macro_rules! serialize_method {
    ($method_name: ident, $type_name: ident) => {
        fn $method_name(self, v: $type_name) -> Result<rlua::Value<'lua>> {
            Ok(
                <$type_name as rlua::ToLua>::to_lua(v, self.lua)?
            )
        }
    };
    ($method_name: ident, &$type_name: ident) => {
        fn $method_name(self, v: &$type_name) -> Result<rlua::Value<'lua>> {
            Ok(
                <&$type_name as rlua::ToLua>::to_lua(v, self.lua)?
            )
        }
    };
}

impl<'lua> serde::Serializer for Serializer<'lua> {
    type Ok = rlua::Value<'lua>;
    type Error = Error;

    type SerializeSeq = SerializeSeq<'lua>;
    type SerializeTuple = SerializeSeq<'lua>;
    type SerializeTupleStruct = SerializeSeq<'lua>;
    type SerializeTupleVariant = SerializeSeq<'lua>;
    type SerializeMap = SerializeStruct<'lua>;
    type SerializeStruct = SerializeStruct<'lua>;
    type SerializeStructVariant = SerializeStruct<'lua>;

    serialize_method!{serialize_bool, bool}
    serialize_method!{serialize_i8, i8}
    serialize_method!{serialize_i16, i16}
    serialize_method!{serialize_i32, i32}
    serialize_method!{serialize_i64, i64}
    serialize_method!{serialize_u8, u8}
    serialize_method!{serialize_u16, u16}
    serialize_method!{serialize_u32, u32}
    serialize_method!{serialize_u64, u64}
    serialize_method!{serialize_f32, f32}
    serialize_method!{serialize_f64, f64}
    serialize_method!{serialize_str, &str}

    fn serialize_char(self, v: char) -> Result<rlua::Value<'lua>> {
        self.serialize_str(v.to_string().as_ref())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<rlua::Value<'lua>> {
        let result = self.lua.create_table();
        for i in 0..v.len() {
            result.raw_set(i + 1, v[i])?;
        }
        Ok(rlua::Value::Table(result))
    }

    fn serialize_none(self) -> Result<rlua::Value<'lua>> {
        Ok("None".to_lua(self.lua)?)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<rlua::Value<'lua>>
    where
        T: ser::Serialize,
    {
        let result = self.lua.create_table();
        let t = value.serialize(self)?;
        result.raw_set(1, "Some")?;
        result.raw_set(2, t)?;
        Ok(rlua::Value::Table(result))
    }

    fn serialize_unit(self) -> Result<rlua::Value<'lua>> {
        Ok(rlua::Value::Nil)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<rlua::Value<'lua>> {
        self.serialize_str(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<rlua::Value<'lua>> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<rlua::Value<'lua>>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<rlua::Value<'lua>>
    where
        T: serde::Serialize,
    {
        let result = self.lua.create_table();
        let t = value.serialize(self)?;
        result.raw_set(1, variant)?;
        result.raw_set(2, t)?;
        Ok(rlua::Value::Table(result))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<SerializeSeq<'lua>> {
        let result_table = self.lua.create_table();
        Ok(SerializeSeq {
            lua: self.lua,
            result: None,
            key: rlua::Value::Nil,
            table: result_table,
            index: 1,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeSeq<'lua>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<SerializeSeq<'lua>> {
        let seq_table = self.lua.create_table();
        Ok(SerializeSeq {
            lua: self.lua,
            result: None,
            key: rlua::Value::Nil,
            table: seq_table,
            index: 1,
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeSeq<'lua>> {
        let result_table = self.lua.create_table();
        let seq_table = self.lua.create_table();
        result_table.raw_set(1, variant)?;
        Ok(SerializeSeq {
            lua: self.lua,
            result: Some(result_table),
            key: 2u64.to_lua(self.lua)?,
            table: seq_table,
            index: 1,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeStruct<'lua>> {
        let result = self.lua.create_table();
        Ok(SerializeStruct {
            lua: self.lua,
            last_key: rlua::Value::Nil,
            result: None,
            key: rlua::Value::Nil,
            table: result,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<SerializeStruct<'lua>> {
        let struct_table = self.lua.create_table();
        Ok(SerializeStruct {
            lua: self.lua,
            last_key: rlua::Value::Nil,
            result: None,
            key: rlua::Value::Nil,
            table: struct_table,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeStruct<'lua>> {
        let result = self.lua.create_table();
        let struct_table = self.lua.create_table();
        result.raw_set(1, variant)?;
        Ok(SerializeStruct {
            lua: self.lua,
            last_key: rlua::Value::Nil,
            result: Some(result),
            key: 2u64.to_lua(self.lua)?,
            table: struct_table,
        })
    }
}

pub struct Deserializer<'lua> {
    value: rlua::Value<'lua>,
    lua: &'lua rlua::Lua,
}

macro_rules! impl_deserialize {
    ($method_name: ident, $visitor_method_name: ident, $type_name: ident) => {
        fn $method_name<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>
        {
            visitor.$visitor_method_name($type_name::from_lua(self.value, self.lua)?)
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    impl_deserialize!{deserialize_bool, visit_bool, bool}
    impl_deserialize!{deserialize_i8, visit_i8, i8}
    impl_deserialize!{deserialize_i16, visit_i16, i16}
    impl_deserialize!{deserialize_i32, visit_i32, i32}
    impl_deserialize!{deserialize_i64, visit_i64, i64}
    impl_deserialize!{deserialize_u8, visit_u8, u8}
    impl_deserialize!{deserialize_u16, visit_u16, u16}
    impl_deserialize!{deserialize_u32, visit_u32, u32}
    impl_deserialize!{deserialize_u64, visit_u64, u64}
    impl_deserialize!{deserialize_f32, visit_f32, f32}
    impl_deserialize!{deserialize_f64, visit_f64, f64}

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = String::from_lua(self.value, self.lua)?;
        if s.len() == 1 {
            let c = s.chars().next().unwrap();
            visitor.visit_char(c)
        } else {
            bail!(
                ErrorKind::DeserializeType(
                    format!("Attempted to deserialize string {} as char", s)
                )
            )
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let rlua::Value::String(ref s) = self.value {
            visitor.visit_str(s.to_str()?)
        } else {
            bail!(
                ErrorKind::DeserializeType(
                    format!("Attempted to deserialize str using {:?}", self.value)
                )
            )
        }
    }

    impl_deserialize!{deserialize_string, visit_string, String}

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let v: Vec<u8> = Vec::from_lua(self.value, self.lua)?;
        visitor.visit_byte_buf(v)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        static ARRAY: &'static [&'static str] = &["None", "Some"];
        self.deserialize_enum(
            "Option",
            ARRAY,
            visitor
        )
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let rlua::Value::Nil = self.value {
            visitor.visit_unit()
        } else {
            bail!(
                ErrorKind::DeserializeType(
                    format!("Attempted to deserialize () using {:?}", self.value)
                )
            )
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let rlua::Value::String(ref s) = self.value {
            if s.as_bytes() == name.as_bytes() {
                return visitor.visit_unit();
            }
        }
        bail!(
            ErrorKind::DeserializeType(
                format!(
                    "Attempted to deserialize unit struct {} using {:?}",
                    name,
                    self.value
                )
            )
        )
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        if let rlua::Value::Table(t) = self.value {
            Ok(
                visitor.visit_seq(
                    SeqAccess {
                        lua: self.lua,
                        index: 1,
                        table: t,
                    }
                )?
            )
        } else {
            bail! (
                ErrorKind::DeserializeType(
                    format!(
                        "Attempted to deserialize array using {:?}",
                        self.value
                    )
                )
            )
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        if let rlua::Value::Table(t) = self.value {
            Ok(
                visitor.visit_map(
                    MapAccess {
                        lua: self.lua,
                        last_value: rlua::Value::Nil,
                        iterator: t.pairs(),
                    }
                )?
            )
        } else {
            bail!(
                ErrorKind::DeserializeType(
                    format!("tried to deserialize {:?} as map", self.value)
                )
            )
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        match self.value {
            rlua::Value::Nil => visitor.visit_unit(),
            rlua::Value::Boolean(b) => visitor.visit_bool(b),
            rlua::Value::Integer(i) => visitor.visit_i64(i as i64),
            rlua::Value::Number(n) => visitor.visit_f64(n as f64),
            rlua::Value::String(s) => visitor.visit_str(s.to_str()?),
            rlua::Value::Table(t) => visitor.visit_seq(
                SeqAccess {
                    lua: self.lua,
                    index: 1,
                    table: t,
                }
            ),
            _ => bail!(
                ErrorKind::DeserializeType(format!("Attempt to deserialize invalid data {:?}", self.value))
            ),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_any(visitor)
    }
}

struct SeqAccess<'lua> {
    lua: &'lua rlua::Lua,
    index: usize,
    table: rlua::Table<'lua>
}

impl<'de> de::SeqAccess<'de> for SeqAccess<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>
    {
        let val = self.table.raw_get(self.index)?;
        self.index += 1;
        if let rlua::Value::Nil = val {
            Ok(None)
        } else {
            Ok(Some(seed.deserialize(
                Deserializer {
                    lua: self.lua,
                    value: val,
                }
            )?))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.table.raw_len() as usize - self.index + 1)
    }
}

struct MapAccess<'lua> {
    lua: &'lua rlua::Lua,
    last_value: rlua::Value<'lua>,
    iterator: rlua::TablePairs<'lua, rlua::Value<'lua>, rlua::Value<'lua>>,
}

impl<'de> de::MapAccess<'de> for MapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>
    {
        if let Some(item) = self.iterator.next() {
            let (key, value) = item?;
            self.last_value = value;
            Ok(Some(seed.deserialize(
                Deserializer {
                    lua: self.lua,
                    value: key,
                }
            )?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<K>(&mut self, seed: K) -> Result<K::Value>
    where
        K: de::DeserializeSeed<'de>
    {
        let mut new_value = rlua::Value::Nil;
        std::mem::swap(&mut new_value, &mut self.last_value);
        Ok(
            seed.deserialize(
                Deserializer {
                    lua: self.lua,
                    value: new_value,
                }
            )?
        )
    }

    fn size_hint(&self) -> Option<usize> {
        if let (lower, Some(upper)) = self.iterator.size_hint() {
            Some(upper - lower)
        } else {
            None
        }
    }
}

impl<'de> de::EnumAccess<'de> for Deserializer<'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>
    {
        match self.value {
            s@rlua::Value::String(_) => {
                return Ok(
                    (
                        seed.deserialize(Deserializer {
                            lua: self.lua,
                            value: s,
                        })?,
                        Deserializer {
                            lua: self.lua,
                            value: rlua::Value::Nil,
                        }
                    )
                );
            },
            rlua::Value::Table(ref t) => {
                if let s@rlua::Value::String(_) = t.raw_get(1)? {
                    let inner_value = t.raw_get(2)?;
                    return Ok(
                        (
                            seed.deserialize(Deserializer {
                                lua: self.lua,
                                value: s,
                            })?,
                            Deserializer {
                                lua: self.lua,
                                value: inner_value,
                            }
                        )
                    )
                }
            },
            _ => {}
        }
        bail!(
            ErrorKind::Custom(format!("attempt to deserialize {:?} as enum", self.value))
        )
    }

}

impl<'de> de::VariantAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if let rlua::Value::Nil = self.value {
            Ok(())
        } else {
            bail!(
                ErrorKind::Custom(
                    format!("attempt to deserialize {:?} as a unit variant", self.value)
                )
            )
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

pub struct ToLuaN<'a, T: 'a>(pub &'a T);

impl<'lua, 'a, T: 'a> ToLua<'lua> for ToLuaN<'a, T>
where
    T: ser::Serialize
{
    fn to_lua(self, lua: &'lua rlua::Lua) -> rlua::Result<rlua::Value<'lua>> {
        let result = self.0.serialize(
            Serializer {
                lua: lua,
            }
        );
        match result {
            Ok(value) => Ok(value),
            Err(Error(ErrorKind::Lua(e), _)) => Err(e),
            Err(e) => Err(
                rlua::Error::ToLuaConversionError {
                    from: "",
                    to: "",
                    message: Some(e.description().to_string()),
                }
            ),
        }
    }
}

pub struct FromLuaN<T>(pub T);

impl<'lua, T> FromLua<'lua> for FromLuaN<T>
where
    T: de::Deserialize<'lua>
{
    fn from_lua(lua_value: rlua::Value<'lua>, lua: &'lua rlua::Lua) -> rlua::Result<Self> {
        let t = T::deserialize(
            Deserializer {
                lua: lua,
                value: lua_value,
            }
        );
        match t {
            Ok(value) => Ok(FromLuaN(value)),
            Err(Error(ErrorKind::Lua(e), _)) => Err(e),
            Err(e) => Err(
                rlua::Error::FromLuaConversionError {
                    from: "",
                    // XXX - I could use serde or another derive macro
                    // to get the name of the Rust type. Decent amount
                    // of code for questionable benefit though.
                    to: "",
                    message: Some(format!("{}", e))
                }
            ),
        }
    }
}
