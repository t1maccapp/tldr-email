use std::{
    hash::{Hash, Hasher},
    mem::discriminant,
};

#[derive(Debug)]
pub enum Actions {
    ListFolders {
        login: String,
    },
    ListEnvelopes {
        login: String,
        folder: String,
        page: usize,
    },
    GetMessage {
        login: String,
        folder: String,
        id: String,
    },
    SendMessage {
        login: String,
        to: String,
        subject: String,
        text: String,
    },
}

impl PartialEq for Actions {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Eq for Actions {}

impl Hash for Actions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
    }
}
