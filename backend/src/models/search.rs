use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct TmpSuggest {
    pub title: Option<String>,
    pub id: Option<String>,
}

pub async fn get_suggests_by_keyword(
    pool: &SqlitePool,
    params: &str,
) -> Result<Vec<TmpSuggest>, sqlx::Error> {
    let bind_value = format!("%{}%", params);

    let res = sqlx::query_as::<_, TmpSuggest>(
        r#"
            SELECT id, title FROM articles WHERE status = 'published' AND title LIKE ? 
        "#,
    )
    .bind(bind_value)
    .fetch_all(pool)
    .await?;
    tracing::info!("搜索建议结果: {:?}", res);
    // println!("{}", bind_value);
    // println!("{:?}", res);

    Ok(res)
}
