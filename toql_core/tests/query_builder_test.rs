
use toql_core::query::Field;
use toql_core::query::Wildcard;
use toql_core::query::Query;

#[test]
fn build_filters() {
    let q = Query::new()
        .and(Field::from("foo").eq("bar"))
        .and(Field::from("foo").eqn())
        .and(Field::from("foo").ne("bar"))
        .and(Field::from("foo").nen())
        .and(Field::from("foo").gt(42))
        .and(Field::from("foo").ge(42))
        .and(Field::from("foo").lt(42))
        .and(Field::from("foo").le(42));

    assert_eq!(
        "foo EQ 'bar',foo EQN,foo NE 'bar',foo NEN,foo GT 42,foo GE 42,foo LT 42,foo LE 42",
        q.to_string()
    );

    let q = Query::new()
        .and(Field::from("foo").lk("foo"))
        .and(Field::from("foo").re("foo"))
        .and(Field::from("foo").bw(41, 43))
        .and(Field::from("foo").ins(vec![1, 2, 3]))
        .and(Field::from("foo").out(vec![1, 2, 3]))
    .and(Field::from("foo").fnc("ma", vec!["bar"]));

    assert_eq!("foo LK 'foo',foo RE 'foo',foo BW 41 43,foo IN 1 2 3,foo OUT 1 2 3,foo FN ma 'bar'", q.to_string());
}

#[test]
fn build_field() {
    let  q = Query::new()
        .and(Field::from("foo").hide().eq(5).aggregate().asc(1))
        .and(Field::from("bar").desc(2));
    assert_eq!("+1.foo !EQ 5,-2bar", q.to_string());
}
#[test]
fn build_wildcards() {
    let  q = Query::double_wildcard()
        .and(Field::from("foo"))
        .and(Wildcard::from("bar"))
        .and(Wildcard::from("bar4_")); // Underscore is optional
    assert_eq!("**,foo,bar_*,bar4_*", q.to_string());
}

#[test]
fn build_logical() {
    let  q = Query::new().and("foo").and("bar").or("foo");
    assert_eq!("(foo,bar);foo", q.to_string());

    let q = Query::new().and("foo").and("bar").or("foo").or("bar");
    assert_eq!("((foo,bar);foo);bar", q.to_string());
}

#[test]
fn build_logical2() {
    let q1 = Query::new().and("foo").and("bar");

    let q2 = Query::new().and("foo").and("bar");

    let q = q1.or(q2);

    assert_eq!("(foo,bar);(foo,bar)", q.to_string());
}
