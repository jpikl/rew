#[derive(Debug, PartialEq)]
pub struct Group<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
}

pub const COMMANDS: Group = Group {
    name: "Commands",
    description: None,
};

pub const OPTIONS: Group = Group {
    name: "Options",
    description: None,
};

pub const ARGUMENTS: Group = Group {
    name: "Arguments",
    description: None,
};
