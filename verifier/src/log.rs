macro_rules! trace {
    ($( $args:tt )*) => {
        #[cfg(feature = "std")]
        log::trace!($( $args )*);
    }
}

macro_rules! debug {
    ($( $args:tt )*) => {
        #[cfg(feature = "std")]
        log::debug!($( $args )*);
    }
}

macro_rules! info {
    ($( $args:tt )*) => {
        #[cfg(feature = "std")]
        log::info!($( $args )*);
    }
}

macro_rules! warn {
    ($( $args:tt )*) => {
        #[cfg(feature = "std")]
        log::warn!($( $args )*);
    }
}

macro_rules! error {
    ($( $args:tt )*) => {
        #[cfg(feature = "std")]
        log::error!($( $args )*);
    }
}

#[cfg(feature = "std")]
macro_rules! log_if_enabled {
    (|$level:tt | $block_expr:block) => {
        if log::log_enabled!(log::Level::$level) $block_expr
    };
}

#[cfg(feature = "no-std")]
macro_rules! log_if_enabled {
    (|$level:tt | $block_expr:block) => {};
}
