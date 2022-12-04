macro_rules! build {
    ($name: ident, $test_name: ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            /// Group with next action
            pub group: bool,
            /// Ask for a response (a status)
            pub resp: bool,
            pub file_id: u8,
            pub header: data::FileHeader,
        }
        crate::v1_2::action::impl_header_op!($name, group, resp, file_id, header);
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({}){}",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.file_id,
                    self.header,
                )
            }
        }
    };
}
pub(crate) use build;
