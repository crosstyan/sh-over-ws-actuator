
use anyhow::{Result, Context};
/// Helper trait to convert error types that don't satisfy `anyhow`s trait requirements to
/// anyhow errors.
pub trait ToAnyhow<U> {
    fn to_anyhow(self) -> crate::anyhow::Result<U>;
}

impl<U> ToAnyhow<U> for Result<U, std::sync::PoisonError<U>> {
    fn to_anyhow(self) -> crate::anyhow::Result<U> {
        match self {
            Ok(val) => crate::anyhow::Ok(val),
            Err(e) => {
              Err(crate::anyhow::anyhow!(
                  "cannot acquire poisoned lock for {e:#?}"
              ))
            },
        }
    }
}
pub trait FatalError<T> {
    /// Mark results as being non-fatal.
    ///
    /// If the result is an `Err` variant, this will [print the error to the log][`to_log`].
    /// Discards the result type afterwards.
    ///
    /// [`to_log`]: LoggableError::to_log
    #[track_caller]
    fn non_fatal(self);

    /// Mark results as being fatal.
    ///
    /// If the result is an `Err` variant, this will unwrap the error and panic the application.
    /// If the result is an `Ok` variant, the inner value is unwrapped and returned instead.
    ///
    /// # Panics
    ///
    /// If the given result is an `Err` variant.
    #[track_caller]
    fn fatal(self) -> T;
}

/// Helper function to silence `#[warn(unused_must_use)]` cargo warnings. Used exclusively in
/// `FatalError::non_fatal`!
fn discard_result<T>(_arg: anyhow::Result<T>) {}

impl<T> FatalError<T> for anyhow::Result<T> {
    fn non_fatal(self) {
        if self.is_err() {
            discard_result(self.context("a non-fatal error occured").to_log());
        }
    }

    fn fatal(self) -> T {
        if let Ok(val) = self {
            val
        } else {
            self.context("a fatal error occured")
                .expect("Program terminates")
        }
    }
}

/// Helper trait to easily log error types.
///
/// The `print_error` function takes a closure which takes a `&str` and fares with it as necessary
/// to log the error to some usable location. For convenience, logging to stdout, stderr and
/// `log::error!` is already implemented.
///
/// Note that the trait functions pass the error through unmodified, so they can be chained with
/// the usual handling of [`std::result::Result`] types.
pub trait LoggableError<T>: Sized {
    /// Gives a formatted error message derived from `self` to the closure `fun` for
    /// printing/logging as appropriate.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use anyhow;
    /// use zellij_utils::errors::LoggableError;
    ///
    /// let my_err: anyhow::Result<&str> = Err(anyhow::anyhow!("Test error"));
    /// my_err
    ///     .print_error(|msg| println!("{msg}"))
    ///     .unwrap();
    /// ```
    #[track_caller]
    fn print_error<F: Fn(&str)>(self, fun: F) -> Self;

    /// Convenienve function, calls `print_error` and logs the result as error.
    ///
    /// This is not a wrapper around `log::error!`, because the `log` crate uses a lot of compile
    /// time macros from `std` to determine caller locations/module names etc. Since these are
    /// resolved at compile time in the location they are written, they would always resolve to the
    /// location in this function where `log::error!` is called, masking the real caller location.
    /// Hence, we build the log message ourselves. This means that we lose the information about
    /// the calling module (Because it can only be resolved at compile time), however the callers
    /// file and line number are preserved.
    #[track_caller]
    fn to_log(self) -> Self {
        let caller = std::panic::Location::caller();
        self.print_error(|msg| {
            // Build the log entry manually
            // NOTE: The log entry has no module path associated with it. This is because `log`
            // gets the module path from the `std::module_path!()` macro, which is replaced at
            // compile time in the location it is written!
            log::logger().log(
                &log::Record::builder()
                    .level(log::Level::Error)
                    .args(format_args!("{}", msg))
                    .file(Some(caller.file()))
                    .line(Some(caller.line()))
                    .module_path(None)
                    .build(),
            );
        })
    }

    /// Convenienve function, calls `print_error` with the closure `|msg| eprintln!("{}", msg)`.
    fn to_stderr(self) -> Self {
        self.print_error(|msg| eprintln!("{}", msg))
    }

    /// Convenienve function, calls `print_error` with the closure `|msg| println!("{}", msg)`.
    fn to_stdout(self) -> Self {
        self.print_error(|msg| println!("{}", msg))
    }
}

impl<T> LoggableError<T> for anyhow::Result<T> {
    fn print_error<F: Fn(&str)>(self, fun: F) -> Self {
        if let Err(ref err) = self {
            fun(&format!("{:?}", err));
        }
        self
    }
}
