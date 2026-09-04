#[doc(hidden)]
pub mod __macro_exports {
    pub use gpui::Styled;
}

use __macro_exports::Styled;

pub trait Kind<E: Styled> {
    type Data<'a>;

    fn apply<'a>(&self, element: E, data: Self::Data<'a>) -> E;
}

#[macro_export]
macro_rules! kinds {
    (
        $visibility:vis $kind_name:ident<_, ($($data_types:tt)+)> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @collect
            [
                @impl generic [
                    $crate::kinds::__macro_exports::Styled
                ] $visibility $kind_name
            ]
            []
            [$($kinds)*]
            $($data_types)+
        }
    };
    (
        $visibility:vis $kind_name:ident<_, &$data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl generic [
                $crate::kinds::__macro_exports::Styled
            ] $visibility $kind_name [&'data $data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<_, $data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl generic [
                $crate::kinds::__macro_exports::Styled
            ] $visibility $kind_name [$data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<
            $first_element_bound:ident $(:: $first_element_bound_tail:ident)*
                $(+ $additional_element_bound:ident $(:: $additional_element_bound_tail:ident)*)+,
            ($($data_types:tt)+)
        > {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @collect
            [
                @impl generic [
                    $first_element_bound $(:: $first_element_bound_tail)*
                        $(+ $additional_element_bound $(:: $additional_element_bound_tail)*)+
                ] $visibility $kind_name
            ]
            []
            [$($kinds)*]
            $($data_types)+
        }
    };
    (
        $visibility:vis $kind_name:ident<
            $first_element_bound:ident $(:: $first_element_bound_tail:ident)*
                $(+ $additional_element_bound:ident $(:: $additional_element_bound_tail:ident)*)+,
            &$data_type:ty
        > {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl generic [
                $first_element_bound $(:: $first_element_bound_tail)*
                    $(+ $additional_element_bound $(:: $additional_element_bound_tail)*)+
            ] $visibility $kind_name [&'data $data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<
            $first_element_bound:ident $(:: $first_element_bound_tail:ident)*
                $(+ $additional_element_bound:ident $(:: $additional_element_bound_tail:ident)*)+,
            $data_type:ty
        > {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl generic [
                $first_element_bound $(:: $first_element_bound_tail)*
                    $(+ $additional_element_bound $(:: $additional_element_bound_tail)*)+
            ] $visibility $kind_name [$data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<$element_type:ty, ($($data_types:tt)+)> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @collect
            [@impl concrete [$element_type] $visibility $kind_name]
            []
            [$($kinds)*]
            $($data_types)+
        }
    };
    (
        $visibility:vis $kind_name:ident<$element_type:ty, &$data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl concrete [$element_type] $visibility $kind_name [&'data $data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<$element_type:ty, $data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl concrete [$element_type] $visibility $kind_name [$data_type] {
                $($kinds)*
            }
        }
    };
    (
        @collect
        [$($callback:tt)*]
        [$($collected:tt)*]
        [$($kinds:tt)*]
        &$data_type:ty, $($remaining:tt)+
    ) => {
        $crate::kinds! {
            @collect
            [$($callback)*]
            [$($collected)* &'data $data_type,]
            [$($kinds)*]
            $($remaining)+
        }
    };
    (
        @collect
        [$($callback:tt)*]
        [$($collected:tt)*]
        [$($kinds:tt)*]
        $data_type:ty, $($remaining:tt)+
    ) => {
        $crate::kinds! {
            @collect
            [$($callback)*]
            [$($collected)* $data_type,]
            [$($kinds)*]
            $($remaining)+
        }
    };
    (
        @collect
        [$($callback:tt)*]
        [$($collected:tt)*]
        [$($kinds:tt)*]
        &$data_type:ty $(,)?
    ) => {
        $crate::kinds! {
            $($callback)* [($($collected)* &'data $data_type)] {
                $($kinds)*
            }
        }
    };
    (
        @collect
        [$($callback:tt)*]
        [$($collected:tt)*]
        [$($kinds:tt)*]
        $data_type:ty $(,)?
    ) => {
        $crate::kinds! {
            $($callback)* [($($collected)* $data_type)] {
                $($kinds)*
            }
        }
    };
    (
        @impl generic [$($element_bounds:tt)+] $visibility:vis $kind_name:ident [$($data_type:tt)+] {
            $(
                $(#[$kind_attr:meta])*
                $kind:ident $(| $additional_kind:ident)* ($this:ident, $data:pat_param) => $body:expr
            ),+ $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Default)]
        $visibility enum $kind_name {
            $(
                $(#[$kind_attr])*
                $kind,
                $(
                    $additional_kind,
                )*
            )+
        }

        impl<E> $crate::Kind<E> for $kind_name
        where
            E: $($element_bounds)+,
        {
            type Data<'data> = $($data_type)+;

            fn apply<'data>(
                &self,
                element: E,
                data: Self::Data<'data>,
            ) -> E {
                match self {
                    $(
                        Self::$kind $(| Self::$additional_kind)* => {
                            let $this = element;
                            let $data = data;
                            $body
                        }
                    )+
                }
            }
        }
    };
    (
        @impl concrete [$element_type:ty] $visibility:vis $kind_name:ident [$($data_type:tt)+] {
            $(
                $(#[$kind_attr:meta])*
                $kind:ident $(| $additional_kind:ident)* ($this:ident, $data:pat_param) => $body:expr
            ),+ $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Default)]
        $visibility enum $kind_name {
            $(
                $(#[$kind_attr])*
                $kind,
                $(
                    $additional_kind,
                )*
            )+
        }

        impl $crate::Kind<$element_type> for $kind_name {
            type Data<'data> = $($data_type)+;

            fn apply<'data>(
                &self,
                element: $element_type,
                data: Self::Data<'data>,
            ) -> $element_type {
                match self {
                    $(
                        Self::$kind $(| Self::$additional_kind)* => {
                            let $this = element;
                            let $data = data;
                            $body
                        }
                    )+
                }
            }
        }
    };
}
