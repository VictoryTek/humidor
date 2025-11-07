use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tokio_postgres::Client;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub date: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub version: String,
    pub created_at: String,
    pub database_type: String,
}

pub async fn create_backup(db: &Client) -> Result<String, Box<dyn std::error::Error>> {
    // Create backups directory if it doesn't exist
    let backups_dir = Path::new("backups");
    fs::create_dir_all(backups_dir)?;

    // Generate timestamped backup filename
    let timestamp = Utc::now().format("%Y.%m.%d.%H.%M.%S").to_string();
    let backup_name = format!("humidor_{}.zip", timestamp);
    let backup_path = backups_dir.join(&backup_name);

    // Create ZIP file
    let file = File::create(&backup_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Export database to JSON
    let database_json = export_database(db).await?;

    // Add metadata
    let metadata = BackupMetadata {
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: Utc::now().to_rfc3339(),
        database_type: "postgresql".to_string(),
    };

    zip.start_file("metadata.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;

    // Add database JSON
    zip.start_file("database.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&database_json)?.as_bytes())?;

    // Add uploaded images if they exist
    let uploads_dir = Path::new("uploads");
    if uploads_dir.exists() {
        add_directory_to_zip(&mut zip, uploads_dir, "uploads", options)?;
    }

    zip.finish()?;

    Ok(backup_name)
}

pub async fn restore_backup(
    db: &Client,
    backup_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if backup_name is a full path or just a filename
    let backup_path = if backup_name.contains("/") || backup_name.contains("\\") {
        // It's a full path
        Path::new(backup_name).to_path_buf()
    } else {
        // It's just a filename, look in backups directory
        let backups_dir = Path::new("backups");
        backups_dir.join(backup_name)
    };

    if !backup_path.exists() {
        return Err("Backup file not found".into());
    }

    // Open ZIP file
    let file = File::open(&backup_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read and validate metadata
    let metadata: BackupMetadata = {
        let mut metadata_file = archive.by_name("metadata.json")?;
        let mut contents = String::new();
        metadata_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    tracing::info!(
        backup_created_at = %metadata.created_at,
        backup_version = %metadata.version,
        "Restoring backup"
    );

    // Read database JSON
    let database_json: serde_json::Value = {
        let mut db_file = archive.by_name("database.json")?;
        let mut contents = String::new();
        db_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Restore database
    import_database(db, &database_json).await?;

    // Restore uploads directory
    let uploads_dir = Path::new("uploads");
    if uploads_dir.exists() {
        fs::remove_dir_all(uploads_dir)?;
    }
    fs::create_dir_all(uploads_dir)?;

    // Extract files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.mangled_name();

        if file.name().starts_with("uploads/") {
            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    Ok(())
}

pub fn list_backups() -> Result<Vec<BackupInfo>, Box<dyn std::error::Error>> {
    let backups_dir = Path::new("backups");

    if !backups_dir.exists() {
        fs::create_dir_all(backups_dir)?;
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(backups_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            let metadata = fs::metadata(&path)?;
            let size = format_size(metadata.len());
            let modified = metadata.modified()?;
            let datetime: chrono::DateTime<Utc> = modified.into();

            backups.push(BackupInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                date: datetime.to_rfc3339(),
                size,
            });
        }
    }

    // Sort by date, newest first
    backups.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(backups)
}

pub fn delete_backup(backup_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backups_dir = Path::new("backups");
    let backup_path = backups_dir.join(backup_name);

    if !backup_path.exists() || !backup_path.is_file() {
        return Err("Backup file not found".into());
    }

    // Security check: ensure the path is within backups directory
    if !backup_path.starts_with(backups_dir) {
        return Err("Invalid backup path".into());
    }

    fs::remove_file(backup_path)?;
    Ok(())
}

async fn export_database(db: &Client) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut export = serde_json::Map::new();

    // Export all tables using pg_dump-like approach
    // For now, we'll use a simple approach: export as JSON strings
    let tables = vec![
        "users",
        "humidors",
        "brands",
        "sizes",
        "ring_gauges",
        "strengths",
        "origins",
        "cigars",
        "favorites",
        "wish_list",
    ];

    for table in tables {
        // Use json_agg to aggregate rows as JSON (returns text, not jsonb)
        let query = format!(
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM {} t",
            table
        );
        let row = db.query_one(&query, &[]).await?;

        // Get the JSON value as a string and parse it
        let json_str: String = row.get(0);
        let table_data: serde_json::Value = serde_json::from_str(&json_str)?;

        export.insert(table.to_string(), table_data);
    }

    Ok(serde_json::Value::Object(export))
}

async fn import_database(
    db: &Client,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let obj = data.as_object().ok_or("Invalid database JSON")?;

    // Disable foreign key checks temporarily
    db.execute("SET CONSTRAINTS ALL DEFERRED", &[]).await?;

    // Clear existing data (in reverse order of dependencies)
    let tables = vec![
        "wish_list",
        "favorites",
        "cigars",
        "origins",
        "strengths",
        "ring_gauges",
        "sizes",
        "brands",
        "humidors",
        "users",
    ];

    for table in &tables {
        let query = format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE", table);
        db.execute(&query, &[]).await?;
    }

    // Import data (in order of dependencies)
    let import_order = vec![
        "users",
        "humidors",
        "brands",
        "sizes",
        "ring_gauges",
        "strengths",
        "origins",
        "cigars",
        "favorites",
        "wish_list",
    ];

    for table in import_order {
        if let Some(rows) = obj.get(table).and_then(|v| v.as_array()) {
            for row in rows {
                // Convert JSON object to INSERT statement
                import_row(db, table, row).await?;
            }
        }
    }

    Ok(())
}

async fn import_row(
    db: &Client,
    table: &str,
    row: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use parameterized query to prevent SQL injection
    // PostgreSQL's tokio-postgres driver handles proper escaping
    let query = format!(
        "INSERT INTO {} SELECT * FROM json_populate_record(NULL::{}, $1::json)",
        table, table
    );

    // Convert to JSON value - the driver will serialize it safely
    let json_value = row.clone();

    tracing::debug!(
        table = %table,
        row_preview = %serde_json::to_string(&json_value)
            .unwrap_or_default()
            .chars()
            .take(100)
            .collect::<String>(),
        "Importing row into table"
    );

    match db.execute(&query, &[&json_value]).await {
        Ok(count) => {
            tracing::debug!(
                table = %table,
                rows_inserted = count,
                "Successfully inserted row"
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!(
                table = %table,
                error = %e,
                "Failed to insert row during backup restore"
            );
            Err(Box::new(e))
        }
    }
}

fn add_directory_to_zip(
    zip: &mut ZipWriter<File>,
    dir: &Path,
    prefix: &str,
    options: FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(dir.parent().unwrap_or(dir))?;
        let zip_path = format!("{}/{}", prefix, name.display());

        if path.is_file() {
            zip.start_file(&zip_path, options)?;
            let mut file = File::open(&path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() {
            zip.add_directory(&zip_path, options)?;
            add_directory_to_zip(zip, &path, prefix, options)?;
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
