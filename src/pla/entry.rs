use std::fmt::{Debug, Formatter};
use crate::pla::sub_blocks::{PlaSubBlock};

pub struct PlaEntry {
    pub id: u32,
    pub description: String,
    pub children: Option<Vec<Box<dyn PlaSubBlock>>>
}

impl PlaEntry {
    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }
}

impl Clone for PlaEntry {
    fn clone(&self) -> Self {
        let cloned_children: Option<Vec<Box<dyn PlaSubBlock>>> = match self.children.as_ref() {
            Some(c) => {
                Some(c
                    .iter()
                    .map(|bsb| {
                        bsb.clone()
                    })
                    .collect())
            },
            None => None
        };

        PlaEntry {
            id: self.id,
            description: String::from(&self.description),
            children: cloned_children,
        }
    }
}

impl PartialEq for PlaEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.description == other.description
    }
}

impl Debug for PlaEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut target_string = String::new();
        use std::fmt::Write;
        let children_str: String = match &self.children {
            Some(x) => {
                write!(target_string, "{:?}", x).unwrap();

                target_string
            },
            None => String::from("None")
        };

        write!(f, "PlaEntry ({:?}), description: {:?}, children: {:?}", self.id, self.description, children_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::pla::entry::PlaEntry;

    #[test]
    fn it_should_be_able_to_clone_an_entry_with_no_children() {
        let entry = PlaEntry {
            id: 196,
            description: "No Operation".to_string(),
            children: None
        };

        let cloned_entry = entry.clone();
        assert_eq!(196, cloned_entry.id);
        assert_eq!(String::from("No Operation"), cloned_entry.description);
        assert!(cloned_entry.children.is_none());
    }
}
