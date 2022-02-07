use std::fmt;
use std::fmt::{Display, Formatter};
use crate::pla::command::PlaCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaParseError {
    pub message: String,
}

impl Display for PlaParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PlaParseError: {}", self.message)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaSubBlockConversionError {
    pub initial_type: PlaCommand
}

impl Display for PlaSubBlockConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to convert from a PlaSubBlock to the requested concrete type: {:?}", self.initial_type)
    }
}

#[cfg(test)]
mod tests {
    use crate::pla::command::PlaCommand;
    use crate::pla::error::{PlaSubBlockConversionError, PlaParseError};

    fn get_error() -> Result<u32, PlaParseError> {
        Err(
            PlaParseError {
                message: String::from("I've got a caribbean soul I can barely control")
            }
        )
    }

    fn get_conversion_error() -> Result<u32, PlaSubBlockConversionError> {
        Err(
            PlaSubBlockConversionError {
                initial_type: PlaCommand::DURATION
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

    #[test]
    fn it_should_be_able_to_create_a_pla_sub_block_conversion_error() {
        let error: PlaSubBlockConversionError = PlaSubBlockConversionError {
            initial_type: PlaCommand::DURATION
        };

        assert_eq!(format!("Unable to convert from a PlaSubBlock to the requested concrete type: DURATION"), format!("{}", error));
    }

    #[test]
    #[should_panic]
    fn it_should_be_able_to_throw_a_pla_sub_block_conversion_error() {
        let res = get_conversion_error();
        res.unwrap();
    }
}