//! Heap data estimator.
//!
//! The `datasize` crate allows estimating the amount of heap memory used by a value. It does so by
//! providing or deriving an implementation of the `DataSize` trait, which knows how to calculate
//! the size for many `std` types and primitives.
//!
//! The aim is to get a reasonable approximation of memory usage, especially with variably sized
//! types like `Vec`s. While it is acceptable to be a few bytes off in some cases, any user should
//! be able to easily tell whether their memory is growing linearly or logarithmically by glancing
//! at the reported numbers.
//!
//! The crate does not take alignment or memory layouts into account, or unusual behavior or
//! optimizations of allocators. It is depending entirely on the data inside the type, thus the name
//! of the crate.
//!
//! # General usage
//!
//! For any type that implements `DataSize`, the `data_size` convenience function can be used to
//! guess the size of its heap allocation:
//!
//! ```rust
//! use datasize::data_size;
//!
//! let data: Vec<u64> = vec![1, 2, 3];
//! assert_eq!(data_size(&data), 24);
//! ```
//!
//! Types implementing the trait also provide two additional constants, `IS_DYNAMIC` and
//! `STATIC_HEAP_SIZE`.
//!
//! `IS_DYNAMIC` indicates whether a value's size can change over time:
//!
//! ```rust
//! use datasize::DataSize;
//!
//! // A `Vec` of any kind may have elements added or removed, so it changes size.
//! assert!(Vec::<u64>::IS_DYNAMIC);
//!
//! // The elements of type `u64` in it are not dynamic. This allows the implementation to
//! // simply estimate the size as number_of_elements * size_of::<u64>.
//! assert!(!u64::IS_DYNAMIC);
//! ```
//!
//! Additionally, `STATIC_HEAP_SIZE` indicates the amount of heap memory a type will always use. A
//! good example is a `Box<u64>` -- it will always use 8 bytes of heap memory, but not change in
//! size:
//!
//!
//! ```rust
//! use datasize::DataSize;
//!
//! assert_eq!(Box::<u64>::STATIC_HEAP_SIZE, 8);
//! assert!(!Box::<u64>::IS_DYNAMIC);
//! ```
//!
//! # Overriding derived data size calculation for single fields.
//!
//! On structs (but not enums!) the calculation for heap size can be overriden for single fields,
//! which is useful when dealing with third-party crates whose fields do not implement `DataSize` by
//! simply annotating it with `#[data_size(with = ...)]` and pointing to a `Fn(T) -> usize`
//! function:
//!
//! ```rust
//! use datasize::DataSize;
//!
//! // Let's pretend this type is from a foreign crate.
//! struct ThirdPartyType;
//!
//! fn estimate_third_party_type(value: &Vec<ThirdPartyType>) -> usize {
//!     // We assume every item is 512 bytes in heap size.
//!     value.len() * 512
//! }
//!
//! #[derive(DataSize)]
//! struct MyStruct {
//!     items: Vec<u32>,
//!     #[data_size(with = estimate_third_party_type)]
//!     other_stuff: Vec<ThirdPartyType>,
//! }
//! ```
//!
//! This automatically marks the whole struct as always dynamic, so the custom estimation function
//! is called every time `MyStruct` is sized.
//!
//! # Implementing `DataSize` for custom types
//!
//! The `DataSize` trait can be implemented for custom types manually:
//!
//! ```rust
//! # use datasize::{DataSize, data_size};
//! struct MyType {
//!     items: Vec<i64>,
//!     flag: bool,
//!     counter: Box<u64>,
//! }
//!
//! impl DataSize for MyType {
//!     // `MyType` contains a `Vec`, so `IS_DYNAMIC` is set to true.
//!     const IS_DYNAMIC: bool = true;
//!
//!     // The only always present heap item is the `counter` value, which is 8 bytes.
//!     const STATIC_HEAP_SIZE: usize = 8;
//!
//!     #[inline]
//!     fn estimate_heap_size(&self) -> usize {
//!         // We can be lazy here and delegate to all the existing implementations:
//!         data_size(&self.items) + data_size(&self.flag) + data_size(&self.counter)
//!     }
//! }
//!
//! let my_data = MyType {
//!     items: vec![1, 2, 3],
//!     flag: true,
//!     counter: Box::new(42),
//! };
//!
//! // Three i64 and one u64 on the heap sum up to 32 bytes:
//! assert_eq!(data_size(&my_data), 32);
//! ```
//!
//! Since implementing this for `struct` types is cumbersome and repetitive, the crate provides a
//! `DataSize` macro for convenience:
//!
//! ```
//! # use datasize::{DataSize, data_size};
//! // Equivalent to the manual implementation above:
//! #[derive(DataSize)]
//! struct MyType {
//!     items: Vec<i64>,
//!     flag: bool,
//!     counter: Box<u64>,
//! }
//! # let my_data = MyType {
//! #     items: vec![1, 2, 3],
//! #     flag: true,
//! #     counter: Box::new(42),
//! # };
//! # assert_eq!(data_size(&my_data), 32);
//! ```
//!
//! See the `DataSize` macro documentation in the `datasize_derive` crate for details.
//!
//! ## Performance considerations
//!
//! Determining the full size of data can be quite expensive, especially if multiple nested levels
//! of dynamic types are used. The crate uses `IS_DYNAMIC` and `STATIC_HEAP_SIZE` to optimize when
//! it can, so in many cases not every element of a vector needs to be checked individually.
//!
//! However, if the contained types are dynamic, every element must (and will) be checked, so keep
//! this in mind when performance is an issue.
//!
//! ## Handlings references, `Arc`s and similar types
//!
//! Any reference will be counted as having a data size of 0, as it does not own the value. There
//! are some special reference-like types like `Arc`, which are discussed below.
//!
//! ### `Arc` and `Rc`
//!
//! Currently `Arc`s are not supported. A planned development is to allow users to mark an instance
//! of an `Arc` as "primary" and have its heap memory usage counted, but currently this is not
//! implemented.
//!
//! Any `Arc` will be estimated to have a heap size of `0`, to avoid cycles resulting in infinite
//! loops.
//!
//! The `Rc` type is handled in the same manner.
//!
//! ## Additional types
//!
//! Some additional types from external crates are available behind feature flags.
//!
//! * `fake_clock-types`: Support for the `fake_instant::FakeClock` type.
//! * `futures-types`: Some types from the `futures` crate.
//! * `smallvec-types`: Support for the `smallvec::SmallVec` type.
//! * `tokio-types`: Some types from the `tokio` crate.
//!
//! ## `no_std` support
//!
//! Although slightly paradoxical due to the fact that without `std` or at least `alloc` there won't
//! be any heap in most cases, the crate supports a `no_std` environment. Disabling the "std"
//! feature (by disabling default features) will produce a version of the crate that does not rely
//! on the standard library. This can be used to derive the `DataSize` trait for types without
//! boilerplate, even though their heap size will usually be 0.
//!
//! ## Arrays and const generics
//!
//! By default, this crate requires at least Rust version 1.51.0, in order to implement DataSize
//! for [T; N] arrays generically. This implementation is provided by the "const-generics"
//! feature flag, which is enabled by default. In order to use an older Rust version,
//! you can specify [`default-features = false`](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features) and `features = ["std"]` for `datasize` in your Cargo.toml.
//!
//! When the `const-generics` feature flag is disabled, a DataSize implementation will be provided
//! for arrays of small sizes, and for some larger sizes related to powers of 2.
//!
//! ## Known issues
//!
//! The derive macro currently does not support generic structs with inline type bounds, e.g.
//!
//! ```ignore
//! struct Foo<T: Copy> { ... }
//! ```
//!
//! This can be worked around by using an equivalent `where` clause:
//!
//! ```ignore
//! struct Foo<T>
//! where T: Copy
//! { ... }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::assertions_on_constants)]

#[cfg(feature = "fake_clock-types")]
mod fake_clock;
#[cfg(feature = "futures-types")]
mod futures;
#[cfg(feature = "smallvec-types")]
mod smallvec;
#[cfg(feature = "std")]
mod std;
#[cfg(feature = "tokio-types")]
mod tokio;

pub use datasize_derive::DataSize;

/// A `const fn` variant of the `min` function.
pub const fn min(a: usize, b: usize) -> usize {
    [a, b][(a > b) as usize]
}

/// Indicates that a type knows how to approximate its memory usage.
pub trait DataSize {
    /// If `true`, the type has a heap size that can vary at runtime, depending on the actual value.
    const IS_DYNAMIC: bool;

    /// The amount of space a value of the type _always_ occupies. If `IS_DYNAMIC` is false, this is
    /// the total amount of heap memory occupied by the value. Otherwise this is a lower bound.
    const STATIC_HEAP_SIZE: usize;

    /// Estimates the size of heap memory taken up by this value.
    ///
    /// Does not include data on the stack, which is usually determined using `mem::size_of`.
    fn estimate_heap_size(&self) -> usize;

    #[cfg(feature = "detailed")]
    /// Create a tree of memory estimations.
    ///
    /// Similar to `estimate_heap_size`, but the returned value is a tree that typically reports
    /// memory used by structs individually.
    ///
    /// Requires the `detailed` feature to be enabled.
    #[inline]
    fn estimate_detailed_heap_size(&self) -> MemUsageNode {
        MemUsageNode::Size(self.estimate_heap_size())
    }
}

#[cfg(feature = "detailed")]
/// A node in a memory reporting tree.
#[derive(Debug, serde::Serialize, PartialEq)]
pub enum MemUsageNode {
    Size(usize),
    Detailed(::std::collections::HashMap<&'static str, MemUsageNode>),
}

#[cfg(feature = "detailed")]
impl MemUsageNode {
    /// Calculate the total memory usage given by detailed estimate
    #[inline]
    pub fn total(&self) -> usize {
        match self {
            MemUsageNode::Size(sz) => *sz,
            MemUsageNode::Detailed(members) => members.values().map(MemUsageNode::total).sum(),
        }
    }
}

/// Estimates allocated heap data from data of value.
///
/// Checks if `T` is dynamic; if it is not, returns `T::STATIC_HEAP_SIZE`. Otherwise delegates to
/// `T::estimate_heap_size`.
#[inline]
pub fn data_size<T: ?Sized>(value: &T) -> usize
where
    T: DataSize,
{
    value.estimate_heap_size()
}

#[cfg(feature = "detailed")]
/// Estimates allocated heap data from data of value.
#[inline]
pub fn data_size_detailed<T: ?Sized>(value: &T) -> MemUsageNode
where
    T: DataSize,
{
    value.estimate_detailed_heap_size()
}

/// Helper macro to define a heap size for one or more non-dynamic types.
#[macro_export]
macro_rules! non_dynamic_const_heap_size {
    ($($ty:ty)*, $sz:expr) => {
        $(impl DataSize for $ty {
            const IS_DYNAMIC: bool = false;
            const STATIC_HEAP_SIZE: usize = $sz;

            #[inline]
            fn estimate_heap_size(&self) -> usize {
                $sz
            }
        })*
    };
}

// Hack to allow `+` to be used to join macro arguments.
macro_rules! strip_plus {
    (+ $($rest: tt)*) => {
        $($rest)*
    }
}

macro_rules! tuple_heap_size {
    ($($n:tt $name:ident);+) => {
        impl<$($name),*> DataSize for ($($name),*)
        where $($name: DataSize),*
        {
            const IS_DYNAMIC: bool = $($name::IS_DYNAMIC)|*;

            const STATIC_HEAP_SIZE: usize =
                strip_plus!($(+ $name::STATIC_HEAP_SIZE)+);

            #[inline]
            fn estimate_heap_size(&self) -> usize {
                strip_plus!($(+ self.$n.estimate_heap_size())+)
            }
        }
    };
}

#[cfg(not(feature = "const-generics"))]
macro_rules! array_heap_size {
    ($($n:tt)+) => {
        $(
        impl<T> DataSize for [T; $n]
        where
            T: DataSize,
        {
            const IS_DYNAMIC: bool = T::IS_DYNAMIC;

            const STATIC_HEAP_SIZE: usize = T::STATIC_HEAP_SIZE * $n;

            #[inline]
            fn estimate_heap_size(&self) -> usize {
                if T::IS_DYNAMIC {
                    (&self[..]).iter().map(DataSize::estimate_heap_size).sum()
                } else {
                    T::STATIC_HEAP_SIZE * $n
                }
            }
        }
        )*
    };
}

// Primitives
non_dynamic_const_heap_size!(() u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize bool char f32 f64, 0);

// Assorted heapless `core` types
non_dynamic_const_heap_size!(core::time::Duration, 0);

tuple_heap_size!(0 T0; 1 T1);
tuple_heap_size!(0 T0; 1 T1; 2 T2);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10; 11 T11);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10; 11 T11; 12 T12);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10; 11 T11; 12 T12; 13 T13);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10; 11 T11; 12 T12; 13 T13; 14 T14);
tuple_heap_size!(0 T0; 1 T1; 2 T2; 3 T3; 4 T4; 5 T5; 6 T6; 7 T7; 8 T8; 9 T9; 10 T10; 11 T11; 12 T12; 13 T13; 14 T14; 15 T15);

impl<T0> DataSize for (T0,)
where
    T0: DataSize,
{
    const IS_DYNAMIC: bool = T0::IS_DYNAMIC;
    const STATIC_HEAP_SIZE: usize = T0::STATIC_HEAP_SIZE;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        self.0.estimate_heap_size()
    }
}

#[cfg(not(feature = "const-generics"))]
array_heap_size!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 128 192 256 384 512 1024 2048 4096 8192 16384 1048576 2097152 3145728 4194304);

#[cfg(feature = "const-generics")]
impl<T, const N: usize> DataSize for [T; N]
where
    T: DataSize,
{
    const IS_DYNAMIC: bool = T::IS_DYNAMIC;

    const STATIC_HEAP_SIZE: usize = T::STATIC_HEAP_SIZE * N;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        if T::IS_DYNAMIC {
            (&self[..]).iter().map(DataSize::estimate_heap_size).sum()
        } else {
            T::STATIC_HEAP_SIZE * N
        }
    }
}

// REFERENCES

impl<T> DataSize for &T {
    const IS_DYNAMIC: bool = false;

    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        0
    }
}

impl<T> DataSize for &mut T {
    const IS_DYNAMIC: bool = false;

    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        0
    }
}

// COMMONLY USED NON-PRIMITIVE TYPES

impl<T> DataSize for Option<T>
where
    T: DataSize,
{
    // Options are only not dynamic if their type has no heap data at all and is not dynamic.
    const IS_DYNAMIC: bool = (T::IS_DYNAMIC || T::STATIC_HEAP_SIZE > 0);

    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        match self {
            Some(val) => data_size(val),
            None => 0,
        }
    }
}

impl<T, E> DataSize for Result<T, E>
where
    T: DataSize,
    E: DataSize,
{
    // Results are only not dynamic if their types have no heap data at all and are not dynamic.
    const IS_DYNAMIC: bool =
        (T::IS_DYNAMIC || E::IS_DYNAMIC || (T::STATIC_HEAP_SIZE != E::STATIC_HEAP_SIZE));

    const STATIC_HEAP_SIZE: usize = min(T::STATIC_HEAP_SIZE, E::STATIC_HEAP_SIZE);

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        match self {
            Ok(val) => data_size(val),
            Err(err) => data_size(err),
        }
    }
}

impl<T> DataSize for core::marker::PhantomData<T> {
    const IS_DYNAMIC: bool = false;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        0
    }
}

impl<T: DataSize> DataSize for core::ops::Range<T> {
    const IS_DYNAMIC: bool = T::IS_DYNAMIC;
    const STATIC_HEAP_SIZE: usize = 2 * T::STATIC_HEAP_SIZE;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        self.start.estimate_heap_size() + self.end.estimate_heap_size()
    }
}

#[cfg(test)]
mod tests {
    use crate as datasize; // Required for the derive macro.
    use crate::{data_size, DataSize};

    #[test]
    fn test_for_simple_builtin_types() {
        // We only sample some, as they are all macro generated.
        assert_eq!(1u8.estimate_heap_size(), 0);
        assert_eq!(1u16.estimate_heap_size(), 0);
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(DataSize)]
        struct Foo(u32);

        assert!(!Foo::IS_DYNAMIC);
        assert_eq!(Foo::STATIC_HEAP_SIZE, 0);
        assert_eq!(data_size(&Foo(123)), 0);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(DataSize)]
        struct Foo(u32, u8);

        assert!(!Foo::IS_DYNAMIC);
        assert_eq!(Foo::STATIC_HEAP_SIZE, 0);
        assert_eq!(data_size(&Foo(123, 45)), 0);
    }

    #[test]
    fn test_tuple_with_one_element() {
        type Foo = (u32,);
        assert!(!Foo::IS_DYNAMIC);
        assert_eq!(Foo::STATIC_HEAP_SIZE, 0);

        let foo: Foo = (456,);
        assert_eq!(data_size(&foo), 0);
    }

    #[test]
    fn test_result() {
        assert_eq!(Result::<u8, u8>::STATIC_HEAP_SIZE, 0);
        assert!(!Result::<u8, u8>::IS_DYNAMIC);

        assert_eq!(Result::<u8, u16>::STATIC_HEAP_SIZE, 0);
        assert!(!Result::<u8, u16>::IS_DYNAMIC);

        assert_eq!(Result::<u8, Box<u16>>::STATIC_HEAP_SIZE, 0);
        assert!(Result::<u8, Box<u16>>::IS_DYNAMIC);

        assert_eq!(Result::<Box<u8>, u16>::STATIC_HEAP_SIZE, 0);
        assert!(Result::<Box<u8>, u16>::IS_DYNAMIC);

        assert_eq!(Result::<Box<u8>, Box<u16>>::STATIC_HEAP_SIZE, 1);
        assert!(Result::<Box<u8>, Box<u16>>::IS_DYNAMIC);

        assert_eq!(Result::<Box<u16>, Box<u16>>::STATIC_HEAP_SIZE, 2);
        assert!(!Result::<Box<u16>, Box<u16>>::IS_DYNAMIC);

        assert_eq!(Result::<u16, Vec<u16>>::STATIC_HEAP_SIZE, 0);
        assert!(Result::<u16, Vec<u16>>::IS_DYNAMIC);

        assert_eq!(Result::<Vec<u16>, u16>::STATIC_HEAP_SIZE, 0);
        assert!(Result::<Vec<u16>, u16>::IS_DYNAMIC);

        assert_eq!(Result::<Vec<u16>, Vec<u16>>::STATIC_HEAP_SIZE, 0);
        assert!(Result::<Vec<u16>, Vec<u16>>::IS_DYNAMIC);
    }

    #[test]
    fn test_empty_struct() {
        #[derive(DataSize)]
        struct Foo {}

        #[derive(DataSize)]
        struct Bar;

        assert!(!Foo::IS_DYNAMIC);
        assert!(!Bar::IS_DYNAMIC);

        assert_eq!(Foo::STATIC_HEAP_SIZE, 0);
        assert_eq!(Bar::STATIC_HEAP_SIZE, 0);

        assert_eq!(data_size(&Foo {}), 0);
        assert_eq!(data_size(&Bar), 0);
    }

    #[test]
    fn test_empty_enum() {
        #[derive(DataSize)]
        enum Foo {}

        assert!(!Foo::IS_DYNAMIC);
        assert_eq!(Foo::STATIC_HEAP_SIZE, 0);

        // We cannot instantiate empty enums.
    }

    #[test]
    fn macro_does_not_panic_on_foreign_attributes() {
        #[derive(DataSize)]
        /// This docstring shows up as `#[doc = ""]`...
        struct Foo {
            /// This docstring shows up as `#[doc = ""]`...
            dummy: u8,
        }
    }

    // TODO: This does not work, the equivalent should be constructed using `trybuild`.
    // #[test]
    // #[should_panic = "unexpected datasize attribute"]
    // fn macro_panics_on_invalid_data_size_attribute() {
    //     #[derive(DataSize)]
    //     /// This docstring shows up as `#[doc = ""]`...
    //     struct Foo {
    //         #[data_size(invalid)]
    //         /// This docstring shows up as `#[doc = ""]`...
    //         dummy: u8,
    //     }
    // }

    #[test]
    fn keeps_where_clauses_on_structs() {
        #[allow(dead_code)]
        #[derive(DataSize)]
        struct Foo<T>
        where
            T: Copy,
        {
            field: T,
        }
    }

    #[test]
    fn keeps_where_clauses_on_enums() {
        #[allow(dead_code)]
        #[derive(DataSize)]
        enum Foo<T>
        where
            T: Copy,
        {
            Value(T),
        }
    }

    #[test]
    fn use_with_annotation() {
        fn ds_for_field_b(value: &u32) -> usize {
            assert_eq!(*value, 2); // set in the example
            1234
        }

        #[derive(DataSize)]
        struct Foo {
            field_a: u32,
            #[data_size(with = ds_for_field_b)]
            field_b: u32,
            field_c: u32,
        }

        let value = Foo {
            field_a: 1,
            field_b: 2,
            field_c: 3,
        };
        assert_eq!(value.estimate_heap_size(), 1234);
    }

    #[test]
    fn derive_with_default_values_for_generic_parameters() {
        #[derive(DataSize)]
        enum E<T = u32> {
            One(T),
            Two(T),
        }
    }
}
