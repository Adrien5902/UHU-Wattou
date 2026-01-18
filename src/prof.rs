#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Prof {
    name: String,
}

impl ToString for Prof {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl Prof {
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
