use super::Mssql;
use diesel::row::{Field, RowSealed};
use diesel::row::{PartialRow, RowIndex};
use tiberius::{Column, ColumnData};

pub struct MssqlField<'a> {
    column_info: Option<&'a Column>,
    field_value: &'a ColumnData<'a>,
}

struct FromSqlHelper<'a>(&'a ColumnData<'static>);

impl<'a> tiberius::FromSql<'a> for FromSqlHelper<'a> {
    fn from_sql(value: &'a ColumnData<'static>) -> Result<Option<Self>, tiberius::error::Error> {
        Ok(Some(Self(value)))
    }
}

impl<'a> Field<'a, Mssql> for MssqlField<'a> {
    fn field_name(&self) -> Option<&str> {
        if let Some(value) = self.column_info {
            return Some(value.name());
        }
        None
    }

    fn value(&self) -> Option<<Mssql as diesel::backend::Backend>::RawValue<'_>> {
        if let ColumnData::I32(value) = &self.field_value {
            if value.is_none() {
                return None;
            }
        }
        if let ColumnData::String(value) = &self.field_value {
            if value.is_none() {
                return None;
            }
        }
        Some(self.field_value.clone())
    }
}

pub struct MssqlRow {
    pub inner_row: tiberius::Row,
}

impl RowSealed for MssqlRow {}

impl RowIndex<usize> for MssqlRow {
    fn idx(&self, idx: usize) -> Option<usize> {
        if idx < self.inner_row.columns().len() {
            Some(idx)
        } else {
            None
        }
    }
}
impl<'a> RowIndex<&'a str> for MssqlRow {
    fn idx(&self, idx: &'a str) -> Option<usize> {
        for (i, col) in self.inner_row.columns().iter().enumerate() {
            if col.name() == idx {
                return Some(i);
            }
        }
        None
    }
}

impl<'a> diesel::row::Row<'a, Mssql> for MssqlRow {
    type Field<'f>
        = MssqlField<'f>
    where
        'a: 'f,
        Self: 'f;

    type InnerPartialRow = Self;

    fn field_count(&self) -> usize {
        self.inner_row.columns().len()
    }

    fn get<'b, I>(&'b self, idx: I) -> Option<Self::Field<'b>>
    where
        'a: 'b,
        Self: diesel::row::RowIndex<I>,
    {
        let idx = self.idx(idx).unwrap();
        let row = &self.inner_row;
        let col = row.columns().get(idx);
        let cell = row.try_get::<FromSqlHelper, _>(idx);
        match cell {
            Ok(value) => value.map(|value| MssqlField {
                        column_info: col,
                        field_value: value.0,
                    }),
            Err(_) => None,
        }
    }

    fn partial_row(
        &self,
        range: std::ops::Range<usize>,
    ) -> diesel::row::PartialRow<'_, Self::InnerPartialRow> {
        PartialRow::new(self, range)
    }

    fn get_value<ST, T, I>(&self, idx: I) -> diesel::deserialize::Result<T>
    where
        Self: diesel::row::RowIndex<I>,
        T: diesel::deserialize::FromSql<ST, Mssql>,
    {
        let field = self.get(idx).ok_or(diesel::result::UnexpectedEndOfRow)?;
        <T as diesel::deserialize::FromSql<ST, Mssql>>::from_nullable_sql(field.value())
    }
}
