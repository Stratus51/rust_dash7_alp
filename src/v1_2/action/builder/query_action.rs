macro_rules! build {
    ($name: ident, $test_name: ident) => {
        /// Add a condition on the execution of the next group of action.
        ///
        /// If the condition is not met, the next group of action should be skipped.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            /// Group with next action
            pub group: bool,
            /// Does not make sense.
            pub resp: bool,
            pub query: crate::v1_2::operand::Query,
        }
        crate::v1_2::action::impl_display_simple_op!($name, query);
        crate::v1_2::action::impl_op_serialized!(
            $name,
            group,
            resp,
            query,
            crate::v1_2::operand::Query,
            crate::v1_2::operand::QueryDecodingError
        );
        #[test]
        fn $test_name() {
            crate::test_tools::test_item(
                $name {
                    group: true,
                    resp: true,
                    query: crate::v1_2::operand::Query::NonVoid(crate::v1_2::operand::NonVoid {
                        size: 4,
                        file: crate::v1_2::operand::FileOffset { id: 5, offset: 6 },
                    }),
                },
                &vec![
                    [crate::v1_2::action::OpCode::$name as u8 | (3 << 6)].as_slice(),
                    &hex_literal::hex!("00 04  05 06"),
                ]
                .concat()[..],
            )
        }
    };
}

pub(crate) use build;
