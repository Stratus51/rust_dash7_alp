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
        crate::action::impl_display_simple_file_op!($name, file_id);
        crate::action::impl_simple_op!($name, group, resp, file_id);
        #[test]
        fn $test_name() {
            test_item(
                $name {
                    group: false,
                    resp: false,
                    file_id: 9,
                },
                &[crate::action::OpCode::$name as u8, 0x09],
            )
        }
    };
}

pub(crate) use build;
