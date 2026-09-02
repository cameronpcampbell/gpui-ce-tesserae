use gpui::{Element, StatefulInteractiveElement, Styled};

pub trait Kind {
    type Data<'a>;

    fn apply<'a, E: Element + Styled + StatefulInteractiveElement>(
        &self,
        element: E,
        data: Self::Data<'a>,
    ) -> E;
}

#[macro_export]
macro_rules! kinds {
    (
        $visibility:vis $kind_name:ident<( $( &$data_type:ty ),+ $(,)? )> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl $visibility $kind_name [($( &'data $data_type ),+)] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<&$data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl $visibility $kind_name [&'data $data_type] {
                $($kinds)*
            }
        }
    };
    (
        $visibility:vis $kind_name:ident<$data_type:ty> {
            $($kinds:tt)*
        }
    ) => {
        $crate::kinds! {
            @impl $visibility $kind_name [$data_type] {
                $($kinds)*
            }
        }
    };
    (
        @impl $visibility:vis $kind_name:ident [$($data_type:tt)+] {
            $(
                $(#[$kind_attr:meta])*
                $kind:ident ($this:ident, $data:pat_param) => $body:expr
            ),+ $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Default)]
        $visibility enum $kind_name {
            $(
                $(#[$kind_attr])*
                $kind,
            )+
        }

        impl $crate::Kind for $kind_name {
            type Data<'data> = $($data_type)+;

            fn apply<'data, E: $crate::__gpui::Element + $crate::__gpui::Styled + $crate::__gpui::StatefulInteractiveElement>(
                &self,
                element: E,
                data: Self::Data<'data>,
            ) -> E {
                match self {
                    $(
                        Self::$kind => {
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
