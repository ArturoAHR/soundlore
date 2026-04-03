use tauri_plugin_sql::Migration;

pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_initial_schema.sql",
        sql: include_str!("../../migrations/0001_create_initial_schema.sql"),
        kind: tauri_plugin_sql::MigrationKind::Up,
    }]
}
