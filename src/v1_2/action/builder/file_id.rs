macro_rules! build {
    ($name: ident, $test_name: ident) => {
        /// Checks whether a file exists
        // ALP_SPEC: How is the result of this command different from a read file of size 0?
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            /// Group with next action
            pub group: bool,
            /// Ask for a response (status?)
            pub resp: bool,
            pub file_id: u8,
        }
        crate::v1_2::action::impl_display_simple_file_op!($name, file_id);
        crate::v1_2::action::impl_simple_op!($name, group, resp, file_id);
    };
}

pub(crate) use build;
