pub trait BuiltinFile {
    fn reader(&self) -> std::io::Cursor<&'static [u8]>;
}

macro_rules! impl_builtin_file {
    ($struct:ident, $($field:ident => $filepath:literal),*) => {
        impl crate::cli::builtins::BuiltinFile for $struct {
            fn reader(&self) -> std::io::Cursor<&'static [u8]> {
				match self {
					$(
						Self::$field => {
							include_flate::flate!(static DATA: [u8] from $filepath);
							std::io::Cursor::new(DATA.as_slice())
						}
					)*
				}
            }
        }
    };
}
pub(crate) use impl_builtin_file;
