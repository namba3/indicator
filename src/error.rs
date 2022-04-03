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
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            InvalidUintRange(e) => f.write_fmt(format_args!("invalid uint range: {e}")),
            InvalidFloatRange(e) => f.write_fmt(format_args!("invalid float range: {e}")),
            InvalidRelation(e) => f.write_fmt(format_args!("invalid relation: {e}")),
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

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
