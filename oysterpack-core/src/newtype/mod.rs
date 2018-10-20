// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! macros for creating standard newtypes

/// Defines a new type for the specified type.
/// - visibility can be specified for the new type and the underlying field
/// - provides a public constructor function named : `pub fn new(value: $T) -> Self `
/// - implements the following traits
///   - Debug
///   - From&lt;T&gt; - where T = underlying type
///   - Deref, where Target = underlying type
/// - metadata attributes can be specified on the new type
///
/// ## Examples
/// ```rust
///
/// #[macro_use]
/// extern crate oysterpack_core;
/// extern crate serde;
/// #[macro_use]
/// extern crate serde_derive;
///
/// pub mod foo {
///   op_newtype! {
///       /// A is private
///       A(u128)
///   }
///
///   op_newtype! {
///       /// B is public
///       #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
///       pub B(u128)
///   }
///
///   op_newtype! {
///       /// C and the underlying value are public
///       #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
///       pub C(pub u128)
///   }
///
/// }
///
/// # fn main() {
///  // will not compile because foo::A is private
///  // let _ = foo::A::new(1);
///
///  // the `new` function constructor is provided
///  let b1 = foo::B::new(1);
///  // From trait is implemented
///  let _ = foo::B::from(1);
///  let b2: foo::B = 1.into();
///  // this works because PartialEq was derived for the foo::B
///  assert_eq!(b1,b2);
///
///  // foo::C can be declared as consts because its underlying field is also public
///  const C_1 : foo::C = foo::C(1);
///  const C_2 : foo::C = foo::C(2);
///
///  // this works because Deref is implemented and foo::C and foo::B both have the same underlying type
///  assert_eq!(*C_1,*b1);
///  assert!(C_1 < C_2);
/// # }
///
/// ```
///  
#[macro_export]
macro_rules! op_newtype {
    (
        $(#[$outer:meta])*
        $Name:ident($T:ty)
    ) => {
        op_newtype!{
            $(#[$outer])*
            () $Name(() $T)
        }
    };
    (
        $(#[$outer:meta])*
        pub $Name:ident($T:ty)
    ) => {
        op_newtype!{
            $(#[$outer])*
            (pub) $Name(() $T)
        }
    };
    (
        $(#[$outer:meta])*
        pub $Name:ident(pub $T:ty)
    ) => {
        op_newtype!{
            $(#[$outer])*
            (pub) $Name((pub) $T)
        }
    };
    (
        $(#[$outer:meta])*
        ($($struct_vis:tt)*) $Name:ident(($($field_vis:tt)*) $T:ty)
    ) => {
        $(#[$outer])*
        #[derive(Debug)]
        $($struct_vis)* struct $Name($($field_vis)* $T);

        op_tt_as_item! {
            impl $Name {
                pub fn new(value: $T) -> Self {
                    $Name(value)
                }
            }
        }

        op_tt_as_item! {
            impl ::std::ops::Deref for $Name {
                type Target = $T;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }

        op_tt_as_item! {
            impl From<$T> for $Name {
                fn from(value: $T) -> $Name {$Name(value)}
            }
        }
    };
}

#[cfg(test)]
mod tests {

    use serde_json;
    use tests;

    pub mod foo {
        op_newtype!{
            /// A is private
            A(u128)
        }

        op_newtype!{
            /// B is public
            pub B(u128)
        }

        op_newtype!{
            /// C and the underlying value are public
            #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
            pub C(pub u128)
        }
    }

    #[test]
    fn newtype_private() {
        tests::run_test(|| {
            // will not compile because foo::A is private
            // let _ = foo::A::new(1);

            let b = foo::B::new(1);
            info!("b = {:?}", b);

            let b: foo::B = 1.into();
            assert_eq!(*b, 1);

            const C_1: foo::C = foo::C(1);
            const C_2: foo::C = foo::C(2);

            assert_eq!(*C_1, *b);
            assert!(C_1 < C_2);
            info!("C_1 as json: {}", serde_json::to_string(&C_1).unwrap());
            let c: foo::C = serde_json::from_str(&serde_json::to_string(&C_1).unwrap()).unwrap();
            assert_eq!(foo::C(1), c);
        });
    }

}
