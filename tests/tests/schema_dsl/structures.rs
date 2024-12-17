use diesel::RunQueryDsl;
use diesel_mssql::Mssql;
use std::marker::PhantomData;

pub struct CreateTable<'a, Cols> {
    name: &'a str,
    columns: Cols,
}

impl<'a, Cols> CreateTable<'a, Cols> {
    pub fn new(name: &'a str, columns: Cols) -> Self {
        CreateTable { name, columns }
    }
}

impl<'a, Cols, Conn> RunQueryDsl<Conn> for CreateTable<'a, Cols> {}

pub struct Column<'a, T> {
    name: &'a str,
    type_name: &'a str,
    _marker: PhantomData<T>,
}

impl<'a, T> Column<'a, T> {
    pub fn new(name: &'a str, type_name: &'a str) -> Self {
        Column {
            name,
            type_name,
            _marker: PhantomData,
        }
    }

    pub fn primary_key(self) -> PrimaryKey<Self> {
        PrimaryKey(self)
    }

    pub fn default(self, expr: &str) -> Default<'_, Self> {
        Default {
            column: self,
            value: expr,
        }
    }

    pub fn not_null(self) -> NotNull<Self> {
        NotNull(self)
    }
}

pub struct PrimaryKey<Col>(Col);

impl<Col> PrimaryKey<Col> {
    pub fn auto_increment(self) -> AutoIncrement<Self> {
        AutoIncrement(self)
    }
}

pub struct AutoIncrement<Col>(Col);

pub struct NotNull<Col>(Col);

impl<'a, T> NotNull<Column<'a, T>> {
    pub fn default(self, expr: &str) -> Default<'_, Self> {
        Default {
            column: self,
            value: expr,
        }
    }
}

pub struct Default<'a, Col> {
    column: Col,
    value: &'a str,
}

use diesel::backend::*;
use diesel::query_builder::*;
use diesel::result::QueryResult;

impl<'a, DB, Cols> QueryFragment<DB> for CreateTable<'a, Cols>
where
    DB: Backend,
    Cols: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();

        out.push_sql("IF OBJECT_ID('");
        out.push_identifier(self.name)?;
        out.push_sql("') IS NULL CREATE TABLE ");
        out.push_identifier(self.name)?;
        out.push_sql(" (");
        self.columns.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<'a, Cols> QueryId for CreateTable<'a, Cols> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<'a, DB, T> QueryFragment<DB> for Column<'a, T>
where
    DB: Backend,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        out.push_identifier(self.name)?;
        out.push_sql(" ");
        out.push_sql(self.type_name);
        Ok(())
    }
}

impl<'a, Cols> QueryId for Column<'a, Cols> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<DB, Col> QueryFragment<DB> for PrimaryKey<Col>
where
    DB: Backend,
    Col: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        self.0.walk_ast(out.reborrow())?;
        out.push_sql(" PRIMARY KEY");
        Ok(())
    }
}

impl<Col> QueryId for PrimaryKey<Col> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<Col> QueryId for AutoIncrement<Col> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<'a> QueryFragment<Mssql>
    for AutoIncrement<PrimaryKey<Column<'a, diesel::sql_types::Integer>>>
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Mssql>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        out.push_identifier((self.0).0.name)?;
        out.push_sql(" INT IDENTITY(1,1) PRIMARY KEY");
        Ok(())
    }
}

impl<DB, Col> QueryFragment<DB> for NotNull<Col>
where
    DB: Backend,
    Col: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        self.0.walk_ast(out.reborrow())?;
        out.push_sql(" NOT NULL");
        Ok(())
    }
}

impl<Col> QueryId for NotNull<Col> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<'a, DB, Col> QueryFragment<DB> for Default<'a, Col>
where
    DB: Backend,
    Col: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        self.column.walk_ast(out.reborrow())?;
        out.push_sql(" DEFAULT ");
        out.push_sql(self.value);
        Ok(())
    }
}

impl<'a, Col> QueryId for Default<'a, Col> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}