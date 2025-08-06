//! https://github.com/clap-rs/clap/issues/2222#issuecomment-2524152894

use clap::{Args, FromArgMatches, Subcommand};

/// `[Args]` wrapper to match `T` variants recursively in `U`.
#[derive(Debug, Clone)]
pub struct ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    /// Specific Variant.
    pub inner: T,
    /// Enum containing `Self<T>` variants, in other words possible follow-up commands.
    pub next: Option<Box<U>>,
}

impl<T, U> ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    pub fn new(inner: T) -> Self {
        Self { inner, next: None }
    }
}

impl<T, U> Args for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn augment_args(cmd: clap::Command) -> clap::Command {
        T::augment_args(cmd).defer(|cmd| U::augment_subcommands(cmd.disable_help_subcommand(true)))
    }
    fn augment_args_for_update(_cmd: clap::Command) -> clap::Command {
        unimplemented!()
    }
}

impl<T, U> FromArgMatches for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        // dbg!(&matches);
        // this doesn't match subcommands but a first match
        let inner = T::from_arg_matches(matches)?;
        let next = if let Some((_name, _sub)) = matches.subcommand() {
            // dbg!(_name, _sub);
            // most weirdly, Subcommand skips into the matched
            // .subcommand, hence we need to pass outer matches
            // (which in the average case should only match enumerated T)
            Some(U::from_arg_matches(matches)?)
            // we are done, since sub-sub commands are matched in U::
        } else {
            None
        };
        Ok(Self {
            inner,
            next: next.map(Box::new),
        })
    }
    fn update_from_arg_matches(&mut self, _matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        unimplemented!()
    }
}

impl<T, U> Default for ReClap<T, U>
where
    T: Args + Default,
    U: Subcommand,
{
    fn default() -> Self {
        Self {
            inner: T::default(),
            next: None,
        }
    }
}
