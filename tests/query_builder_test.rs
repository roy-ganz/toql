extern crate toql;

    use toql::query::Query;
    use toql::query::Field;

    
#[test]
    fn build_filters() {
       
       let mut q = Query::new();
        q.and( Field::from("foo").eq("bar"));
        q.and( Field::from("foo").eqn());
        q.and( Field::from("foo").ne("bar"));
        q.and( Field::from("foo").nen());
        q.and( Field::from("foo").gt(42));
        q.and( Field::from("foo").ge(42));
        q.and( Field::from("foo").lt(42));
        q.and( Field::from("foo").le(42));

        assert_eq!("foo EQ 'bar',foo EQN,foo NE 'bar',foo NEN,foo GT 42,foo GE 42,foo LT 42,foo LE 42", q.to_string());
        
         let mut q = Query::new();
        q.and( Field::from("foo").lk("foo"));
        q.and( Field::from("foo").re("foo"));
        q.and( Field::from("foo").sc("foo"));
        q.and( Field::from("foo").bw(41, 43));
        q.and( Field::from("foo").ins(vec![1, 2, 3]));
        q.and( Field::from("foo").out(vec![1, 2, 3]));
        q.and( Field::from("foo").fnc("ma", vec!["bar"]));
                    
        assert_eq!("foo LK 'foo',foo RE 'foo',foo SC 'foo',foo BW 41 43,foo IN 1 2 3,foo OUT 1 2 3,foo FN ma 'bar'", q.to_string());            
        
    }

    #[test]
    fn build_field () {

        let mut q = Query::new();
        q.and(Field::from("foo").hide().eq(5).aggregate().asc(1));
        q.and(Field::from("bar").desc(2));
        assert_eq!("+1.foo !EQ 5,-2bar", q.to_string());            
    }
    #[test]
    fn build_logical () {

        let mut q = Query::new();
        q.and ("foo");
        q.and ("bar");
        q.or ("foo");
        assert_eq!("(foo,bar);foo", q.to_string()); 

        let mut q = Query::new();
        q.and ("foo");
        q.and ("bar");
        q.or ("foo");
        q.or ("bar");
        assert_eq!("((foo,bar);foo);bar", q.to_string());            
    }

    #[test]
     fn build_logical2 () {

        let mut q1 = Query::new();
        q1.and ("foo");
        q1.and ("bar");
       
        let mut q2 = Query::new();
        q2.and ("foo");
        q2.and ("bar");
        
        q1.or(q2);

        assert_eq!("(foo,bar);(foo,bar)", q1.to_string());            
    }

