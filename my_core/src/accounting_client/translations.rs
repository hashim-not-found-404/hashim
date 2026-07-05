pub(crate) enum Language {
    English,
}

impl Language {
    pub(crate) fn change_language(&mut self, language: Self) {
        *self = language;
    }

    pub(crate) fn user_id(&self) -> &str {
        match self {
            Self::English => return "user id",
        }
    }
}
