/// Add a condition on the execution of the next group of action.
///
/// If the condition is not met, the next group of action should be skipped.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryAction {
    /// Group with next action
    pub group: bool,
    /// Does not make sense.
    pub resp: bool,
    pub query: crate::spec::v1_2::operand::Query,
}
super::impl_display_simple_op!(QueryAction, query);
super::impl_op_serialized!(
    QueryAction,
    group,
    resp,
    query,
    super::operand::Query,
    super::operand::QueryDecodingError
);
