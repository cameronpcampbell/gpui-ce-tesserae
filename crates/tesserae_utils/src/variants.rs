use gpui::{Element, StatefulInteractiveElement, Styled};

pub trait Variant {
    type Data<'a>;

    fn apply<'a, E: Element + Styled + StatefulInteractiveElement>(
        &self,
        element: E,
        data: Self::Data<'a>,
    ) -> E;
}

#[macro_export]
macro_rules! variants {
    (
        $visibility:vis $variant_name:ident<( $( &$data_type:ty ),+ $(,)? )> {
            $($variants:tt)*
        }
    ) => {
        $crate::variants! {
            @impl $visibility $variant_name [($( &'data $data_type ),+)] {
                $($variants)*
            }
        }
    };
    (
        $visibility:vis $variant_name:ident<&$data_type:ty> {
            $($variants:tt)*
        }
    ) => {
        $crate::variants! {
            @impl $visibility $variant_name [&'data $data_type] {
                $($variants)*
            }
        }
    };
    (
        $visibility:vis $variant_name:ident<$data_type:ty> {
            $($variants:tt)*
        }
    ) => {
        $crate::variants! {
            @impl $visibility $variant_name [$data_type] {
                $($variants)*
            }
        }
    };
    (
        @impl $visibility:vis $variant_name:ident [$($data_type:tt)+] {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident ($this:ident, $data:pat_param) => $body:expr
            ),+ $(,)?
        }
    ) => {
        #[derive(Default)]
        $visibility enum $variant_name {
            $(
                $(#[$variant_attr])*
                $variant,
            )+
        }

        impl $crate::Variant for $variant_name {
            type Data<'data> = $($data_type)+;

            fn apply<'data, E: $crate::__gpui::Element + $crate::__gpui::Styled + $crate::__gpui::StatefulInteractiveElement>(
                &self,
                element: E,
                data: Self::Data<'data>,
            ) -> E {
                match self {
                    $(
                        Self::$variant => {
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
