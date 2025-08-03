use crate::cli::builtins::BuiltinFile;
use clap::{
    ValueEnum,
    builder::{TypedValueParser, ValueParserFactory},
};

#[derive(Debug, Clone)]
pub enum BuiltinOrFile<T: ValueEnum> {
    /// A built-in value.
    Builtin(T),
    /// A file input.
    File(clio::Input),
}

impl<T: ValueEnum + BuiltinFile> BuiltinOrFile<T> {
    pub fn reader(&mut self) -> Box<dyn std::io::BufRead + '_> {
        match self {
            Self::Builtin(builtin) => Box::new(builtin.reader()),
            Self::File(file) => file.lock(),
        }
    }
}

impl<T: ValueEnum> From<T> for BuiltinOrFile<T> {
    fn from(value: T) -> Self {
        BuiltinOrFile::Builtin(value)
    }
}

impl<T: ValueEnum + Default> Default for BuiltinOrFile<T> {
    fn default() -> Self {
        BuiltinOrFile::Builtin(T::default())
    }
}

impl<T: ValueEnum> ValueParserFactory for BuiltinOrFile<T> {
    type Parser = BuiltinOrFileParser<T>;

    fn value_parser() -> Self::Parser {
        BuiltinOrFileParser(std::marker::PhantomData)
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinOrFileParser<T: ValueEnum>(std::marker::PhantomData<T>);

impl<T: ValueEnum + Send + Sync + 'static> TypedValueParser for BuiltinOrFileParser<T> {
    type Value = BuiltinOrFile<T>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        if let Ok(builtin) = clap::builder::EnumValueParser::<T>::new().parse_ref(cmd, arg, value) {
            Ok(Self::Value::Builtin(builtin))
        } else {
            clio::Input::try_from(value)
                .map(Self::Value::File)
                .map_err(|err| {
                    clap::Error::raw(clap::error::ErrorKind::Io, err.to_string()).with_cmd(cmd)
                })
        }
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(
            T::value_variants()
                .iter()
                .filter_map(|v| v.to_possible_value())
                .chain(std::iter::once(
                    clap::builder::PossibleValue::new("<filepath>").help("Path to a file"),
                )),
        ))
    }
}
