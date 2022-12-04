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
    };
}

pub(crate) use build;
