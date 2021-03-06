//! A lightweight library for displaying errors and their sources.
//!
//! A sample output:
//!
//! ```rust
//! macro_rules! impl_error {
//!     // ...
//! #    ($ty:ty, $display:expr, $source:expr) => {
//! #        impl ::std::fmt::Display for $ty {
//! #            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//! #                write!(f, "{}", $display)
//! #            }
//! #        }
//! #
//! #        impl ::std::error::Error for $ty {
//! #            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
//! #                $source
//! #            }
//! #        }
//! #    };
//! }
//!
//! // `TopLevel` is caused by a `MidLevel`.
//! #[derive(Debug)]
//! struct TopLevel;
//! impl_error!(TopLevel, "top level", Some(&MidLevel));
//!
//! // `MidLevel` is caused by a `LowLevel`.
//! #[derive(Debug)]
//! struct MidLevel;
//! impl_error!(MidLevel, "mid level", Some(&LowLevel));
//!
//! // `LowLevel` is the cause itself.
//! #[derive(Debug)]
//! struct LowLevel;
//! impl_error!(LowLevel, "low level", None);
//!
//! // Now let's see how it works:
//! let formatted = display_error_chain::DisplayErrorChain::new(&TopLevel).to_string();
//! assert_eq!(
//!     formatted,
//!     "\
//!top level
//!Caused by:
//!   -> mid level
//!   -> low level"
//! );
//! ```

use std::{error::Error, fmt};

/// Provides an [fmt::Display] implementation for an error as a chain.
///
/// ```rust
/// use display_error_chain::DisplayErrorChain;
///
/// // Let's set up a custom error. Normally one would use `snafu` or
/// // something similar to avoid the boilerplate.
/// #[derive(Debug)]
/// enum CustomError {
///     NoCause,
///     IO { source: std::io::Error },
/// }
///
/// // Custom display implementation (which doesn't take error
/// // sources into consideration).
/// impl std::fmt::Display for CustomError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             CustomError::NoCause => {
///                 write!(f, "No cause")
///             }
///             CustomError::IO { .. } => {
///                 write!(f, "Some I/O")
///             }
///         }
///     }
/// }
///
/// impl std::error::Error for CustomError {
///     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
///         match self {
///             CustomError::NoCause => None,
///             CustomError::IO { source } => Some(source),
///         }
///     }
/// }
///
/// // And finally let's see how `DisplayErrorChain` helps!
/// let io = CustomError::IO {
///     source: std::io::Error::new(std::io::ErrorKind::AlreadyExists, "wow"),
/// };
/// let formatted = DisplayErrorChain::new(&io).to_string();
/// assert_eq!("Some I/O\nCaused by:\n  -> wow", formatted);
///
/// let no_cause = CustomError::NoCause;
/// let formatted = DisplayErrorChain::new(&no_cause).to_string();
/// assert_eq!("No cause", formatted);
/// ```
pub struct DisplayErrorChain<'a, E: ?Sized>(&'a E);

impl<'a, E> DisplayErrorChain<'a, E>
where
    E: Error + ?Sized,
{
    /// Initializes the formatter with the error provided.
    pub fn new(error: &'a E) -> Self {
        DisplayErrorChain(error)
    }
}

impl<'a, E> fmt::Display for DisplayErrorChain<'a, E>
where
    E: Error + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;

        let mut cause_printed = false;
        let mut source = self.0.source();
        while let Some(cause) = source {
            if !cause_printed {
                cause_printed = true;
                writeln!(f, "\nCaused by:")?;
            } else {
                writeln!(f)?
            }
            write!(f, "  -> {}", cause)?;
            source = cause.source();
        }
        Ok(())
    }
}
