// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

/// Iterate T over a sequence of types.
///
/// This macro and `type_select` are intended to help with the mismatch between
/// generic types and user input.
///
/// ```
/// # #[macro_use] extern crate attalus;
/// # fn main() {
/// type_for! {
///     T in [u8, u16, u32, u64];
///     println!("The size is {}", std::mem::size_of::<T>());
/// }
/// # }
/// ```
///
/// `break` works:
///
/// ```
/// # #[macro_use] extern crate attalus;
/// # fn main() {
/// type_for! {
///     T in [u8, u16, u32, u64];
///     println!("The size is {}", std::mem::size_of::<T>());
///     if std::mem::size_of::<T>() == 4 {
///         break;
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! type_for {
    (@
     $typename: ident,
     [$($typelist: ty,)*],
     $code: expr
    ) => {
        loop {
            {
                $({
                    type $typename = $typelist;
                    $code
                })*
            }
            break;
        }
    };

    (
        $typename: ident in [$($typelist: ty),*];
        $($code: tt)*
    ) => {
        type_for! {@
            $typename,
            [$($typelist,)*],
            {
                $($code)*
            }
        }
    };

    (
        $typename: ident in [$($typelist: ty,)*];
        $($code: tt)*
    ) => {
        type_for! {@
            $typename,
            [$($typelist,)*],
            {
                $($code)*
            }
        }
    };
}

/// Select a type based on a match clause and execute a code block.
///
/// ```
/// # #[macro_use] extern crate attalus;
/// # fn main() {
/// let x = 5;
/// type_select! {
///     match x {
///         0 => u8,
///         1 => u16,
///         2 => u32,
///         _ => u64,
///     } for T {
///         println!("Size is {}", std::mem::size_of::<T>());
///     }
/// }
/// # }
/// ```
///
/// Or you can use an `else` clause
///
/// ```
/// # #[macro_use] extern crate attalus;
/// # fn main() {
/// let x = 5;
/// type_select! {
///     match x {
///         0 => u8,
///         1 => u16,
///         2 => u32,
///     } for T {
///         println!("Size is {}", std::mem::size_of::<T>());
///     } else {
///         println!("No type used");
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! type_select {
    (@
     match {$a: expr} {
         $($stuff: tt)*
     },
     noelse,
     $typename: ident,
     [],
     [],
     $code: expr,
     $else: expr
    ) => {
        match $a {
            $($stuff)*
        }
    };

    (@
     match {$a: expr} {
         $($stuff: tt)*
     },
     useelse,
     $typename: ident,
     [],
     [],
     $code: expr,
     $else: expr
    ) => {
        match $a {
            $($stuff)*
            _ => {
                $else
            }
        }
    };

    (@
     match {$a: expr} {
         $($stuff: tt)*
     },
     $useelse: ident,
     $typename: ident,
     [$typ: ty, $($typs: ty,)*],
     [$patt: pat, $($patts: pat,)*],
     $code: expr,
     $else: expr
    ) => {
        type_select! {@
            match {$a} {
                $($stuff)*
                $patt => {
                    type $typename = $typ;
                    $code
                },
            },
            $useelse,
            $typename,
            [$($typs,)*],
            [$($patts,)*],
            $code,
            $else
        }
    };

    (
        match $a: tt {
            $($patt: pat => $typ: ty,)*
        } for $typename: ident {
            $($code: tt)*
        } else {
            $($else: tt)*
        }
    ) => {
        type_select! {@
            match {$a} {
            },
            useelse,
            $typename,
            [$($typ,)*],
            [$($patt,)*],
            { $($code)* },
            { $($else)* }
        }
    };

    (
        match $a: tt {
            $($patt: pat => $typ: ty,)*
        } for $typename: ident {
            $($code: tt)*
        }
    ) => {
        type_select! {@
            match {$a} {
            },
            noelse,
            $typename,
            [$($typ,)*],
            [$($patt,)*],
            { $($code)* },
            {}
        }
    };

    (
        match $a: tt {
            $($patt: pat => $typ: ty),*
        }
        $($rest: tt)*
    ) => {
        type_select! {
            match $a {
                $($patt => $typ,)*
            }
            $($rest)*
        }
    };
}
