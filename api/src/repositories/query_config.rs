use deref_derive::{Deref, DerefMut};
use rocket::{
    async_trait,
    data::ToByteUnit,
    form::{self, DataField, FromFormField, ValueField},
    FromForm,
};
use std::fmt::Display;

#[derive(Debug, Clone, FromForm)]
pub struct QueryConfig {
    pub range: PageRange,
    pub items: Option<ItemCount>,
    pub sort: Option<String>,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, Deref, DerefMut)]
pub struct ItemCount(i64);

impl Default for ItemCount {
    fn default() -> Self {
        Self(20)
    }
}

#[async_trait]
impl<'r> FromFormField<'r> for ItemCount {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Ok(Self(field.value.parse()?))
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field
            .request
            .limits()
            .get("sort_order")
            .unwrap_or(256.kibibytes());

        // Read the capped data stream, returning a limit error as needed.
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        // Store the bytes in request-local cache.
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);
        let value =
            String::from_utf8(bytes.into()).map_err(|_| form::Error::validation("invalid_str"))?;

        Ok(Self(value.parse()?))
    }
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => f.write_str("asc"),
            SortOrder::Desc => f.write_str("desc"),
        }
    }
}

#[async_trait]
impl<'r> FromFormField<'r> for SortOrder {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(form::Error::validation("invalid_sort_order").into()),
        }
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field
            .request
            .limits()
            .get("sort_order")
            .unwrap_or(256.kibibytes());

        // Read the capped data stream, returning a limit error as needed.
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        // Store the bytes in request-local cache.
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);
        let value =
            String::from_utf8(bytes.into()).map_err(|_| form::Error::validation("invalid_str"))?;
        match value.as_str() {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(form::Error::validation("invalid_sort_order").into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PageRange {
    All,
    Single(i64),
    Range(i64, i64),
}

#[async_trait]
impl<'r> FromFormField<'r> for PageRange {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        // all, 1, 1-10
        match field.value {
            "all" => Ok(Self::All),
            value => match value.find('-') {
                Some(index) => {
                    let (start, end) = value.split_at(index);
                    let start = start.parse()?;
                    let end = end[1..].parse()?;
                    Ok(Self::Range(start, end))
                }
                None => Ok(Self::Single(value.parse()?)),
            },
        }
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field
            .request
            .limits()
            .get("page_range")
            .unwrap_or(256.kibibytes());

        // Read the capped data stream, returning a limit error as needed.
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        // Store the bytes in request-local cache.
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);
        let value =
            String::from_utf8(bytes.into()).map_err(|_| form::Error::validation("invalid_str"))?;
        match value.as_str() {
            "all" => Ok(Self::All),
            value => match value.find('-') {
                Some(index) => {
                    let (start, end) = value.split_at(index);
                    let start = start.parse()?;
                    let end = end[1..].parse()?;
                    Ok(Self::Range(start, end))
                }
                None => Ok(Self::Single(value.parse()?)),
            },
        }
    }
}

#[macro_export]
macro_rules! impl_query_config {
    ($dsl_table:expr, $table:ty, $boxed:ty, $type:ident, [$(($column:expr, $name:expr),)*]) => {
        use crate::{
            repositories::{
                query_config::{PageRange, QueryConfig, SortOrder},
            },
            error::{Error, ValidationError, QueryValidationError},
        };
        use diesel::{pg::Pg, query_builder::QueryFragment, AppearsOnTable, ExpressionMethods, QueryDsl};

        pub trait $type {
            fn get_query(&self) -> Result<$boxed, Error>;
        }

        impl $type for QueryConfig {
            fn get_query(&self) -> Result<$boxed, Error> {
                match (
                    &self.range,
                    &self.items,
                    &self.sort,
                    &self.order,
                ) {
                    // Proper All
                    (PageRange::All, None, None, None)
                    // All with invalid item count or invalid sort
                    | (PageRange::All, None, None, Some(_))
                    | (PageRange::All, None, Some(_), None)
                    | (PageRange::All, Some(_), None, None)
                    | (PageRange::All, Some(_), None, Some(_))
                    | (PageRange::All, Some(_), Some(_), None)
                    // Single with invalid item count or invalid sort
                    | (PageRange::Single(_), None, None, None)
                    | (PageRange::Single(_), None, None, Some(_))
                    | (PageRange::Single(_), None, Some(_), None)
                    // Range with invalid item count or invalid sort
                    | (PageRange::Range(_, _), None, None, None)
                    | (PageRange::Range(_, _), None, None, Some(_))
                    | (PageRange::Range(_, _), None, Some(_), None)
                     => {
                        Ok($dsl_table.into_boxed())
                    }
                    // All with valid sort
                    (PageRange::All, None, Some(column), Some(order))
                    // All with invalid item count and valid sort
                    | (PageRange::All, Some(_), Some(column), Some(order))
                    // Single with invalid item count and valid sort
                    | (PageRange::Single(_), None, Some(column), Some(order))
                    // Range with invalid item count and valid sort
                    | (PageRange::Range(_, _), None, Some(column), Some(order))
                     => {
                        Ok(sort_by_column_str($dsl_table.into_boxed(), column.as_str(), &order)?)
                    }
                    // Single with valid item count
                    (PageRange::Single(i), Some(count), None, None)
                    // Single with valid item count and invalid sort
                    | (PageRange::Single(i), Some(count), None, Some(_))
                    | (PageRange::Single(i), Some(count), Some(_), None)
                     => {
                        if *i <= 0 {
                            return Ok($dsl_table.limit(**count).into_boxed());
                        }
                        Ok($dsl_table.limit(**count).offset(**count * *i).into_boxed())
                    }
                    // Single with valid item count and valid sort
                    (PageRange::Single(i), Some(count), Some(column), Some(order)) => {
                        if *i <= 0 {
                            return Ok(sort_by_column_str($dsl_table.into_boxed(), column, order)?.limit(**count));
                        }
                        Ok(sort_by_column_str($dsl_table.into_boxed(), column, order)?.limit(**count).offset(**count * *i))
                    }
                    // Range with valid item count
                    (PageRange::Range(start, end), Some(count), None, None)
                    // Range with valid item count and invalid sort
                    | (PageRange::Range(start, end), Some(count), None, Some(_))
                    | (PageRange::Range(start, end), Some(count), Some(_), None)
                     => {
                        let pages = (end - start).max(0) + 1;
                        if *start <= 0 {
                            return Ok($dsl_table.limit(**count * pages).into_boxed());
                        }
                        Ok($dsl_table.limit(**count * pages).offset(**count * start).into_boxed())
                    }
                    // Range with valid item count and valid sort
                    (PageRange::Range(start, end), Some(count), Some(column), Some(sort)) => {
                        let pages = (end - start).max(0) + 1;
                        if *start <= 0 {
                            return Ok(sort_by_column_str($dsl_table.into_boxed(), column.as_str(), &sort)?.limit(**count * pages));
                        }
                        Ok(sort_by_column_str($dsl_table.into_boxed(), column.as_str(), &sort)?.limit(**count * pages).offset(**count * start))
                    }
                }
            }
        }

        fn sort_by_column<U: 'static + Send + Sync + AppearsOnTable<$table>>(
            query: $boxed,
            column: U,
            order: &SortOrder,
        ) -> $boxed
        where
            U: ExpressionMethods + QueryFragment<Pg>,
        {
            match order {
                SortOrder::Asc => query.order(column.asc()),
                SortOrder::Desc => query.order(column.desc()),
            }
        }

        fn sort_by_column_str(
            query: $boxed,
            column: &str,
            order: &SortOrder,
        ) -> Result<$boxed, Error> {
            match column {
                $(
                    $name => Ok(sort_by_column(query, $column, order)),
                )*
                _ => {
                    return Err(Error::ValidationError(ValidationError::Query(
                        QueryValidationError::InvalidColumn(column.to_string()),
                    )))
                }
            }
        }
    };
}
