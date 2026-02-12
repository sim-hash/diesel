extern crate diesel;

use diesel::prelude::*;

table! {
    parents {
        id -> Integer,
    }
}

table! {
    childs {
        id -> Integer,
        parent_id -> Integer,
        amount -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(parents, childs);

fn main() {
    // Mixing an aggregate function with a subselect that references an outer
    // column should fail when the outer query has no GROUP BY clause.
    // The subquery references parents::id in its filter, which is ungrouped.
    let _ = parents::table.select((
        diesel::dsl::count_distinct(parents::id),
        childs::table
            .filter(childs::parent_id.eq(parents::id))
            .select(diesel::dsl::sum(childs::amount).assume_not_null())
            .single_value()
            .assume_not_null(),
    ));
    //~^ ERROR
}
