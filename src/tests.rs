#[allow(unused_imports)]

pub mod tests {
    extern crate rand;
    extern crate test;
    use ndarray::{arr2, arr1, Axis, stack};
    use dataframe::*;
    use join::{Join, JoinType};
    use test::Bencher;
    use ndarray::Array;
    use rand::distributions::Range;
    use ndarray_rand::RandomExt;
    use std::rc::Rc;
    use rand::{thread_rng, Rng};
    use std::collections::{HashMap, BTreeMap};
    use chrono::*;
    use types::*;
    use from::*;
    use error::*;

    #[test]
    fn outer_left_join() {
        let mut left = HashMap::new();
        left.insert(1, "Alice");
        left.insert(2, "Bob");

        let mut right = HashMap::new();
        right.insert(1, "Programmer");
        let mut res: Vec<(usize, &str, Option<&str>)> =
            Join::new(JoinType::OuterLeftJoin, left.into_iter(), right).collect();
        res.sort_by_key(|x| x.1);
        assert_eq!(res,
                   vec![(1, "Alice", Some("Programmer")), (2, "Bob", None)])

    }
    #[test]
    fn inner_join() {
        let mut left = HashMap::new();
        left.insert(1, "Alice");
        left.insert(2, "Bob");
        left.insert(3, "Suchin");

        let mut right = HashMap::new();
        right.insert(1, "Programmer");
        right.insert(3, "Data Scientist");
        let mut res: Vec<(usize, &str, Option<&str>)> =
            Join::new(JoinType::InnerJoin, left.into_iter(), right).collect();
        res.sort_by_key(|x| x.1);
        assert_eq!(res,
                   vec![(1, "Alice", Some("Programmer")), (3, "Suchin", Some("Data Scientist"))])

    }

    #[test]
    fn dataframe_creation() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("a".to_string()), 0);
        names.insert(ColumnType::Str("b".to_string()), 1);

        let df = DataFrame::new(a).columns(names);
        assert!(df.is_ok())
    }

    #[test]
    fn dataframe_creation_datetime_index() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Date(UTC.ymd(2014, 7, 8).and_hms(9, 10, 11)), 0);
        names.insert(ColumnType::Date(UTC.ymd(2014, 10, 5).and_hms(2, 5, 7)), 1);

        let df: Result<DataFrame> = DataFrame::new(a).columns(names);
        assert!(df.is_ok())
    }

    #[test]
    fn dataframe_index() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("a".to_string()), 0);
        names.insert(ColumnType::Str("b".to_string()), 1);

        let mut index: BTreeMap<IndexType, usize> = BTreeMap::new();
        index.insert(IndexType::Str("a".to_string()), 0);
        index.insert(IndexType::Str("b".to_string()), 1);

        let df = DataFrame::new(a).columns(names).unwrap();
        assert!(df.get(ColumnType::Str("a".to_string())).unwrap() ==
                arr2(&[[2., 3.], [3., 4.]]).mapv(InnerType::from).column(0).to_owned())
    }

    #[test]
    fn dataframe_join() {

        let a = Array::random((3, 2), Range::new(0., 10.));
        let b = arr2(&[[1.], [5.], [2.]]);
        let f = arr2(&[[1.], [0.], [2.]]);
        let d = Array::random((3, 2), Range::new(0., 10.));
        let c = stack(Axis(1), &[a.view(), b.view()]).unwrap();
        let e = stack(Axis(1), &[d.view(), f.view()]).unwrap();

        let mut c_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        c_names.insert(ColumnType::Str("1".to_string()), 0);
        c_names.insert(ColumnType::Str("2".to_string()), 1);
        c_names.insert(ColumnType::Str("3".to_string()), 2);


        let mut e_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        e_names.insert(ColumnType::Str("4".to_string()), 0);
        e_names.insert(ColumnType::Str("5".to_string()), 1);
        e_names.insert(ColumnType::Str("3".to_string()), 2);



        let c_df = DataFrame::new(c.clone()).columns(c_names).unwrap();
        let e_df = DataFrame::new(e.clone()).columns(e_names).unwrap();



        let mut join_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        join_names.insert(ColumnType::Str("1".to_string()), 0);
        join_names.insert(ColumnType::Str("2".to_string()), 1);
        join_names.insert(ColumnType::Str("3".to_string()), 2);
        join_names.insert(ColumnType::Str("4_x".to_string()), 3);
        join_names.insert(ColumnType::Str("5_x".to_string()), 4);
        join_names.insert(ColumnType::Str("3_x".to_string()), 5);

        let mut join_index: BTreeMap<IndexType, usize> = BTreeMap::new();
        join_index.insert(IndexType::Str("0".to_string()), 0);
        join_index.insert(IndexType::Str("1".to_string()), 1);
        join_index.insert(IndexType::Str("2".to_string()), 2);

        let join_matrix = stack(Axis(1),
                                &[c.select(Axis(0), &[0, 1, 2]).view(),
                                  e.select(Axis(0), &[0, 1, 2]).view()])
            .unwrap();

        let join_df = DataFrame::new(join_matrix).columns(join_names).unwrap().index(join_index);
        let test_df = c_df.inner_join(&e_df);
        assert_eq!(join_df.unwrap(), test_df.unwrap().clone())

    }


    #[test]
    fn dataframe_insert() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("a".to_string()), 0);
        names.insert(ColumnType::Str("b".to_string()), 1);
        let df = DataFrame::new(a).columns(names).unwrap();

        let new_array = arr2(&[[5.], [6.]]);

        let new_df = df.insert_column(new_array, ColumnType::Str("c".to_string()));
        let mut new_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        new_names.insert(ColumnType::Str("a".to_string()), 0);
        new_names.insert(ColumnType::Str("b".to_string()), 1);
        new_names.insert(ColumnType::Str("c".to_string()), 2);
        let a_prime = arr2(&[[2., 3., 5.], [3., 4., 6.]]);
        assert_eq!(DataFrame::new(a_prime).columns(new_names).unwrap(),
                   new_df.unwrap())
    }

    #[test]
    fn dataframe_concat() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let b = arr2(&[[7., 1.], [7., 6.]]);
        let mut a_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        a_names.insert(ColumnType::Str("a".to_string()), 0);
        a_names.insert(ColumnType::Str("b".to_string()), 1);

        let mut b_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        b_names.insert(ColumnType::Str("c".to_string()), 0);
        b_names.insert(ColumnType::Str("d".to_string()), 1);
        let df = DataFrame::new(a).columns(a_names).unwrap();
        let df_1 = DataFrame::new(b).columns(b_names).unwrap();

        let row_concat = df.concat(Axis(0), &df_1);
        let col_concat = df.concat(Axis(1), &df_1);


        let mut a_prime_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        a_prime_names.insert(ColumnType::Str("a".to_string()), 0);
        a_prime_names.insert(ColumnType::Str("b".to_string()), 1);

        let mut a_prime_index: BTreeMap<IndexType, usize> = BTreeMap::new();
        a_prime_index.insert(IndexType::Str("0".to_string()), 0);
        a_prime_index.insert(IndexType::Str("1".to_string()), 1);
        a_prime_index.insert(IndexType::Str("0_x".to_string()), 2);
        a_prime_index.insert(IndexType::Str("1_x".to_string()), 3);

        let mut b_prime_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        b_prime_names.insert(ColumnType::Str("a".to_string()), 0);
        b_prime_names.insert(ColumnType::Str("b".to_string()), 1);
        b_prime_names.insert(ColumnType::Str("c".to_string()), 2);
        b_prime_names.insert(ColumnType::Str("d".to_string()), 3);
        let a_prime = arr2(&[[2., 3.], [3., 4.], [7., 1.], [7., 6.]]);
        let b_prime = arr2(&[[2., 3., 7., 1.], [3., 4., 7., 6.]]);

        assert_eq!(DataFrame::new(a_prime)
                       .columns(a_prime_names)
                       .unwrap()
                       .index(a_prime_index)
                       .unwrap(),
                   row_concat.unwrap());
        assert_eq!(DataFrame::new(b_prime).columns(b_prime_names).unwrap(),
                   col_concat.unwrap())
    }

    #[test]
    fn dataframe_drop_column() {
        let a = arr2(&[[2., 3.], [3., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("a".to_string()), 0);
        names.insert(ColumnType::Str("b".to_string()), 1);
        let mut df = DataFrame::new(a).columns(names).unwrap();
        let new_df = df.drop_column(&[ColumnType::Str("a".to_string())]);
        let mut new_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        new_names.insert(ColumnType::Str("b".to_string()), 0);
        let a_prime = arr2(&[[3.], [4.]]);
        assert_eq!(DataFrame::new(a_prime).columns(new_names).unwrap(),
                   new_df.unwrap())
    }

    #[test]
    fn dataframe_drop_row() {
        let a = arr2(&[[2., 3.], [5., 4.]]);
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("a".to_string()), 0);
        names.insert(ColumnType::Str("b".to_string()), 1);
        let mut df = DataFrame::new(a).columns(names).unwrap();
        let new_df = df.drop_row(&[IndexType::Str("1".to_string())]);
        let mut new_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        new_names.insert(ColumnType::Str("a".to_string()), 0);
        new_names.insert(ColumnType::Str("b".to_string()), 1);
        let a_prime = arr2(&[[2., 3.]]);
        assert_eq!(DataFrame::new(a_prime).columns(new_names).unwrap(),
                   new_df.unwrap())
    }


    #[test]
    fn dataframe_creation_failure() {
        let a = Array::random((2, 5), Range::new(0., 10.));
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("1".to_string()), 0);
        names.insert(ColumnType::Str("2".to_string()), 1);
        let df = DataFrame::new(a).columns(names);
        assert!(df.is_err())
    }



    #[bench]
    fn bench_creation(b: &mut Bencher) {
        let a = Array::random((10, 5), Range::new(0., 10.));
        let mut names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        names.insert(ColumnType::Str("1".to_string()), 0);
        names.insert(ColumnType::Str("2".to_string()), 1);
        names.insert(ColumnType::Str("3".to_string()), 2);
        names.insert(ColumnType::Str("4".to_string()), 3);
        names.insert(ColumnType::Str("5".to_string()), 4);
        b.iter(|| DataFrame::new(a.clone()).columns(names.clone()));
    }



    #[bench]
    fn bench_inner_join(b: &mut Bencher) {
        let c = Array::random((20000, 10), Range::new(0., 10.));
        let e = Array::random((20000, 10), Range::new(0., 10.));


        let mut c_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        for i in 0..10 {
            c_names.insert(ColumnType::Str(i.to_string()), i);
        }

        let mut e_names: BTreeMap<ColumnType, usize> = BTreeMap::new();
        for i in 0..10 {
            e_names.insert(ColumnType::Str(i.to_string()), i);
        }

        let mut c_index: BTreeMap<IndexType, usize> = BTreeMap::new();
        for i in 0..20000 {
            c_index.insert(IndexType::Str(i.to_string()), i);
        }

        let mut e_index: BTreeMap<IndexType, usize> = BTreeMap::new();
        for i in 1999..21999 {
            e_index.insert(IndexType::Str(i.to_string()), i - 1999);
        }

        let c_df = DataFrame::new(c).columns(c_names).unwrap().index(c_index).unwrap();
        let e_df = DataFrame::new(e).columns(e_names).unwrap().index(e_index).unwrap();

        b.iter(|| c_df.inner_join(&e_df));
    }

    #[bench]
    fn bench_stack(b: &mut Bencher) {
        let a = Array::random((20000, 10), Range::new(0., 10.));
        let d = Array::random((20000, 10), Range::new(0., 10.));


        b.iter(|| stack(Axis(1), &[a.view(), d.view()]));
    }

    #[bench]
    fn bench_inner_join_bare(b: &mut Bencher) {
        let mut left = HashMap::new();
        for (i, j) in (1..20000).zip((1..20000).map(|x| x.to_string())) {
            left.insert(i, j);
        }

        let mut right = HashMap::new();
        for (i, j) in (19993..40000).zip((19993..40000).map(|x| x.to_string())) {
            right.insert(i, j);
        }

        b.iter(|| {
            let res: Vec<(i32, String, Option<String>)> =
                Join::new(JoinType::InnerJoin, left.clone().into_iter(), right.clone()).collect();
            res
        });

    }

}
