use anyhow::anyhow;
use anyhow::Result;
use clap::builder::PossibleValue;
use clap::builder::StyledStr;
use clap::builder::ValueRange;
use clap::Arg;
use clap::Command;
use rew::command::Group;
use rew::commands::get_meta;
use std::borrow::Cow;

pub struct Adapter<'a> {
    inner: &'a Command,
    parents: Vec<&'a Command>,
}

impl<'a> Adapter<'a> {
    pub fn new(inner: &'a Command) -> Self {
        Self {
            inner,
            parents: Vec::new(),
        }
    }

    pub fn parents_and_self(&self) -> Vec<&'a Command> {
        [&self.parents[..], &[self.inner]].concat()
    }

    pub fn name(&self) -> &str {
        self.inner.get_name()
    }

    pub fn full_name(&self) -> &str {
        self.inner
            .get_bin_name()
            .unwrap_or_else(|| self.inner.get_name())
    }

    pub fn file_stem(&self) -> &str {
        self.inner
            .get_display_name()
            .unwrap_or_else(|| self.inner.get_name())
    }

    pub fn description(&self) -> Result<&StyledStr> {
        self.inner
            .get_long_about()
            .map_or_else(|| self.short_description(), Ok)
    }

    pub fn short_description(&self) -> Result<&StyledStr> {
        self.inner
            .get_about()
            .ok_or_else(|| anyhow!("Command '{}' does not have description", self.full_name()))
    }

    pub fn synopsis_args(&'a self) -> Vec<SynopsisArg<'a>> {
        let mut args = Vec::new();

        if self.raw_opt_args().next().is_some() {
            args.push(SynopsisArg::Options);
        }

        if let Some(pos_args) = self.pos_args() {
            for arg in pos_args {
                args.push(SynopsisArg::Positional(arg));
            }
        }

        if self.raw_sucommands().next().is_some() {
            args.push(SynopsisArg::Commands(self));
        }

        args
    }

    pub fn subcommands(&'a self) -> Option<Vec<Adapter<'a>>> {
        self.filter_subcommands(|_| true)
    }

    pub fn groupped_subcommands(&'a self) -> Option<Vec<(Group, Vec<Adapter<'a>>)>> {
        let mut groups = Vec::new();

        for group in Group::values() {
            let filter = |subcommand: &&Command| {
                if self.parents.is_empty() {
                    let meta = get_meta(subcommand.get_name());
                    meta.map(|sc| sc.group).unwrap_or_default() == group
                } else {
                    false
                }
            };
            if let Some(subcommands) = self.filter_subcommands(filter) {
                groups.push((group, subcommands));
            }
        }

        if groups.is_empty() {
            None
        } else {
            Some(groups)
        }
    }

    pub fn filter_subcommands(
        &'a self,
        filter: impl (Fn(&&Command) -> bool),
    ) -> Option<Vec<Adapter<'a>>> {
        if self.name() == "help" {
            return None;
        }

        let subcommands = self
            .raw_sucommands()
            .filter(filter)
            .map(|subcommand| Adapter {
                inner: subcommand,
                parents: self.parents_and_self(),
            })
            .collect::<Vec<_>>();

        if subcommands.is_empty() {
            None
        } else {
            Some(subcommands)
        }
    }

    pub fn pos_args(&self) -> Option<Vec<PositionalArg<'_>>> {
        let positionals = self
            .raw_pos_args()
            .map(BaseArg)
            .map(PositionalArg)
            .collect::<Vec<_>>();

        if positionals.is_empty() {
            None
        } else {
            Some(positionals)
        }
    }

    pub fn opt_args(&self) -> Option<Vec<OptionalArg<'_>>> {
        let options = self
            .raw_opt_args()
            .map(BaseArg)
            .map(OptionalArg)
            .collect::<Vec<_>>();

        if options.is_empty() {
            None
        } else {
            Some(options)
        }
    }

    pub fn global_opt_args(&'a self) -> Option<GlobalArgs<'a>> {
        let mut inherited = Vec::new();
        let mut own = Vec::new();

        for global_arg in self.raw_global_opt_args() {
            if self.parents.iter().any(|parent| {
                parent
                    .get_arguments()
                    .any(|arg| arg.get_id() == global_arg.get_id())
            }) {
                inherited.push(global_arg.get_id());
            } else {
                own.push(OptionalArg(BaseArg(global_arg)));
            }
        }

        let own = if own.is_empty() { None } else { Some(own) };

        let inherited_from = self
            .parents
            .iter()
            .rev()
            .find(|parent| {
                parent
                    .get_arguments()
                    .any(|arg| inherited.contains(&arg.get_id()))
            })
            .map(|parent| Adapter {
                inner: parent,
                parents: Vec::new(),
            });

        if inherited_from.is_none() && own.is_none() {
            None
        } else {
            Some(GlobalArgs {
                inherited_from,
                own,
            })
        }
    }

    fn raw_sucommands(&'a self) -> impl Iterator<Item = &'a Command> {
        self.inner
            .get_subcommands()
            .filter(|subcommand| !subcommand.is_hide_set())
    }

    fn raw_opt_args(&'a self) -> impl Iterator<Item = &'a Arg> {
        self.raw_args()
            .filter(|arg| !arg.is_positional())
            .filter(|arg| !arg.is_global_set())
    }

    fn raw_global_opt_args(&'a self) -> impl Iterator<Item = &'a Arg> {
        self.raw_args()
            .filter(|arg| !arg.is_positional())
            .filter(|arg| arg.is_global_set())
    }

    fn raw_pos_args(&'a self) -> impl Iterator<Item = &'a Arg> {
        self.raw_args().filter(|arg| arg.is_positional())
    }

    fn raw_args(&'a self) -> impl Iterator<Item = &'a Arg> {
        self.inner.get_arguments().filter(|arg| !arg.is_hide_set())
    }
}

pub enum SynopsisArg<'a> {
    Options,
    Positional(PositionalArg<'a>),
    Commands(&'a Adapter<'a>),
}

impl<'a> SynopsisArg<'a> {
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            Self::Options => "OPTIONS".into(),
            Self::Positional(arg) => arg.name().into(),
            Self::Commands(command) => command
                .inner
                .get_subcommand_value_name()
                .unwrap_or("COMMAND")
                .into(),
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            Self::Options => false,
            Self::Positional(arg) => arg.is_required(),
            Self::Commands(command) => command.inner.is_subcommand_required_set(),
        }
    }

    pub fn is_many(&self) -> bool {
        match self {
            Self::Positional(arg) => arg.is_many(),
            _ => false,
        }
    }
}

pub struct PositionalArg<'a>(BaseArg<'a>);

impl<'a> PositionalArg<'a> {
    pub fn base(&self) -> &BaseArg<'a> {
        &self.0
    }

    pub fn name(&self) -> String {
        self.base()
            .0
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .next()
            .map_or_else(
                || self.base().0.get_id().to_string().to_uppercase(),
                ToString::to_string,
            )
    }

    pub fn is_required(&self) -> bool {
        self.base().0.is_required_set()
    }

    pub fn is_many(&self) -> bool {
        self.base().value_range().max_values() > 1
    }
}

pub struct OptionalArg<'a>(BaseArg<'a>);

impl<'a> OptionalArg<'a> {
    pub fn base(&self) -> &BaseArg<'a> {
        &self.0
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();

        if !self.base().0.is_hide_short_help_set() {
            if let Some(shorts) = self.base().0.get_short_and_visible_aliases() {
                for short in shorts {
                    names.push(format!("-{short}"));
                }
            }
        }

        if !self.base().0.is_hide_long_help_set() {
            if let Some(longs) = self.base().0.get_long_and_visible_aliases() {
                for long in longs {
                    names.push(format!("--{long}"));
                }
            }
        }

        names
    }

    pub fn value_names(&self) -> Vec<String> {
        if !self.base().value_range().takes_values() {
            return Vec::new();
        }
        self.base()
            .0
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect()
    }
}

pub struct GlobalArgs<'a> {
    pub inherited_from: Option<Adapter<'a>>,
    pub own: Option<Vec<OptionalArg<'a>>>,
}

pub struct BaseArg<'a>(&'a Arg);

impl<'a> BaseArg<'a> {
    pub fn description(&self) -> Result<&StyledStr> {
        let long_help = if self.0.is_hide_long_help_set() {
            None
        } else {
            self.0.get_long_help()
        };

        let short_help = if self.0.is_hide_short_help_set() {
            None
        } else {
            self.0.get_help()
        };

        long_help
            .or(short_help)
            .ok_or_else(|| anyhow!("Argument '{}' does not have description", self.0.get_id()))
    }

    pub fn possible_values(&self) -> Option<Vec<Value>> {
        if self.0.is_hide_possible_values_set() {
            return None;
        }

        if self.0.is_positional() || self.value_range().takes_values() {
            let values = self
                .0
                .get_possible_values()
                .into_iter()
                .filter(|value| !value.is_hide_set())
                .map(Value)
                .collect::<Vec<Value>>();

            if !values.is_empty() {
                return Some(values);
            }
        }

        None
    }

    pub fn default_value(&self) -> Option<String> {
        if self.0.is_hide_default_value_set() || !self.value_range().takes_values() {
            return None;
        }

        let values = self.0.get_default_values();
        if values.is_empty() {
            return None;
        }

        let result = values
            .iter()
            .map(|value| value.to_string_lossy())
            .collect::<Vec<_>>()
            .join(",");

        Some(result)
    }

    pub fn env_var(&self) -> Option<Cow<'_, str>> {
        if self.0.is_hide_env_set() {
            return None;
        }
        self.0.get_env().map(|value| value.to_string_lossy())
    }

    fn value_range(&self) -> ValueRange {
        self.0
            .get_num_args()
            .expect("command.build() was not called")
    }
}

pub struct Value(PossibleValue);

impl Value {
    pub fn name(&self) -> &str {
        self.0.get_name()
    }

    pub fn description(&self) -> Result<&StyledStr> {
        self.0.get_help().ok_or_else(|| {
            anyhow!(
                "Possible value '{}' does not have description",
                self.0.get_name()
            )
        })
    }
}
