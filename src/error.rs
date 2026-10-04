use core::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum Range<T> {
    LowerBounded { min: T },
    UpperBounded { max: T },
    BothBounded { min: T, max: T },
}

#[derive(Debug, Clone)]
pub struct Parameter<T: Display> {
    pub(crate) name: &'static str,
    pub(crate) value: T,
}
impl<T: Display> Parameter<T> {
    pub(crate) fn new(name: &'static str, value: T) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone)]
pub struct InvalidRangeError<T: Display> {
    pub(crate) param: Parameter<T>,
    pub(crate) range: Range<T>,
}
impl<T: Display> InvalidRangeError<T> {
    pub(crate) fn new(name: &'static str, value: T, range: Range<T>) -> Self {
        Self {
            param: Parameter { name, value },
            range,
        }
    }
}
impl<T: Display> Display for InvalidRangeError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Range::*;
        let Self {
            param: Parameter { name, value },
            range,
        } = self;

        match range {
            LowerBounded { min } => f.write_fmt(format_args!(
                "expected to be {min} <= {name}, but actually {value}."
            )),
            UpperBounded { max } => f.write_fmt(format_args!(
                "expected to be {name} <= {max}, but actually {value}."
            )),
            BothBounded { min, max } => f.write_fmt(format_args!(
                "expected to be {min} <= {name} <= {max}, but actually {value}."
            )),
        }
    }
}
#[cfg(feature = "std")]
impl<T: Debug + Display> std::error::Error for InvalidRangeError<T> {}

#[derive(Debug, Clone)]
pub struct InvalidPriceError {
    pub(crate) price: f64,
}
impl InvalidPriceError {
    pub(crate) fn new(price: f64) -> Self {
        Self { price }
    }
}
impl Display for InvalidPriceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "expected price to be finite, but actually {}.",
            self.price
        ))
    }
}
#[cfg(feature = "std")]
impl std::error::Error for InvalidPriceError {}

#[derive(Debug, Clone)]
pub struct InvalidVolumeError {
    pub(crate) volume: f64,
}
impl InvalidVolumeError {
    pub(crate) fn new(volume: f64) -> Self {
        Self { volume }
    }
}
impl Display for InvalidVolumeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "expected volume to be finite and non-negative, but actually {}.",
            self.volume
        ))
    }
}
#[cfg(feature = "std")]
impl std::error::Error for InvalidVolumeError {}

#[derive(Debug, Clone)]
pub struct InvalidBinaryRelationError<T: Display> {
    pub(crate) operator: &'static str,
    pub(crate) lhs: Parameter<T>,
    pub(crate) rhs: Parameter<T>,
}
impl<T: Display> Display for InvalidBinaryRelationError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let Self {
            operator: op,
            lhs:
                Parameter {
                    name: lname,
                    value: lvalue,
                },
            rhs:
                Parameter {
                    name: rname,
                    value: rvalue,
                },
        } = self;
        f.write_fmt(format_args!(
            "expected to be {lname} {op} {rname}, found {lvalue} {op} {rvalue}."
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidUintRange(InvalidRangeError<usize>),
    InvalidFloatRange(InvalidRangeError<f64>),
    InvalidRelation(InvalidBinaryRelationError<usize>),
    InvalidPrice(InvalidPriceError),
    InvalidVolume(InvalidVolumeError),
    AllocationFailed,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            InvalidUintRange(e) => f.write_fmt(format_args!("invalid uint range: {e}")),
            InvalidFloatRange(e) => f.write_fmt(format_args!("invalid float range: {e}")),
            InvalidRelation(e) => f.write_fmt(format_args!("invalid relation: {e}")),
            InvalidPrice(e) => f.write_fmt(format_args!("invalid price: {e}")),
            InvalidVolume(e) => f.write_fmt(format_args!("invalid volume: {e}")),
            AllocationFailed => f.write_str("allocation failed"),
        }
    }
}
impl From<InvalidRangeError<usize>> for Error {
    fn from(e: InvalidRangeError<usize>) -> Self {
        Self::InvalidUintRange(e)
    }
}
impl From<InvalidRangeError<f64>> for Error {
    fn from(e: InvalidRangeError<f64>) -> Self {
        Self::InvalidFloatRange(e)
    }
}
impl From<InvalidBinaryRelationError<usize>> for Error {
    fn from(e: InvalidBinaryRelationError<usize>) -> Self {
        Self::InvalidRelation(e)
    }
}
impl From<InvalidPriceError> for Error {
    fn from(e: InvalidPriceError) -> Self {
        Self::InvalidPrice(e)
    }
}
impl From<InvalidVolumeError> for Error {
    fn from(e: InvalidVolumeError) -> Self {
        Self::InvalidVolume(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_range_errors_format_each_bound_shape() {
        let lower = InvalidRangeError::new("period", 0, Range::LowerBounded { min: 1 });
        let upper = InvalidRangeError::new("value", 3.0, Range::UpperBounded { max: 2.0 });
        let both = InvalidRangeError::new("value", 5, Range::BothBounded { min: 1, max: 4 });

        assert_eq!(
            lower.to_string(),
            "expected to be 1 <= period, but actually 0."
        );
        assert_eq!(
            upper.to_string(),
            "expected to be value <= 2, but actually 3."
        );
        assert_eq!(
            both.to_string(),
            "expected to be 1 <= value <= 4, but actually 5."
        );
    }

    #[test]
    fn invalid_binary_relation_error_formats_operands() {
        let error = InvalidBinaryRelationError {
            operator: "<",
            lhs: Parameter::new("short_period", 2),
            rhs: Parameter::new("long_period", 2),
        };

        assert_eq!(
            error.to_string(),
            "expected to be short_period < long_period, found 2 < 2."
        );
    }

    #[test]
    fn error_formats_each_wrapped_error_kind() {
        let uint_range = Error::from(InvalidRangeError::new(
            "period",
            0,
            Range::LowerBounded { min: 1 },
        ));
        let float_range = Error::from(InvalidRangeError::new(
            "value",
            3.0,
            Range::UpperBounded { max: 2.0 },
        ));
        let relation = Error::from(InvalidBinaryRelationError {
            operator: "<",
            lhs: Parameter::new("short_period", 3),
            rhs: Parameter::new("long_period", 2),
        });
        let volume = Error::from(InvalidVolumeError::new(-1.0));
        let price = Error::from(InvalidPriceError::new(f64::NAN));

        assert_eq!(
            uint_range.to_string(),
            "invalid uint range: expected to be 1 <= period, but actually 0."
        );
        assert_eq!(
            float_range.to_string(),
            "invalid float range: expected to be value <= 2, but actually 3."
        );
        assert_eq!(
            relation.to_string(),
            "invalid relation: expected to be short_period < long_period, found 3 < 2."
        );
        assert_eq!(
            volume.to_string(),
            "invalid volume: expected volume to be finite and non-negative, but actually -1."
        );
        assert_eq!(
            price.to_string(),
            "invalid price: expected price to be finite, but actually NaN."
        );
        assert_eq!(Error::AllocationFailed.to_string(), "allocation failed");
    }
}
