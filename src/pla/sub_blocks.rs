use std::fmt::{Debug, Display, Formatter};
use chrono::NaiveDate;
use mopa::mopafy;
use regex::Regex;
use crate::pla::command::PlaCommand;
use crate::pla::error::{PlaConversionError, PlaParseError};
use crate::pla::parser::{HeirarchicalPlaLine, PlaLine};

use dyn_clone::{clone_trait_object, DynClone};

pub trait PlaSubBlock: mopa::Any + DynClone {
    fn get_command(&self) -> PlaCommand;
    fn get_parent_id(&self) -> u32;
}

clone_trait_object!(PlaSubBlock);

mopafy!(PlaSubBlock);

impl std::fmt::Debug for Box<dyn PlaSubBlock> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get_command())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaStartBlock {
    pub parent_id: u32,
    pub date: NaiveDate,
    pub hour: u32,
}

impl PlaSubBlock for PlaStartBlock {
    fn get_command(&self) -> PlaCommand {
        PlaCommand::START
    }

    fn get_parent_id(&self) -> u32 {
        self.parent_id
    }
}

impl TryFrom<&Box<dyn PlaSubBlock>> for PlaStartBlock {
    type Error = PlaConversionError;

    fn try_from(value: &Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::START {
            return Err(PlaConversionError {
                initial_type: PlaCommand::START
            });
        }

        let converted_opt: Option<&PlaStartBlock> = value.downcast_ref::<PlaStartBlock>();

        if converted_opt.is_none() {
            return Err(PlaConversionError {
                initial_type: PlaCommand::START
            });
        }

        let dc_ref = converted_opt.unwrap();
        Ok(PlaStartBlock {
            parent_id: dc_ref.parent_id,
            date: dc_ref.date.clone(),
            hour: dc_ref.hour
        })
    }
}

impl TryFrom<Box<dyn PlaSubBlock>> for PlaStartBlock {
    type Error = PlaConversionError;

    fn try_from(value: Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::START {
            return Err(PlaConversionError {
                initial_type: PlaCommand::START
            });
        }

        let converted_opt: Option<&PlaStartBlock> = value.downcast_ref::<PlaStartBlock>();

        if converted_opt.is_none() {
            return Err(PlaConversionError {
                initial_type: PlaCommand::START
            });
        }

        let dc_ref = converted_opt.unwrap();
        Ok(PlaStartBlock {
            parent_id: dc_ref.parent_id,
            date: dc_ref.date.clone(),
            hour: dc_ref.hour
        })
    }
}

impl TryFrom<&HeirarchicalPlaLine> for PlaStartBlock {
    type Error = PlaParseError;

    fn try_from(value: &HeirarchicalPlaLine) -> Result<Self, Self::Error> {
        let str_command = value.text.trim_start();
        match value.parent_id {
            Some(x) => {
                match PlaStartBlock::try_from((x, str_command)) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(PlaParseError{ message: String::from("start block parsing failed")}),
                }
            },
            None => Err(PlaParseError{ message: String::from("Unable to parse HeirarchicalPlaLine without parent id as PlaStartBlock") }),
        }
    }
}

impl TryFrom<(u32, &str)> for PlaStartBlock {
    type Error = PlaParseError;

    fn try_from(value: (u32, &str)) -> Result<Self, Self::Error> {
        let (parent_id, command_text) = value;
        let tokens: Vec<String> = command_text.split(' ').map(|s| String::from(s)).collect();

        let str_date: String = match tokens.get(1) {
            Some(x) => String::from(x),
            None => return Err(PlaParseError { message: String::from("unable to parse date for start command") }),
        };

        let date: NaiveDate = match NaiveDate::parse_from_str(&str_date, "%Y-%m-%d") {
            Ok(x) => x,
            Err(_e) => return Err(PlaParseError { message: String::from("unable to parse date from string") })
        };

        // Default to midnight if an hour isn't provided.
        let str_hour = match tokens.get(2) {
            Some(x) => x,
            None => "0"
        };

        let hour = match str_hour.parse::<u32>() {
            Ok(x) => x,
            Err(_e) => return Err(PlaParseError { message: String::from("unable to parse hour of day in start sub block") })
        };

        return Ok(PlaStartBlock {
            date,
            hour,
            parent_id
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaChildBlock {
    pub parent_id: u32,
    pub child_id: u32
}

impl PlaSubBlock for PlaChildBlock {
    fn get_command(&self) -> PlaCommand {
        PlaCommand::CHILD
    }

    fn get_parent_id(&self) -> u32 {
        self.parent_id
    }
}

impl TryFrom<&HeirarchicalPlaLine> for PlaChildBlock {
    type Error = PlaParseError;

    fn try_from(value: &HeirarchicalPlaLine) -> Result<Self, Self::Error> {
        let str_command = value.text.trim_start();
        match value.parent_id {
            Some(x) => {
                match PlaChildBlock::try_from((x, str_command)) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(PlaParseError{ message: String::from("child block parsing failed")}),
                }
            },
            None => Err(PlaParseError{ message: String::from("Unable to parse HeirarchicalPlaLine without parent id as PlaChildBlock") }),
        }
    }
}

impl TryFrom<(u32, &str)> for PlaChildBlock {
    type Error = PlaParseError;

    fn try_from(value: (u32, &str)) -> Result<Self, Self::Error> {
        let (parent_id, input) = value;
        let tokens: Vec<String> = input.split(' ').map(|s| String::from(s)).collect();

        let child_id: u32 = match tokens.get(1) {
            Some(x) => match String::from(x).parse::<u32>() {
                Ok(y) => y,
                Err(e) => return Err(PlaParseError { message: String::from("Cannot parse child id for child command as u32")})
            },
            None => return Err(PlaParseError { message: String::from("unable to parse child id for child command") }),
        };

        Ok(PlaChildBlock {
            parent_id,
            child_id
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::pla::sub_blocks::PlaChildBlock;

    #[test]
    #[should_panic]
    fn it_should_panic_when_a_child_string_is_unparseable() {
        let child_block = PlaChildBlock::try_from((86, "child abc")).unwrap();
    }

    #[test]
    fn it_should_parse_a_child_block_from_a_valid_string_and_parent_id() {
        let child_block = PlaChildBlock::try_from((86, "child 202")).unwrap();
        assert_eq!(86, child_block.parent_id);
        assert_eq!(202, child_block.child_id);
    }

}

