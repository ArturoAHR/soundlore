use sea_query::{Asterisk, Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    track::models::{Track, TrackIden},
};

pub async fn get_tracks(pool: SqlitePool) -> Result<Vec<Track>, AppError> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .and_where(Expr::col(TrackIden::DeletedAt).is_null())
        .build_sqlx(SqliteQueryBuilder);

    let tracks = sqlx::query_as_with::<_, Track, _>(&sql, values)
        .fetch_all(&pool)
        .await?;

    Ok(tracks)
}
