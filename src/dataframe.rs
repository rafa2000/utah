use error::*;
use types::*;
use std::string::ToString;
use std::iter::Iterator;
use aggregate::*;
use transform::*;
use ndarray::{Axis, Array};
use types::UtahAxis;
use process::*;
use std::iter::FromIterator;

/// A read-only dataframe.
#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
    pub columns: Vec<OuterType>,
    pub data: Matrix<InnerType>,
    pub index: Vec<OuterType>,
}

/// A read-write dataframe
#[derive(Debug, PartialEq)]
pub struct MutableDataFrame<'a> {
    pub columns: Vec<OuterType>,
    pub data: MatrixMut<'a, InnerType>,
    pub index: Vec<OuterType>,
}



// and we'll implement FromIterator
impl<'a> FromIterator<(OuterType, RowView<'a, InnerType>)> for DataFrame {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = (OuterType, RowView<'a, InnerType>)>
    {
        let mut c = Vec::new();
        let mut n = Vec::new();
        let mut nrows = 0;
        let mut ncols = 0;
        for (i, j) in iter {
            nrows = j.len();
            ncols += 1;
            c.extend(j);
            n.push(i);
        }

        let columns: Vec<OuterType> = (0..ncols).map(OuterType::from).collect();
        let index: Vec<OuterType> = (0..nrows).map(OuterType::from).collect();

        DataFrame {
            columns: columns,
            data: Array::from_shape_vec((nrows, ncols), c).unwrap().mapv(|x| x.to_owned()),
            index: index,
        }
    }
}

impl<'a> FromIterator<(OuterType, RowViewMut<'a, InnerType>)> for MutableDataFrame<'a> {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = (OuterType, RowViewMut<'a, InnerType>)>
    {
        let mut c = Vec::new();
        let mut n = Vec::new();
        let mut nrows = 0;
        let mut ncols = 0;
        for (i, j) in iter {
            nrows = j.len();
            ncols += 1;
            c.extend(j);
            n.push(i);
        }
        let index: Vec<OuterType> = (0..nrows).map(OuterType::from).collect();
        let data = Array::from_shape_vec((nrows, ncols), c).unwrap();
        MutableDataFrame {
            columns: n.clone(),
            data: data,
            index: index,
        }
    }
}

impl DataFrame {
    /// Create a new dataframe. The only required argument is data to populate the dataframe. The data's elements can be any of `InnerType`.
    /// By default, the columns and index of the dataframe are `["1", "2", "3"..."N"]`, where *N* is
    /// the number of columns (or rows) in the data.
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a);
    /// ```
    ///
    /// When populating the dataframe with mixed-types, wrap the elements with `InnerType` enum:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[InnerType::Float(2.0), InnerType::Str("ak".into())],
    ///                [InnerType::Int32(6), InnerType::Int64(10)]]);
    /// let df = DataFrame::new(a);
    /// ```
    pub fn new<T: Clone>(data: Matrix<T>) -> DataFrame
        where InnerType: From<T>
    {
        let data: Matrix<InnerType> = data.mapv(InnerType::from);
        let data: Matrix<InnerType> = data.mapv_into(|x| {
            match x {
                InnerType::Float(y) => {
                    if y.is_nan() {
                        return InnerType::Empty;
                    } else {
                        return x;
                    }
                }
                _ => return x,
            }

        });
        let columns: Vec<OuterType> = (0..data.shape()[1])
            .map(|x| OuterType::Str(x.to_string()))
            .collect();

        let index: Vec<OuterType> = (0..data.shape()[0])
            .map(|x| OuterType::Str(x.to_string()))
            .collect();

        DataFrame {
            data: data,
            columns: columns,
            index: index,
        }
    }

    /// Populate the dataframe with a set of columns. The column elements can be any of `OuterType`. Example:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a).columns(&["a", "b"]);
    /// df.is_ok();
    /// ```
    pub fn columns<'a, T>(mut self, columns: &'a [T]) -> Result<DataFrame>
        where OuterType: From<&'a T>
    {
        if columns.len() != self.data.shape()[1] {
            return Err(ErrorKind::ColumnShapeMismatch.into());
        }
        self.columns = columns.iter()
            .map(|x| OuterType::from(x))
            .collect();
        Ok(self)
    }

    /// Populate the dataframe with an index. The index elements can be any of `OuterType`. Example:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2]);
    /// df.is_ok();
    /// ```
    ///
    /// You can also populate the dataframe with both column names and index names, like so:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2]).columns(&["a", "b"]).unwrap();
    /// ```
    pub fn index<'a, T>(mut self, index: &'a [T]) -> Result<DataFrame>
        where OuterType: From<&'a T>
    {
        if index.len() != self.data.shape()[0] {
            return Err(ErrorKind::RowShapeMismatch.into());
        }
        self.index = index.iter()
            .map(|x| OuterType::from(x))
            .collect();
        Ok(self)
    }

    /// Get the dimensions of the dataframe.
    pub fn shape(self) -> (usize, usize) {
        self.data.dim()
    }


    /// Return a dataframe iterator over the specified `UtahAxis`.
    ///
    /// The dataframe iterator yields a mutable view of a row or column of the dataframe for
    /// computation. Example:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2]).columns(&["a", "b"]).unwrap();
    /// ```
    pub fn df_iter<'a>(&'a self, axis: UtahAxis) -> DataFrameIterator<'a> {
        match axis {
            UtahAxis::Row => {
                DataFrameIterator {
                    names: self.index.iter(),
                    data: self.data.axis_iter(Axis(0)),
                    other: self.columns.clone(),
                    axis: UtahAxis::Row,
                }
            }
            UtahAxis::Column => {
                DataFrameIterator {
                    names: self.columns.iter(),
                    data: self.data.axis_iter(Axis(1)),
                    other: self.index.to_owned(),
                    axis: UtahAxis::Column,
                }
            }
        }
    }

    pub fn df_iter_mut<'a>(&'a mut self, axis: UtahAxis) -> MutableDataFrameIterator<'a> {
        match axis {
            UtahAxis::Row => {
                MutableDataFrameIterator {
                    names: self.index.iter(),
                    data: self.data.axis_iter_mut(Axis(0)),
                    axis: UtahAxis::Row,
                    other: self.columns.clone(),
                }
            }
            UtahAxis::Column => {
                MutableDataFrameIterator {
                    names: self.columns.iter(),
                    data: self.data.axis_iter_mut(Axis(1)),
                    axis: UtahAxis::Row,
                    other: self.index.clone(),
                }
            }
        }
    }
    /// Select rows or columns over the specified `UtahAxis`.
    ///
    /// The Select transform adaptor yields a mutable view of a row or column of the dataframe for
    /// computation
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.select(&["a", "c"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```

    pub fn select<'a, T>(&'a self,
                         names: &'a [T],
                         axis: UtahAxis)
                         -> Select<'a, DataFrameIterator<'a>>
        where OuterType: From<&'a T>
    {
        let names = names.iter()
            .map(|x| OuterType::from(x))
            .collect();
        match axis {
            UtahAxis::Row => {
                Select::new(self.df_iter(UtahAxis::Row),
                            names,
                            self.columns.clone(),
                            UtahAxis::Row)
            }
            UtahAxis::Column => {
                Select::new(self.df_iter(UtahAxis::Column),
                            names,
                            self.index.clone(),
                            UtahAxis::Column)
            }
        }
    }

    /// Remove rows or columns over the specified `UtahAxis`.
    ///
    /// The Remove transform adaptor yields a mutable view of a row or column of the dataframe for
    /// computation.
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn remove<'a, T>(&'a mut self,
                         names: &'a [T],
                         axis: UtahAxis)
                         -> Remove<'a, DataFrameIterator<'a>>
        where OuterType: From<&'a T>
    {
        let names = names.iter()
            .map(|x| OuterType::from(x))
            .collect();
        match axis {
            UtahAxis::Row => {
                Remove::new(self.df_iter(UtahAxis::Row),
                            names,
                            self.columns.clone(),
                            UtahAxis::Row)
            }
            UtahAxis::Column => {
                Remove::new(self.df_iter(UtahAxis::Column),
                            names,
                            self.index.clone(),
                            UtahAxis::Column)
            }
        }
    }

    /// Append  a row or column along the specified `UtahAxis`.
    ///
    /// The Remove transform adaptor yields a mutable view of a row or column of the dataframe for
    /// computation.
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn append<'a, T>(&'a mut self,
                         name: &'a T,
                         data: RowView<'a, InnerType>,
                         axis: UtahAxis)
                         -> Append<'a, DataFrameIterator<'a>>
        where OuterType: From<&'a T>
    {
        let name = OuterType::from(name);
        match axis {
            UtahAxis::Row => {
                Append::new(self.df_iter(UtahAxis::Row),
                            name,
                            data,
                            self.columns.clone(),
                            UtahAxis::Row)
            }
            UtahAxis::Column => {
                Append::new(self.df_iter(UtahAxis::Column),
                            name,
                            data,
                            self.index.clone(),
                            UtahAxis::Column)
            }

        }
    }


    /// Perform an inner left join between two dataframes along the specified `UtahAxis`.
    ///

    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn inner_left_join<'a>(&'a mut self,
                               other: &'a mut DataFrame,
                               axis: UtahAxis)
                               -> InnerJoin<'a, DataFrameIterator<'a>> {
        match axis {
            UtahAxis::Row => {
                InnerJoin::new(self.df_iter(UtahAxis::Row), other.df_iter(UtahAxis::Row))
            }
            UtahAxis::Column => {
                InnerJoin::new(self.df_iter(UtahAxis::Column),
                               other.df_iter(UtahAxis::Column))
            }
        }
    }

    /// Perform an outer left join between two dataframes along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn outer_left_join<'a>(&'a mut self,
                               other: &'a mut DataFrame,
                               axis: UtahAxis)
                               -> OuterJoin<'a, DataFrameIterator<'a>> {
        match axis {
            UtahAxis::Row => {
                OuterJoin::new(self.df_iter(UtahAxis::Row), other.df_iter(UtahAxis::Row))
            }
            UtahAxis::Column => {
                OuterJoin::new(self.df_iter(UtahAxis::Column),
                               other.df_iter(UtahAxis::Column))
            }

        }

    }

    /// Perform an inner right join between two dataframes along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn inner_right_join<'a>(&'a mut self,
                                other: &'a mut DataFrame,
                                axis: UtahAxis)
                                -> InnerJoin<'a, DataFrameIterator<'a>> {
        match axis {
            UtahAxis::Row => {
                InnerJoin::new(other.df_iter(UtahAxis::Row), self.df_iter(UtahAxis::Row))
            }
            UtahAxis::Column => {
                InnerJoin::new(other.df_iter(UtahAxis::Column),
                               self.df_iter(UtahAxis::Column))
            }

        }
    }

    /// Perform an outer right join between two dataframes along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn outer_right_join<'a>(&'a mut self,
                                other: &'a mut DataFrame,
                                axis: UtahAxis)
                                -> OuterJoin<'a, DataFrameIterator<'a>> {
        match axis {
            UtahAxis::Row => {
                OuterJoin::new(other.df_iter(UtahAxis::Row), self.df_iter(UtahAxis::Row))
            }
            UtahAxis::Column => {
                OuterJoin::new(other.df_iter(UtahAxis::Column),
                               self.df_iter(UtahAxis::Column))
            }

        }
    }


    /// Sum along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn sumdf<'a>(&'a mut self, axis: UtahAxis) -> Sum<'a, DataFrameIterator<'a>> {

        match axis {
            UtahAxis::Row => Sum::new(self.df_iter(UtahAxis::Row)),
            UtahAxis::Column => Sum::new(self.df_iter(UtahAxis::Column)),

        }
    }

    /// Map a function along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn map<'a, F, B>(&'a mut self,
                         f: F,
                         axis: UtahAxis)
                         -> MapDF<'a, DataFrameIterator<'a>, F, B>
        where F: Fn(&InnerType) -> B
    {

        match axis {
            UtahAxis::Row => MapDF::new(self.df_iter(UtahAxis::Row), f, UtahAxis::Row),
            UtahAxis::Column => MapDF::new(self.df_iter(UtahAxis::Column), f, UtahAxis::Column),

        }
    }

    /// Get the average of entries along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn mean<'a>(&'a mut self, axis: UtahAxis) -> Mean<'a, DataFrameIterator<'a>> {

        match axis {
            UtahAxis::Row => Mean::new(self.df_iter(UtahAxis::Row)),
            UtahAxis::Column => Mean::new(self.df_iter(UtahAxis::Column)),

        }
    }

    /// Get the maximum of entries along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn max<'a>(&'a mut self, axis: UtahAxis) -> Max<'a, DataFrameIterator<'a>> {

        match axis {
            UtahAxis::Row => Max::new(self.df_iter(UtahAxis::Row)),
            UtahAxis::Column => Max::new(self.df_iter(UtahAxis::Column)),

        }
    }

    /// Get the minimum of entries along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn min<'a>(&'a mut self, axis: UtahAxis) -> Min<'a, DataFrameIterator<'a>> {

        match axis {
            UtahAxis::Row => Min::new(self.df_iter(UtahAxis::Row)),
            UtahAxis::Column => Min::new(self.df_iter(UtahAxis::Column)),

        }
    }

    /// Get the standard deviation along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn stdev<'a>(&'a mut self, axis: UtahAxis) -> Stdev<'a, DataFrameIterator<'a>> {

        Stdev::new(self.df_iter(axis))


    }
    /// Get the standard deviation along the specified `UtahAxis`.
    ///
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0], [2.0, 8.0]]);
    /// let df = DataFrame::new(a).index(&[1, 2, 3]).columns(&["a", "b"]).unwrap();
    /// for (idx, row) in df.remove(&["b"], UtahAxis::Column) {
    ///        assert_eq!(row, a.row(idx))
    ///    }
    /// ```
    pub fn impute<'a>(&'a mut self,
                      strategy: ImputeStrategy,
                      axis: UtahAxis)
                      -> Impute<'a, MutableDataFrameIterator<'a>> {

        let index = self.index.clone();
        let columns = self.columns.clone();
        match axis {
            UtahAxis::Row => {
                Impute::new(self.df_iter_mut(UtahAxis::Row),
                            strategy,
                            columns,
                            UtahAxis::Row)
            }
            UtahAxis::Column => {
                Impute::new(self.df_iter_mut(UtahAxis::Column),
                            strategy,
                            index,
                            UtahAxis::Column)
            }

        }
    }
}



impl<'a> MutableDataFrame<'a> {
    /// Create a new dataframe. The only required argument is data to populate the dataframe. The data's elements can be any of `InnerType`.
    /// By default, the columns and index of the dataframe are `["1", "2", "3"..."N"]`, where *N* is
    /// the number of columns (or rows) in the data.
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[2.0, 7.0], [3.0, 4.0]]);
    /// let df = DataFrame::new(a);
    /// ```
    ///
    /// When populating the dataframe with mixed-types, wrap the elements with `InnerType` enum:
    ///
    /// ```
    /// use ndarray::arr2;
    /// use dataframe::DataFrame;
    ///
    /// let a = arr2(&[[InnerType::Float(2.0), InnerType::Str("ak".into())],
    ///                [InnerType::Int32(6), InnerType::Int64(10)]]);
    /// let df = DataFrame::new(a);
    /// ```
    pub fn to_df(self) -> DataFrame {
        let d = self.data.map(|x| (x.as_ref().clone()));
        DataFrame {
            data: d,
            columns: self.columns,
            index: self.index,
        }
    }
}
// To implement....?
// // parallelized join
// // parallelized concatenation
// // parallelized frequency counts
// // index dataframe?
// // sample rows
// // find/select
// // sort
// // statistics (mean, median, stdev)
// // print
//
// // statistics (mean, median, stdev)
// // print
