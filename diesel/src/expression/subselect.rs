use core::marker::PhantomData;

use crate::expression::array_comparison::InExpression;
use crate::expression::*;
use crate::query_builder::*;
use crate::result::QueryResult;

/// This struct tells our type system that the whatever we put in `values`
/// will be handled by SQL as an expression of type `ST`.
/// It also implements the usual `SelectableExpression` and `AppearsOnTable` traits
/// (which is useful when using this as an expression). To enforce correctness here, it checks
/// the dedicated [`ValidSubselect`]. This however does not check that the `SqlType` of
/// [`SelectQuery`], matches `ST`, so appropriate constraints should be checked in places that
/// construct Subselect. (It's not always equal, notably .single_value() makes `ST` nullable, and
/// `exists` checks bounds on `SubSelect<T, Bool>` although there is actually no such subquery in
/// the final SQL.)
///
/// The third type parameter `W` carries the WHERE clause type from the inner query.
/// When `W` implements `ValidGrouping<GB>`, the subselect's aggregate status is
/// determined by the WHERE clause's column references. The default `W = ()` yields
/// `is_aggregate::Never` for backward compatibility with `Exists` and `IN` subqueries.
#[derive(Debug, Copy, Clone, QueryId)]
pub struct Subselect<T, ST, W = ()> {
    values: T,
    _sql_type: PhantomData<ST>,
    _where: PhantomData<W>,
}

impl<T, ST, W> Subselect<T, ST, W> {
    pub(crate) fn new(values: T) -> Self {
        Self {
            values,
            _sql_type: PhantomData,
            _where: PhantomData,
        }
    }
}

impl<T: SelectQuery, ST, W> Expression for Subselect<T, ST, W>
where
    ST: SqlType + TypedExpressionType,
{
    // This is useful for `.single_value()`
    type SqlType = ST;
}

impl<T, ST: SqlType, W> InExpression for Subselect<T, ST, W> {
    type SqlType = ST;
    fn is_empty(&self) -> bool {
        false
    }
    fn is_array(&self) -> bool {
        false
    }
}

impl<T, ST, W, QS> SelectableExpression<QS> for Subselect<T, ST, W>
where
    Subselect<T, ST, W>: AppearsOnTable<QS>,
    T: ValidSubselect<QS>,
{
}

impl<T, ST, W, QS> AppearsOnTable<QS> for Subselect<T, ST, W>
where
    Subselect<T, ST, W>: Expression,
    T: ValidSubselect<QS>,
{
}

impl<T, ST, W, GB> ValidGrouping<GB> for Subselect<T, ST, W>
where
    W: ValidGrouping<GB>,
{
    type IsAggregate = <W as ValidGrouping<GB>>::IsAggregate;
}

impl<T, ST, W, DB> QueryFragment<DB> for Subselect<T, ST, W>
where
    DB: Backend,
    T: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        self.values.walk_ast(out.reborrow())?;
        Ok(())
    }
}

pub trait ValidSubselect<QS> {}

/// Extracts the WHERE clause type from a query for use in
/// [`Subselect`]'s `ValidGrouping` implementation.
#[doc(hidden)]
pub trait SubselectWhereClause {
    /// The WHERE clause type of this query.
    type WhereClause;
}
