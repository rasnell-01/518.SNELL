#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafeFloat {
    /// A valid floating-point value.
    Value(f64),
    /// A prior computation became undefined (NaN / ±Inf / div-by-zero).
    NaN,
}

impl SafeFloat {

    /// Wrap a raw f64.  If the value is already NaN or infinite
    /// we immediately represent it as SafeFloat::NaN.
    pub fn of(x: f64) -> Self {
        if x.is_nan() || x.is_infinite() {
            SafeFloat::NaN
        } else {
            SafeFloat::Value(x)
        }
    }

    /// Apply a pure function to the inner value.
    /// If we are already NaN the function is never called.
    pub fn map<F: Fn(f64) -> f64>(self, f: F) -> Self {
        match self {
            SafeFloat::NaN      => SafeFloat::NaN,
            SafeFloat::Value(x) => SafeFloat::of(f(x)),
        }
    }

    /// Sequence a computation that may itself produce NaN.
    /// Classic monadic bind: if we are already NaN, short-circuit;
    /// otherwise run `f` and return its result (which may be NaN).
    pub fn and_then<F: Fn(f64) -> SafeFloat>(self, f: F) -> Self {
        match self {
            SafeFloat::NaN      => SafeFloat::NaN,
            SafeFloat::Value(x) => f(x),
        }
    }

    pub fn unwrap_or(self, default: f64) -> f64 {
        match self {
            SafeFloat::Value(x) => x,
            SafeFloat::NaN      => default,
        }
    }
}

/// Safe division — returns NaN when dividing by zero.
pub fn safe_div(numerator: f64) -> impl Fn(f64) -> SafeFloat {
    move |denominator: f64| {
        if denominator == 0.0 {
            SafeFloat::NaN
        } else {
            SafeFloat::of(numerator / denominator)
        }
    }
}

/// Safe square root — returns NaN for negative inputs.
pub fn safe_sqrt(x: f64) -> SafeFloat {
    if x < 0.0 {
        SafeFloat::NaN
    } else {
        SafeFloat::of(x.sqrt())
    }
}

/// Safe natural logarithm — returns NaN for non-positive inputs.
pub fn safe_ln(x: f64) -> SafeFloat {
    if x <= 0.0 {
        SafeFloat::NaN
    } else {
        SafeFloat::of(x.ln())
    }
}