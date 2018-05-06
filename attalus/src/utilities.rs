use std::fmt;
use std::marker::PhantomData;
use std::thread;
use std::time::{Duration, Instant};

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

macro_rules! type_to_format_string {
    (u8) => {
        "{:>02X}"
    };
    (u16) => {
        "{:>04X}"
    };
    (u32) => {
        "{:>08X}"
    };
    (u64) => {
        "{:>016X}"
    };
    (usize) => {
        "{:>016X}"
    };
    ($t:ty) => {
        "{}"
    };
}

macro_rules! display_branch {
    ($x: ident, $formatter: ident, $variant: ident) => {
        if let $variant = $x {
            return stringify!($variant).fmt($formatter);
        }
    };
    ($x: ident, $formatter: ident, $variant: ident {
        $($member_name: ident: $typ: ty),*
    }) => {
        if let $variant {
            $($member_name),*
        } = $x {
            return
                format_args!(
                    concat!(
                        stringify!($variant_name),
                        " {{ ",
                        $(
                            stringify!($member_name:), " ", type_to_format_string!($typ), ",",
                        )*
                        " }}"
                    ),
                    $($member_name),*
                ).fmt($formatter);
        }
    }
}

use serde::de::{Deserialize, Deserializer, Error as DeError, SeqAccess, Visitor};

//// Deriving help

macro_rules! impl_partial_eq_via {
    ($my_type:ty, $other_type:ty) => {
        impl PartialEq for $my_type {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                use std::mem::transmute;
                // assert that types are the same size
                let _ = transmute::<$my_type, $other_type>;
                let s: &$other_type = unsafe { transmute(self) };
                let o: &$other_type = unsafe { transmute(rhs) };
                s == o
            }
        }
    };
}

macro_rules! impl_serde_via {
    ($my_type:ty, $other_type:ty) => {
        use serde as _macro_serde;
        use std as _macro_std;
        impl<'de> _macro_serde::Deserialize<'de> for $my_type {
            #[inline]
            fn deserialize<D>(deserializer: D) -> _macro_std::result::Result<Self, D::Error>
            where
                D: _macro_serde::Deserializer<'de>,
            {
                use std::mem::transmute;
                <$other_type as _macro_serde::Deserialize<'de>>::deserialize(deserializer)
                    .map(|x| unsafe { transmute(x) })
            }
        }

        impl _macro_serde::Serialize for $my_type {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> _macro_std::result::Result<S::Ok, S::Error>
            where
                S: _macro_serde::Serializer,
            {
                use std::mem::transmute;
                // assert that types are the same size
                let _ = transmute::<$my_type, $other_type>;
                let o: &$other_type = unsafe { transmute(self) };
                <$other_type as _macro_serde::Serialize>::serialize(o, serializer)
            }
        }
    };
}

macro_rules! impl_hash_via {
    ($my_type:ty, $other_type:ty) => {
        use std::hash as _macro_hash;
        impl _macro_hash::Hash for $my_type {
            #[inline]
            fn hash<H: _macro_hash::Hasher>(&self, state: &mut H) {
                use std::mem::transmute;
                // assert that types are the same size
                let _ = transmute::<$my_type, $other_type>;
                let o: &$other_type = unsafe { transmute(self) };
                <$other_type as _macro_hash::Hash>::hash(o, state);
            }
        }
    };
}

//// Serializing and deserializing arrays

/// `ArrayWrap<[T; len]>` can be deserialized if `T` implements `Deserialize`,
/// `Copy`, and `Default`, and if `array_deserialize!{len}` has been called.
/// Ditto `ArrayWrap<Vec<[T; len]>>` if `vec_array_deserialize!{len}` has been
/// called (which requires `array_deserialize`).
pub struct ArrayWrap<T>(pub T);

struct ArrayVisitor<T>(PhantomData<T>);

macro_rules! array_deserialize {
    ($array_len:expr) => {
        impl<'de, T> Visitor<'de> for ArrayVisitor<[T; $array_len]>
        where
            T: Deserialize<'de> + Copy + Default,
        {
            type Value = ArrayWrap<[T; $array_len]>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of length {}", $array_len)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let error = |i: usize| Err(A::Error::invalid_length(i, &self));

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
            T: Deserialize<'de> + Copy + Default,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_seq(ArrayVisitor::<[T; $array_len]>(Default::default()))
            }
        }
    };
}

array_deserialize!{0x2000}
array_deserialize!{0x4000}
array_deserialize!{0x10000}

//// Things that are immediately helpful for an emulator

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeInfo {
    pub total_cycles: u64,
    pub cycles_start: u64,
    pub frequency: Option<u64>,
    pub start_time: Instant,
    pub hold_duration: Duration,
}

/// Sleeps until a clock running at the given `frequency` would have run for `target_cycles`.
///
///
/// * `start_time` - What time did we start measuring at?
/// * `start_cycles` - How many cycles were on the clock at `start_time`?
/// * `target_cycles` - How many cycles do we want on the clock when we get done sleeping?
/// * `frequency` - What's the frequency (in Hz) the clock is running at?
pub fn time_govern2(start_time: Instant, start_cycles: u64, target_cycles: u64, frequency: u64) {
    // We need target_cycles - start_cycles = (sleep_time + now - start_time) * frequency

    let now = Instant::now();
    let duration_since_start = now.duration_since(start_time);

    let cycles_diff = target_cycles.wrapping_sub(start_cycles);
    let total_time_seconds = cycles_diff / frequency;
    let remainder_cycles = cycles_diff % frequency;
    let total_time_nanos = (1000000000 * remainder_cycles) / frequency;
    let total_duration = Duration::new(total_time_seconds, total_time_nanos as u32);

    if let Some(sleep_duration) = total_duration.checked_sub(duration_since_start) {
        thread::sleep(sleep_duration);
    }
}

pub fn time_govern(time_info: TimeInfo) {
    debug_assert!(time_info.cycles_start <= time_info.total_cycles);

    let now = Instant::now();

    if let Some(frequency) = time_info.frequency {
        let guest_cycles = time_info.total_cycles - time_info.cycles_start;
        let guest_duration_seconds = guest_cycles / frequency;
        let guest_cycles_remaining = guest_cycles % frequency;
        let guest_duration_subsec_nanos = (1000000000 * guest_cycles_remaining) / frequency;
        let guest_duration =
            Duration::new(guest_duration_seconds, guest_duration_subsec_nanos as u32);

        let host_total_duration = now.duration_since(time_info.start_time);
        debug_assert!(host_total_duration >= time_info.hold_duration);
        let host_active_duration = host_total_duration - time_info.hold_duration;

        if let Some(diff_duration) = guest_duration.checked_sub(host_active_duration) {
            thread::sleep(diff_duration);
        }
    };
}
