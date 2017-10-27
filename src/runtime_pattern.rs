
use std::clone::Clone;
use std::fmt::Debug;

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WholePattern<T, P>
{
    Wild,
    WildVariable(Option<T>),
    Patt(P),
}

use self::WholePattern::*;

pub trait Matchable: Sized + Clone {
    type Pattern;

    fn match_impl(&self, pattern: &mut Self::Pattern) -> bool;

    fn matc(&self, pattern: &mut WholePattern<Self, Self::Pattern>) -> bool {
        match pattern {
            &mut Wild => true,
            &mut WildVariable(_) => {
                *pattern = WildVariable(Some(self.clone()));
                true
            },
            &mut Patt(ref mut x) => self.match_impl(x),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatternN<T> {
    It(T),
    Range(T, T),
}

use self::PatternN::*;

macro_rules! impl_matchable_numeric {
    ($typename: ident) => {
        impl Matchable for $typename {
            type Pattern = PatternN<$typename>;

            fn match_impl(&self, pattern: &mut Self::Pattern) -> bool {
                match *pattern {
                    It(x) => x == *self,
                    Range(x, y) => x <= *self && *self <= y,
                }
            }
        }
    }
}

impl_matchable_numeric!{u8}
impl_matchable_numeric!{u16}
impl_matchable_numeric!{u32}
impl_matchable_numeric!{u64}
impl_matchable_numeric!{i8}
impl_matchable_numeric!{i16}
impl_matchable_numeric!{i32}
impl_matchable_numeric!{i64}
impl_matchable_numeric!{f32}
impl_matchable_numeric!{f64}

macro_rules! impl_matchable_eq {
    ($typename: tt) => {
        impl Matchable for $typename {
            type Pattern = Self;

            fn match_impl(&self, pattern: &mut Self) -> bool {
                self == pattern
            }
        }
    }
}

impl_matchable_eq!{bool}
impl_matchable_eq!{char}

macro_rules! impl_matchable_arrays {
    ($typename: ident) => {
        impl_matchable_eq!{[$typename; 1]}
        impl_matchable_eq!{[$typename; 2]}
        impl_matchable_eq!{[$typename; 3]}
        impl_matchable_eq!{[$typename; 4]}
        impl_matchable_eq!{[$typename; 5]}
        impl_matchable_eq!{[$typename; 6]}
        impl_matchable_eq!{[$typename; 7]}
        impl_matchable_eq!{[$typename; 8]}
        impl_matchable_eq!{[$typename; 9]}
        impl_matchable_eq!{[$typename; 10]}
        impl_matchable_eq!{[$typename; 11]}
        impl_matchable_eq!{[$typename; 12]}
        impl_matchable_eq!{[$typename; 13]}
        impl_matchable_eq!{[$typename; 14]}
        impl_matchable_eq!{[$typename; 15]}
        impl_matchable_eq!{[$typename; 16]}
        impl_matchable_eq!{[$typename; 17]}
        impl_matchable_eq!{[$typename; 18]}
        impl_matchable_eq!{[$typename; 19]}
        impl_matchable_eq!{[$typename; 20]}
        impl_matchable_eq!{[$typename; 21]}
        impl_matchable_eq!{[$typename; 22]}
        impl_matchable_eq!{[$typename; 23]}
        impl_matchable_eq!{[$typename; 24]}
        impl_matchable_eq!{[$typename; 25]}
        impl_matchable_eq!{[$typename; 26]}
        impl_matchable_eq!{[$typename; 27]}
        impl_matchable_eq!{[$typename; 28]}
        impl_matchable_eq!{[$typename; 29]}
        impl_matchable_eq!{[$typename; 30]}
        impl_matchable_eq!{[$typename; 31]}
    }
}

impl_matchable_arrays!{u8}
impl_matchable_arrays!{u16}
impl_matchable_arrays!{u32}
impl_matchable_arrays!{u64}
impl_matchable_arrays!{i8}
impl_matchable_arrays!{i16}
impl_matchable_arrays!{i32}
impl_matchable_arrays!{i64}
impl_matchable_arrays!{bool}
impl_matchable_arrays!{char}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Matchable)]
    struct TestUnitStruct;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Matchable)]
    struct TestStruct<T, U> {
        a: u8,
        b: T,
        c: U,
    }

    #[test]
    fn test_struct() {
        let mut pat = Patt(TestStructPattern {
            a: Patt(It(0u8)),
            b: Patt(It(2u32)),
            c: Patt(It(3u64)),
        });
        let ts = TestStruct {
            a: 0u8, b: 2u32, c: 3u64,
        };
        let result = ts.matc(&mut pat);
        assert!(result);

        let ts2 = TestStruct {
            a: 1u8, b: 2u32, c: 3u64,
        };
        let result2 = ts2.matc(&mut pat);
        assert!(!result2);
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Matchable)]
    pub struct TestTupleStruct<T>(pub u8, u8, T);

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Matchable)]
    pub struct TestTupleStruct2(pub u8, u8, f32);

    #[test]
    fn test_tuple_struct() {
        let mut pat = Patt(TestTupleStructPattern(Patt(It(1)), Patt(It(2)), Patt(It(3u64))));
        let tts = TestTupleStruct(1, 2, 3u64);
        let result = tts.matc(&mut pat);
        assert!(result);
    }

    #[test]
    fn test_basic_matching() {
        let x = 18u8;
        let mut patt = Patt(Range(2, 19));
        let result = x.matc(&mut patt);
        assert!(result);

        let result2 = {
            let mut patt2 = WildVariable(None);
            let is_match = x.matc(&mut patt2);
            assert!(
                if let WildVariable(Some(x)) = patt2 {
                    x == 18
                } else {
                    false
                }
            );
            is_match
        };
        assert!(result2);
        assert_eq!(2 + 2, 4);
    }

    #[derive(Clone, Copy, Debug, Matchable, Eq, PartialEq, Serialize, Deserialize)]
    pub enum TestEnum<T> {
        Nope,
        Another(T),
        Yes(u8),
        S {
            a: u8,
            b: u8,
        }
    }

    #[test]
    fn test_enum() {
        let mut patt1 = Patt(TestEnumPattern::Nope);
        let te1 = TestEnum::Nope::<u32>;
        let result1 = te1.matc(&mut patt1);
        assert!(result1);
    }
}
