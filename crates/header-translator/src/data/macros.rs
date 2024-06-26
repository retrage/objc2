/// See `README.md` for details on this macro.
macro_rules! data {
    // Parse module declarations
    (
        $(
            mod $module:ident;
        )+
    ) => {
        $(
            #[allow(non_snake_case)]
            mod $module;
        )+

        #[allow(unused_variables)]
        pub(crate) fn apply_tweaks(data: &mut $crate::config::LibraryConfig) {
            $(
                if data.framework == stringify!($module) {
                    $module::apply_tweaks(data);
                }
            )+
        }
    };
    ($($rest:tt)*) => {
        #[allow(unused_variables)]
        pub(crate) fn apply_tweaks(config: &mut $crate::config::LibraryConfig) {
            __data_inner! {
                @(config)
                $($rest)*
            }
        }
    };
}

macro_rules! __set_mutability {
    ($data:expr;) => {};
    ($data:expr; ImmutableWithMutableSubclass<$framework:ident::$file_name:ident::$subclass:ident>) => {
        $data.mutability = $crate::stmt::Mutability::ImmutableWithMutableSubclass(
            $crate::ItemIdentifier::from_raw(
                stringify!($subclass).to_string(),
                stringify!($framework).to_string(),
                stringify!($file_name).to_string(),
            ),
        );
    };
    ($data:expr; MutableWithImmutableSuperclass<$framework:ident::$file_name:ident::$superclass:ident>) => {
        $data.mutability = $crate::stmt::Mutability::MutableWithImmutableSuperclass(
            $crate::ItemIdentifier::from_raw(
                stringify!($superclass).to_string(),
                stringify!($framework).to_string(),
                stringify!($file_name).to_string(),
            ),
        );
    };
    ($data:expr; Immutable) => {
        $data.mutability = $crate::stmt::Mutability::Immutable;
    };
    ($data:expr; Mutable) => {
        $data.mutability = $crate::stmt::Mutability::Mutable;
    };
    ($data:expr; MainThreadOnly) => {
        $data.mutability = $crate::stmt::Mutability::MainThreadOnly;
    };
}

macro_rules! __data_inner {
    // Base case
    (
        @($config:expr)
    ) => {};
    // Class
    (
        @($config:expr)

        class $class:ident $(: $mutability:ident $(<$framework:ident::$file_name:ident::$ty:ident>)?)? {
            $($methods:tt)*
        }

        $($rest:tt)*
    ) => {
        #[allow(unused_mut)]
        let mut data = $config.class_data.entry(stringify!($class).to_string()).or_default();

        __set_mutability! {
            data;
            $($mutability $(<$framework::$file_name::$ty>)?)?
        }

        __data_methods! {
            @(data)
            $($methods)*
        }

        __data_inner! {
            @($config)
            $($rest)*
        }
    };
    // Protocol
    (
        @($config:expr)

        protocol $protocol:ident {
            $($methods:tt)*
        }

        $($rest:tt)*
    ) => {
        #[allow(unused_mut)]
        let mut data = $config.protocol_data.entry(stringify!($protocol).to_string()).or_default();

        __data_methods! {
            @(data)
            $($methods)*
        }

        __data_inner! {
            @($config)
            $($rest)*
        }
    };
    // Function
    (
        @($config:expr)

        unsafe fn $function:ident;

        $($rest:tt)*
    ) => {
        #[allow(unused_mut)]
        let mut data = $config.fns.entry(stringify!($function).to_string()).or_default();

        data.unsafe_ = false;

        __data_inner! {
            @($config)
            $($rest)*
        }
    }
}

macro_rules! __data_methods {
    // Base case
    (
        @($data:expr)
    ) => {};
    // Mark init (and by extension, `new`) method as safe
    (
        @($data:expr)

        unsafe -init;

        $($rest:tt)*
    ) => {
        #[allow(unused_mut)]
        let mut method_data = $data.methods.entry("init".to_string()).or_default();
        method_data.unsafe_ = false;

        #[allow(unused_mut)]
        let mut method_data = $data.methods.entry("new".to_string()).or_default();
        method_data.unsafe_ = false;

        __data_methods! {
            @($data)
            $($rest)*
        }
    };
    // Mark new (and by extension, `init`) method as safe
    (
        @($data:expr)

        unsafe +new;

        $($rest:tt)*
    ) => {
        __data_methods! {
            @($data)

            unsafe -init;

            $($rest)*
        }
    };
    // Mark method as safe
    (
        @($data:expr)

        unsafe -$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        let sel = concat!(stringify!($sel_base), $(':', $(stringify!($sel_ext), ':',)*)?);
        #[allow(unused_mut)]
        let mut method_data = $data.methods.entry(sel.to_string()).or_default();

        method_data.unsafe_ = false;

        __data_methods! {
            @($data)
            $($rest)*
        }
    };
    // Mark class method as safe
    (
        @($data:expr)

        unsafe +$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        // TODO
        __data_methods! {
            @($data)
            unsafe -$sel_base $(: $($sel_ext:)*)?;

            $($rest)*
        }
    };
    // Mark method as mutable
    (
        @($data:expr)

        mut -$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        let sel = concat!(stringify!($sel_base), $(':', $(stringify!($sel_ext), ':',)*)?);
        #[allow(unused_mut)]
        let mut method_data = $data.methods.entry(sel.to_string()).or_default();

        method_data.mutating = Some(true);

        __data_methods! {
            @($data)
            $($rest)*
        }
    };
    // Mark class method as mutable
    (
        @($data:expr)

        mut +$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        // TODO
        __data_methods! {
            @($data)
            mut -$sel_base $(: $($sel_ext:)*)?;

            $($rest)*
        }
    };
    // Mark method as safe and mutable
    (
        @($data:expr)

        unsafe mut -$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        let sel = concat!(stringify!($sel_base), $(':', $(stringify!($sel_ext), ':',)*)?);
        #[allow(unused_mut)]
        let mut method_data = $data.methods.entry(sel.to_string()).or_default();

        method_data.unsafe_ = false;
        method_data.mutating = Some(true);

        __data_methods! {
            @($data)
            $($rest)*
        }
    };
    // Mark class method as safe and mutable
    (
        @($data:expr)

        unsafe mut +$sel_base:ident $(: $($sel_ext:ident:)*)?;

        $($rest:tt)*
    ) => {
        // TODO
        __data_methods! {
            @($data)
            unsafe mut -$sel_base $(: $($sel_ext:)*)?;

            $($rest)*
        }
    };
}
