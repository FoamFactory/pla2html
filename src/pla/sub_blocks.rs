use std::fmt::{Debug, Formatter};
use chrono::NaiveDate;
use mopa::mopafy;
use crate::pla::command::PlaCommand;
use crate::pla::error::{PlaSubBlockConversionError, PlaParseError};
use crate::pla::parser::{HeirarchicalPlaLine};

use dyn_clone::{clone_trait_object, DynClone};

pub trait PlaSubBlock: mopa::Any + DynClone {
    fn get_command(&self) -> PlaCommand;
    fn get_parent_id(&self) -> u32;
}

clone_trait_object!(PlaSubBlock);

mopafy!(PlaSubBlock);

impl std::fmt::Debug for Box<dyn PlaSubBlock> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self.get_command() {
            PlaCommand::START => {
                let start_res = PlaStartBlock::try_from(self);
                if start_res.is_ok() {
                    let start = start_res.unwrap();
                    String::from(format!("START {:?} {:?}", start.date, start.hour))
                } else {
                    String::from("ERROR")
                }
            },
            PlaCommand::CHILD => {
                let child_res = PlaChildBlock::try_from(self);
                if child_res.is_ok() {
                    let child = child_res.unwrap();
                    String::from(format!("CHILD {:?}", child.child_id))
                } else {
                    String::from("ERROR")
                }
            },
            PlaCommand::DURATION => {
                let duration_res = PlaDurationBlock::try_from(self);
                if duration_res.is_ok() {
                    let duration = duration_res.unwrap();
                    String::from(format!("DURATION {:?}", duration.duration))
                } else {
                    String::from("ERROR")
                }
            },
            PlaCommand::DEPENDENCY => {
                let dependency_res = PlaDependencyBlock::try_from(self);
                if dependency_res.is_ok() {
                    let dependency = dependency_res.unwrap();
                    String::from(format!("DEPENDENCY {:?}", dependency.dependency_id))
                } else {
                    String::from("ERROR")
                }
            },
            _ => String::from("ERROR")
        };
        write!(f, "{:?}", &str_rep)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaDependencyBlock {
    pub parent_id: u32,
    pub dependency_id: u32
}

impl PlaSubBlock for PlaDependencyBlock {
    fn get_command(&self) -> PlaCommand {
        PlaCommand::DEPENDENCY
    }

    fn get_parent_id(&self) -> u32 {
        self.parent_id
    }
}

impl TryFrom<&Box<dyn PlaSubBlock>> for PlaDependencyBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: &Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::DEPENDENCY {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::DEPENDENCY
            });
        }

        let converted_opt: Option<&PlaDependencyBlock> = value.downcast_ref::<PlaDependencyBlock>();

        if converted_opt.is_none() {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::DEPENDENCY
            });
        }

        let dc_ref = converted_opt.unwrap();
        Ok(PlaDependencyBlock {
            parent_id: dc_ref.parent_id,
            dependency_id: dc_ref.dependency_id
        })
    }
}

impl TryFrom<Box<dyn PlaSubBlock>> for PlaDependencyBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        PlaDependencyBlock::try_from(&value)
    }
}

impl TryFrom<&HeirarchicalPlaLine> for PlaDependencyBlock {
    type Error = PlaParseError;

    fn try_from(value: &HeirarchicalPlaLine) -> Result<Self, Self::Error> {
        let str_command = value.text.trim_start();
        match value.parent_id {
            Some(x) => {
                match PlaDependencyBlock::try_from((x, str_command)) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(PlaParseError{ message: String::from("dependency block parsing failed")}),
                }
            },
            None => Err(PlaParseError{ message: String::from("Unable to parse HeirarchicalPlaLine without parent id as PlaDependencyBlock") }),
        }
    }
}

impl TryFrom<(u32, &str)> for PlaDependencyBlock {
    type Error = PlaParseError;

    fn try_from(value: (u32, &str)) -> Result<Self, Self::Error> {
        let (parent_id, command_text) = value;
        let tokens: Vec<String> = command_text.split(' ').map(|s| String::from(s)).collect();

        let str_dep_id: &String = match tokens.get(1) {
            Some(x) => x,
            None => return Err(PlaParseError { message: String::from("Cannot parse dependency block without a dependency id") })
        };

        let dependency_id = match str_dep_id.parse::<u32>() {
            Ok(x) => x,
            Err(_) => return Err(PlaParseError { message: String::from("Unable to parse dependency id from string")})
        };

        return Ok(PlaDependencyBlock {
            parent_id,
            dependency_id
        });
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
    type Error = PlaSubBlockConversionError;

    fn try_from(value: &Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::START {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::START
            });
        }

        let converted_opt: Option<&PlaStartBlock> = value.downcast_ref::<PlaStartBlock>();

        if converted_opt.is_none() {
            return Err(PlaSubBlockConversionError {
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
    type Error = PlaSubBlockConversionError;

    fn try_from(value: Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        PlaStartBlock::try_from(&value)
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
                    Err(_) => Err(PlaParseError{ message: String::from("start block parsing failed")}),
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
pub struct PlaDurationBlock {
    pub parent_id: u32,
    pub duration: u32,
}

impl TryFrom<Box<dyn PlaSubBlock>> for PlaDurationBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        PlaDurationBlock::try_from(&value)
    }
}

impl TryFrom<&Box<dyn PlaSubBlock>> for PlaDurationBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: &Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::DURATION {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::DURATION
            });
        }

        let converted_opt: Option<&PlaDurationBlock> = value.downcast_ref::<PlaDurationBlock>();

        if converted_opt.is_none() {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::DURATION
            });
        }

        let dc_ref = converted_opt.unwrap();
        Ok(PlaDurationBlock {
            parent_id: dc_ref.parent_id,
            duration: dc_ref.duration
        })
    }
}

impl PlaSubBlock for PlaDurationBlock {
    fn get_command(&self) -> PlaCommand {
        PlaCommand::DURATION
    }

    fn get_parent_id(&self) -> u32 {
        self.parent_id
    }
}

impl TryFrom<&HeirarchicalPlaLine> for PlaDurationBlock {
    type Error = PlaParseError;

    fn try_from(value: &HeirarchicalPlaLine) -> Result<Self, Self::Error> {
        let str_command = value.text.trim_start();
        match value.parent_id {
            Some(x) => {
                match PlaDurationBlock::try_from((x, str_command)) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(PlaParseError{ message: String::from("duration block parsing failed")}),
                }
            },
            None => Err(PlaParseError{ message: String::from("Unable to parse HeirarchicalPlaLine without parent id as PlaDurationBlock") }),
        }
    }
}

impl TryFrom<(u32, &str)> for PlaDurationBlock {
    type Error = PlaParseError;

    fn try_from(value: (u32, &str)) -> Result<Self, Self::Error> {
        let (parent_id, input) = value;
        let tokens: Vec<String> = input.split(' ').map(|s| String::from(s)).collect();

        let duration_length: u32 = match tokens.get(1) {
            Some(x) => match String::from(x).parse::<u32>() {
                Ok(y) => y,
                Err(_) => return Err(PlaParseError { message: String::from("Cannot parse duration for duration command as u32")} )
            },
            None => return Err(PlaParseError { message: String::from("unable to parse duration length for duration command") })

        };

        Ok(PlaDurationBlock {
            parent_id,
            duration: duration_length
        })
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

impl TryFrom<Box<dyn PlaSubBlock>> for PlaChildBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        PlaChildBlock::try_from(&value)
    }
}

impl TryFrom<&Box<dyn PlaSubBlock>> for PlaChildBlock {
    type Error = PlaSubBlockConversionError;

    fn try_from(value: &Box<dyn PlaSubBlock>) -> Result<Self, Self::Error> {
        if value.get_command() != PlaCommand::CHILD {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::CHILD
            });
        }

        let converted_opt: Option<&PlaChildBlock> = value.downcast_ref::<PlaChildBlock>();

        if converted_opt.is_none() {
            return Err(PlaSubBlockConversionError {
                initial_type: PlaCommand::CHILD
            });
        }

        let dc_ref = converted_opt.unwrap();
        Ok(PlaChildBlock {
            parent_id: dc_ref.parent_id,
            child_id: dc_ref.child_id,
        })
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
                    Err(_) => Err(PlaParseError{ message: String::from("child block parsing failed")}),
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
                Err(_) => return Err(PlaParseError { message: String::from("Cannot parse child id for child command as u32")})
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
    use crate::pla::sub_blocks::{PlaChildBlock, PlaDurationBlock};

    #[test]
    #[should_panic]
    fn it_should_panic_when_a_child_string_is_unparseable() {
        let _child_block = PlaChildBlock::try_from((86, "child abc")).unwrap();
    }

    #[test]
    fn it_should_parse_a_child_block_from_a_valid_string_and_parent_id() {
        let child_block = PlaChildBlock::try_from((86, "child 202")).unwrap();
        assert_eq!(86, child_block.parent_id);
        assert_eq!(202, child_block.child_id);
    }

    #[test]
    fn it_should_parse_a_duration_block_from_a_valid_string_and_parent_id() {
        let duration_block: PlaDurationBlock = PlaDurationBlock::try_from((86, "duration 22")).unwrap();
        assert_eq!(86, duration_block.parent_id);
        assert_eq!(22, duration_block.duration);
    }

}

