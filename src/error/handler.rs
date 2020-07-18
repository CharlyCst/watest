struct Error {
    fun: String,
    message: String,
}

pub struct ErrorHandler {
    has_error: bool,
    errors: Vec<Error>,
}

/// An error handler is responsible for storing errors and print them on demand.
impl ErrorHandler {
    pub fn new() -> ErrorHandler {
        ErrorHandler {
            has_error: false,
            errors: Vec::new(),
        }
    }

    /// Return `true` if at leas one error has been reported.
    pub fn has_error(&self) -> bool {
        self.has_error
    }

    /// Report an error.
    pub fn report(&mut self, fun: String, message: String) {
        self.has_error = true;
        self.errors.push(Error { fun, message });
    }

    /// Print all errors previously reported.
    pub fn print(&self) {
        for error in &self.errors {
            println!("In {}: {}", error.fun, error.message);
        }
    }
}

