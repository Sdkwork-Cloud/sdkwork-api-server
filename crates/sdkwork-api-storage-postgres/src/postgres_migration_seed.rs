use super::*;

const BUILTIN_CHANNEL_SEEDS: [(&str, &str, i32); 5] = [
    ("openai", "OpenAI", 10),
    ("anthropic", "Anthropic", 20),
    ("gemini", "Gemini", 30),
    ("openrouter", "OpenRouter", 40),
    ("ollama", "Ollama", 50),
];

pub(crate) async fn seed_postgres_builtin_channels(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    for (channel_id, channel_name, sort_order) in BUILTIN_CHANNEL_SEEDS {
        sqlx::query(
            "INSERT INTO ai_channel (
                channel_id,
                channel_name,
                channel_description,
                sort_order,
                is_builtin,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, '', $3, TRUE, TRUE, 0, 0)
            ON CONFLICT (channel_id) DO UPDATE SET
                channel_name = EXCLUDED.channel_name,
                sort_order = EXCLUDED.sort_order,
                is_builtin = TRUE,
                is_active = TRUE",
        )
        .bind(channel_id)
        .bind(channel_name)
        .bind(sort_order)
        .execute(&pool)
        .await?;
    }
    Ok(())
}
