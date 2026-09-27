//! Per-user data export. Every query is filtered by `user_id`: cigars have no `user_id` column, so
//! owned cigars are reached through `humidors.user_id`; wish list and favorites through their own
//! `user_id`. Images are deliberately excluded (`image_url` can be a multi-MB base64 data URL).

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio_postgres::{Client, Row};
use uuid::Uuid;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct ExportedHumidor {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Descriptive cigar fields shared by the collection, wish list and favorites exports.
/// `quantity` / `in_stock` are `None` only for favorites whose cigar no longer exists.
#[derive(Debug, Serialize)]
pub struct CigarDetails {
    pub name: String,
    pub brand: Option<String>,
    pub size: Option<String>,
    pub length_inches: Option<f64>,
    pub ring_gauge: Option<i32>,
    pub strength: Option<String>,
    pub origin: Option<String>,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub quantity: Option<i32>,
    pub in_stock: Option<bool>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub retail_link: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OwnedCigar {
    pub humidor: String,
    pub favorite: bool,
    #[serde(flatten)]
    pub details: CigarDetails,
}

#[derive(Debug, Serialize)]
pub struct WishListEntry {
    pub added_at: DateTime<Utc>,
    pub list_notes: Option<String>,
    #[serde(flatten)]
    pub details: CigarDetails,
}

#[derive(Debug, Serialize)]
pub struct FavoriteEntry {
    pub added_at: DateTime<Utc>,
    /// false when the cigar was deleted; only the name/organizer snapshot remains
    pub cigar_still_exists: bool,
    #[serde(flatten)]
    pub details: CigarDetails,
}

#[derive(Debug, Serialize)]
pub struct UserExport {
    pub format_version: u32,
    pub exported_at: DateTime<Utc>,
    pub humidors: Vec<ExportedHumidor>,
    pub cigars: Vec<OwnedCigar>,
    pub favorites: Vec<FavoriteEntry>,
    pub wish_list: Vec<WishListEntry>,
}

// Column list shared by the cigar queries; `c` = cigars, and the joins below provide organizer names.
const DETAIL_COLUMNS: &str = "b.name AS brand, s.name AS size, c.length, rg.gauge, st.name AS strength, \
     o.name AS origin, c.wrapper, c.binder, c.filler, c.quantity, c.is_active, c.price, \
     c.purchase_date, c.retail_link, c.notes";

const ORGANIZER_JOINS: &str = "LEFT JOIN brands b ON b.id = c.brand_id \
     LEFT JOIN sizes s ON s.id = c.size_id \
     LEFT JOIN strengths st ON st.id = c.strength_id \
     LEFT JOIN origins o ON o.id = c.origin_id \
     LEFT JOIN ring_gauges rg ON rg.id = c.ring_gauge_id";

/// Read the cigar description from a row that selected `c.name` (or a COALESCE of it) at `name_idx`
/// followed by the columns of `DETAIL_COLUMNS` in order.
fn details_from_row(row: &Row, name_idx: usize) -> CigarDetails {
    let i = name_idx;
    CigarDetails {
        name: row.get(i),
        brand: row.get(i + 1),
        size: row.get(i + 2),
        length_inches: row.get(i + 3),
        ring_gauge: row.get(i + 4),
        strength: row.get(i + 5),
        origin: row.get(i + 6),
        wrapper: row.get(i + 7),
        binder: row.get(i + 8),
        filler: row.get(i + 9),
        quantity: row.get(i + 10),
        in_stock: row.get(i + 11),
        price: row.get(i + 12),
        purchase_date: row.get(i + 13),
        retail_link: row.get(i + 14),
        notes: row.get(i + 15),
    }
}

pub async fn collect_user_export(
    db: &Client,
    user_id: Uuid,
) -> Result<UserExport, tokio_postgres::Error> {
    let humidors = db
        .query(
            "SELECT id, name, description, location, capacity, target_humidity, created_at \
             FROM humidors WHERE user_id = $1 ORDER BY name",
            &[&user_id],
        )
        .await?
        .iter()
        .map(|r| ExportedHumidor {
            id: r.get(0),
            name: r.get(1),
            description: r.get(2),
            location: r.get(3),
            capacity: r.get(4),
            target_humidity: r.get(5),
            created_at: r.get(6),
        })
        .collect();

    let owned_sql = format!(
        "SELECT h.name, c.name, {DETAIL_COLUMNS}, \
                EXISTS (SELECT 1 FROM favorites f WHERE f.user_id = $1 AND f.cigar_id = c.id) \
         FROM cigars c \
         JOIN humidors h ON h.id = c.humidor_id AND h.user_id = $1 \
         {ORGANIZER_JOINS} \
         ORDER BY h.name, b.name, c.name"
    );
    let cigars = db
        .query(&owned_sql, &[&user_id])
        .await?
        .iter()
        .map(|r| OwnedCigar {
            humidor: r.get(0),
            details: details_from_row(r, 1),
            favorite: r.get(17),
        })
        .collect();

    let wish_sql = format!(
        "SELECT w.created_at, w.notes, c.name, {DETAIL_COLUMNS} \
         FROM wish_list w \
         JOIN cigars c ON c.id = w.cigar_id \
         {ORGANIZER_JOINS} \
         WHERE w.user_id = $1 \
         ORDER BY w.created_at DESC"
    );
    let wish_list = db
        .query(&wish_sql, &[&user_id])
        .await?
        .iter()
        .map(|r| WishListEntry {
            added_at: r.get(0),
            list_notes: r.get(1),
            details: details_from_row(r, 2),
        })
        .collect();

    // Favorites keep a snapshot of the cigar's name/organizers, so they survive the cigar's deletion.
    let fav_sql = "SELECT f.created_at, c.id IS NOT NULL, COALESCE(c.name, f.snapshot_name, '(unknown)'), \
                b.name, s.name, c.length, rg.gauge, st.name, o.name, \
                c.wrapper, c.binder, c.filler, c.quantity, c.is_active, c.price, \
                c.purchase_date, c.retail_link, c.notes \
         FROM favorites f \
         LEFT JOIN cigars c ON c.id = f.cigar_id \
         LEFT JOIN brands b ON b.id = COALESCE(c.brand_id, f.snapshot_brand_id) \
         LEFT JOIN sizes s ON s.id = COALESCE(c.size_id, f.snapshot_size_id) \
         LEFT JOIN strengths st ON st.id = COALESCE(c.strength_id, f.snapshot_strength_id) \
         LEFT JOIN origins o ON o.id = COALESCE(c.origin_id, f.snapshot_origin_id) \
         LEFT JOIN ring_gauges rg ON rg.id = COALESCE(c.ring_gauge_id, f.snapshot_ring_gauge_id) \
         WHERE f.user_id = $1 \
         ORDER BY f.created_at DESC";
    let favorites = db
        .query(fav_sql, &[&user_id])
        .await?
        .iter()
        .map(|r| FavoriteEntry {
            added_at: r.get(0),
            cigar_still_exists: r.get(1),
            details: details_from_row(r, 2),
        })
        .collect();

    Ok(UserExport {
        format_version: FORMAT_VERSION,
        exported_at: Utc::now(),
        humidors,
        cigars,
        favorites,
        wish_list,
    })
}

// ---------------------------------------------------------------------------------------------
// CSV (RFC 4180) with spreadsheet formula-injection protection (OWASP CSV Injection)
// ---------------------------------------------------------------------------------------------

/// Prepended so Excel detects UTF-8 (accents such as "Padrón"). Programmatic readers can use `utf-8-sig`.
const UTF8_BOM: &str = "\u{feff}";

const CSV_HEADER: [&str; 19] = [
    "list",
    "humidor",
    "brand",
    "name",
    "size",
    "length_inches",
    "ring_gauge",
    "strength",
    "origin",
    "wrapper",
    "binder",
    "filler",
    "quantity",
    "in_stock",
    "price",
    "purchase_date",
    "retail_link",
    "favorite",
    "notes",
];

/// Encode a free-text cell. Cells that a spreadsheet could interpret as a formula (leading `=`,
/// `+`, `-`, `@`, tab, CR or LF) get a leading tab inside the quoted field (OWASP's recommended
/// mitigation; the tab remains part of the value). Fields containing `,`, `"`, CR or LF are quoted
/// and `"` is doubled, per RFC 4180.
fn csv_text(value: &str) -> String {
    let dangerous = matches!(
        value.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r' | '\n')
    );
    let needs_quotes = dangerous || value.contains([',', '"', '\r', '\n']);
    let mut out = String::with_capacity(value.len() + 3);
    if needs_quotes {
        out.push('"');
        if dangerous {
            out.push('\t');
        }
        out.push_str(&value.replace('"', "\"\""));
        out.push('"');
    } else {
        out.push_str(value);
    }
    out
}

fn opt_text(value: &Option<String>) -> String {
    value.as_deref().map(csv_text).unwrap_or_default()
}

fn opt_num<T: ToString>(value: &Option<T>) -> String {
    value.as_ref().map(|v| v.to_string()).unwrap_or_default()
}

fn yes_no(value: Option<bool>) -> String {
    value
        .map(|v| if v { "yes" } else { "no" }.to_string())
        .unwrap_or_default()
}

fn csv_row(list: &str, humidor: &str, favorite: &str, d: &CigarDetails) -> String {
    let fields = [
        list.to_string(),
        csv_text(humidor),
        opt_text(&d.brand),
        csv_text(&d.name),
        opt_text(&d.size),
        opt_num(&d.length_inches),
        opt_num(&d.ring_gauge),
        opt_text(&d.strength),
        opt_text(&d.origin),
        opt_text(&d.wrapper),
        opt_text(&d.binder),
        opt_text(&d.filler),
        opt_num(&d.quantity),
        yes_no(d.in_stock),
        opt_num(&d.price),
        d.purchase_date
            .map(|p| p.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
        opt_text(&d.retail_link),
        favorite.to_string(),
        opt_text(&d.notes),
    ];
    fields.join(",")
}

/// Spreadsheet-oriented CSV: owned cigars (`collection`) then wish-list items (`wish_list`).
pub fn to_csv(export: &UserExport) -> String {
    let mut out = String::from(UTF8_BOM);
    out.push_str(&CSV_HEADER.join(","));
    out.push_str("\r\n");
    for c in &export.cigars {
        out.push_str(&csv_row(
            "collection",
            &c.humidor,
            if c.favorite { "yes" } else { "no" },
            &c.details,
        ));
        out.push_str("\r\n");
    }
    for w in &export.wish_list {
        out.push_str(&csv_row("wish_list", "", "", &w.details));
        out.push_str("\r\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_not_quoted() {
        assert_eq!(csv_text("Padrón 1964"), "Padrón 1964");
    }

    #[test]
    fn rfc4180_quoting() {
        assert_eq!(csv_text("a,b"), "\"a,b\"");
        assert_eq!(csv_text("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_text("line1\nline2"), "\"line1\nline2\"");
        assert_eq!(csv_text("cr\rhere"), "\"cr\rhere\"");
    }

    #[test]
    fn formula_prefixes_are_neutralised_with_a_leading_tab() {
        for evil in [
            "=1+1",
            "+1+1",
            "-1+1",
            "@SUM(A1)",
            "=HYPERLINK(\"http://x\",\"y\")",
            "\t=1",
        ] {
            let cell = csv_text(evil);
            assert!(
                cell.starts_with("\"\t"),
                "{evil:?} must be quoted with a leading tab, got {cell:?}"
            );
        }
        // a formula character that is not the first character is left alone
        assert_eq!(csv_text("1=1"), "1=1");
        assert_eq!(csv_text("a-b"), "a-b");
    }

    fn details(name: &str) -> CigarDetails {
        CigarDetails {
            name: name.to_string(),
            brand: None,
            size: None,
            length_inches: None,
            ring_gauge: None,
            strength: None,
            origin: None,
            wrapper: None,
            binder: None,
            filler: None,
            quantity: Some(2),
            in_stock: Some(true),
            price: Some(12.5),
            purchase_date: None,
            retail_link: None,
            notes: None,
        }
    }

    #[test]
    fn empty_export_is_bom_plus_header_only() {
        let export = UserExport {
            format_version: FORMAT_VERSION,
            exported_at: Utc::now(),
            humidors: vec![],
            cigars: vec![],
            favorites: vec![],
            wish_list: vec![],
        };
        let csv = to_csv(&export);
        assert!(csv.starts_with('\u{feff}'));
        assert!(
            csv.trim_start_matches('\u{feff}')
                .starts_with("list,humidor,brand,name,")
        );
        assert_eq!(csv.matches("\r\n").count(), 1);
    }

    #[test]
    fn rows_have_one_field_per_header_column() {
        let row = csv_row("collection", "Main", "no", &details("Plain"));
        assert_eq!(row.split(',').count(), CSV_HEADER.len());
        assert!(row.starts_with("collection,Main,,Plain,"));
    }
}
