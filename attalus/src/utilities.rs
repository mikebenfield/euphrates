// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::marker::PhantomData;
use std::fmt;

use ::errors::*;

pub fn to16(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

pub fn to8(x: u16) -> (u8, u8) {
    ((x & 0xFF) as u8, ((x & 0xFF00) >> 8) as u8)
}

pub fn set_bit(dest: &mut u8, bit: u8) {
    *dest |= 1 << bit;
}

pub fn clear_bit(dest: &mut u8, bit: u8) {
    *dest &= !(1 << bit);
}

use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error as DeError};
use serde::ser::{Serialize, Serializer};

/// A VecArrayWrap<Vec<[T; len]>> can be serialized, at least if `vec_array_serialize!{len}` has been called.
pub struct VecArrayWrap<'a, T: 'a>(pub &'a T);

macro_rules! vec_array_serialize {
    ($array_len: expr) => {
        impl<'a, T> Serialize for VecArrayWrap<'a, Vec<[T; $array_len]>>
        where
            T: Serialize + 'a
        {
            fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
            where
                S: Serializer
            {
                use serde::ser::SerializeSeq;

                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for page in self.0.iter() {
                    let page0: &[T] = page;
                    seq.serialize_element(page0)?;
                }
                seq.end()
            }
        }
    }
}

/// `ArrayWrap<[T; len]>` can be deserialized if `T` implements `Deserialize`,
/// `Copy`, and `Default`, and if `array_deserialize!{len}` has been called.
/// Ditto `ArrayWrap<Vec<[T; len]>>` if `vec_array_deserialize!{len}` has been
/// called (which requires `array_deserialize`).
pub struct ArrayWrap<T>(pub T);

struct ArrayVisitor<T>(PhantomData<T>);

macro_rules! array_deserialize {
    ($array_len: expr) => {
        impl<'de, T> Visitor<'de> for ArrayVisitor<[T; $array_len]>
        where
            T: Deserialize<'de> + Copy + Default
        {
            type Value = ArrayWrap<[T; $array_len]>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "an array of length {}",
                    $array_len
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
            where
                A: SeqAccess<'de>
            {
                let error = |i: usize| {
                    Err(
                        A::Error::invalid_length(
                            i,
                            &self,
                        )
                    )
                };

                let mut result = [Default::default(); $array_len];
                let mut i = 0;
                while let Some(a) = seq.next_element::<T>()? {
                    if i >= $array_len {
                        return error(i);
                    }
                    result[i] = a;
                    i += 1;
                }

                if i != $array_len {
                    return error(i);
                }

                Ok(ArrayWrap(result))
            }
        }

        impl<'de, T> Deserialize<'de> for ArrayWrap<[T; $array_len]>
        where
            T: Deserialize<'de> + Copy + Default
        {
            fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
            where
                D: Deserializer<'de>
            {
                deserializer.deserialize_seq(ArrayVisitor::<[T; $array_len]>(Default::default()))
            }
        }
    }
}

macro_rules! vec_array_deserialize {
    ($array_len: expr) => {
        impl<'de, T> Deserialize<'de> for ArrayWrap<Vec<[T; $array_len]>>
        where
            T: Deserialize<'de> + Copy + Default
        {
            fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
            where
                D: Deserializer<'de>
            {
                use std::mem;
                let result = <Vec<ArrayWrap<[T; $array_len]>> as Deserialize>::deserialize(deserializer)?;
                Ok(
                    unsafe {
                        mem::transmute(result)
                    }
                )
            }
        }
    }
}

vec_array_serialize!{0x2000}

array_deserialize!{0x2000}
array_deserialize!{0x4000}

vec_array_deserialize!{0x2000}

macro_rules! utilities_count {
    () => {
        0
    };

    ($y: tt, $($x: tt,)*) => {
        1 + utilities_count!($($x,)*)
    };
}

macro_rules! serde_struct_arrays {
    (
        $modname: ident,
        $typename: ident,
        [$($plain_field: ident,)*],
        [$($array_field: ident : [$array_ty: ty; $array_len: expr],)*],
        [$($vec_of_arrays_field: ident : [$vec_of_arrays_ty: ty; $vec_of_arrays_len: expr],)*]
    ) => { mod $modname {
        use super::$typename;

        use std::fmt;

        use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, MapAccess, Error};
        use serde::ser::{Serialize, Serializer, SerializeStruct};

        use ::utilities;

        impl<'de> Deserialize<'de> for $typename {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>
            {
                #[allow(non_camel_case_types)]
                #[derive(Eq, PartialEq, Deserialize)]
                enum Field { 
                    $($plain_field,)*
                    $($array_field,)*
                    $($vec_of_arrays_field,)*
                }

                struct Visitor0;

                impl<'de> Visitor<'de> for Visitor0 {
                    type Value = $typename;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(stringify!(struct $typename))
                    }

                    #[allow(unused_assignments)]
                    fn visit_seq<V>(self, mut seq: V) -> Result<$typename, V::Error>
                    where
                        V: SeqAccess<'de>
                    {
                        let mut i = 0;

                        macro_rules! extract {
                            ($t: ty) => {
                                {
                                    let result: $t = seq.next_element()?
                                        .ok_or_else(|| V::Error::invalid_length(i, &self))?;
                                    i += 1;
                                    result.0
                                }
                            };

                            () => {
                                {
                                    let result = seq.next_element()?
                                        .ok_or_else(|| V::Error::invalid_length(i, &self))?;
                                    i += 1;
                                    result
                                }
                            };
                        }

                        $(
                            let $plain_field = extract!();
                        )*

                        $(
                            let $array_field = extract!(utilities::ArrayWrap<[$array_ty; $array_len]>);
                        )*

                        $(
                            let $vec_of_arrays_field = extract!(utilities::ArrayWrap<Vec<[$vec_of_arrays_ty; $vec_of_arrays_len]>>);
                        )*

                        Ok(
                            $typename {
                                $(
                                    $plain_field,
                                )*
                                $(
                                    $array_field,
                                )*
                                $(
                                    $vec_of_arrays_field,
                                )*
                            }
                        )
                    }

                    #[allow(unused_assignments)]
                    fn visit_map<V>(self, mut map: V) -> Result<$typename, V::Error>
                    where
                        V: MapAccess<'de>
                    {
                        $(
                            let mut $plain_field = None;
                        )*

                        $(
                            let mut $array_field:
                                Option<utilities::ArrayWrap<[$array_ty; $array_len]>> = None;
                        )*

                        $(
                            let mut $vec_of_arrays_field:
                                Option<utilities::ArrayWrap<Vec<[$vec_of_arrays_ty; $vec_of_arrays_len]>>> = None;
                        )*

                        macro_rules! branch {
                            ($key: expr, $field: ident) => {
                                if $key == Field::$field {
                                    if $field.is_some() {
                                        return Err(V::Error::duplicate_field(stringify!($field)));
                                    }
                                    $field = Some(map.next_value()?);
                                }
                            }
                        }

                        while let Some(key) = map.next_key::<Field>()? {
                            $(
                                branch!{key, $plain_field}
                            )*
                            $(
                                branch!{key, $array_field}
                            )*
                            $(
                                branch!{key, $vec_of_arrays_field}
                            )*
                            unreachable!();
                        }

                        $(
                            let $plain_field = $plain_field.ok_or_else(
                                || V::Error::missing_field(stringify!($plain_field))
                            )?;
                        )*
                        $(
                            let $array_field = $array_field.ok_or_else(
                                || V::Error::missing_field(stringify!($array_field))
                            )?.0;
                        )*
                        $(
                            let $vec_of_arrays_field = $vec_of_arrays_field.ok_or_else(
                                || V::Error::missing_field(stringify!($vec_of_arrays_field))
                            )?.0;
                        )*

                        Ok(
                            $typename {
                                $($plain_field,)*
                                $($array_field,)*
                                $($vec_of_arrays_field,)*
                            }
                        )
                    }
                }

                const FIELDS: &'static [&'static str] = &[
                    $(
                        stringify!($plain_field),
                    )*
                    $(
                        stringify!($array_field),
                    )*
                    $(
                        stringify!($vec_of_arrays_field),
                    )*
                ];

                deserializer.deserialize_struct(stringify!($typename), FIELDS, Visitor0)
            }
        }

        impl Serialize for $typename {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer
            {
                let mut struc = serializer.serialize_struct(
                    stringify!($typename),
                    utilities_count!($($plain_field,)* $($array_field,)* $($vec_of_arrays_field,)*)
                )?;

                $(
                    struc.serialize_field(
                        stringify!($plain_field),
                        &self.$plain_field
                    )?;
                )*

                $(
                    {
                        let slice: &[$array_ty] = &self.$array_field;
                        struc.serialize_field(
                            stringify!($array_field),
                            slice
                        )?;
                    }
                )*

                $(
                    struc.serialize_field(
                        stringify!($vec_of_arrays_field),
                        &utilities::VecArrayWrap(&self.$vec_of_arrays_field)
                    )?;
                )*

                struc.end()
            }
        }
    }
}}
