pub enum Language {
    English,
}

impl Language {
    pub fn change_language(&mut self, language: Self) {
        *self = language;
    }

    pub fn user_id(&self) -> &str {
        match self {
            Self::English => return "user id",
        }
    }
}
