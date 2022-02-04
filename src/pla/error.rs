use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaParseError {
    pub message: String,
}

impl Display for PlaParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PlaParseError: {}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};
    use crate::pla::error::PlaParseError;

    fn get_error() -> Result<u32, PlaParseError> {
        Err(
            PlaParseError {
                message: String::from("I've got a caribbean soul I can barely control")
            }
        )
    }

    #[test]
    fn it_should_be_able_to_create_a_pla_parse_error() {
        let error: PlaParseError = PlaParseError {
            message: String::from("I've got a caribbean soul I can barely control")
        };

        assert_eq!(format!("PlaParseError: {}", error.message), format!("{}", error))
    }

    #[test]
    #[should_panic]
    fn it_should_be_able_to_throw_a_pla_parse_error() {
        let res = get_error();
        res.unwrap();
    }
}