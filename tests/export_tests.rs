//! Per-user export: scoping (no cross-user leakage), CSV validity, formula-injection protection.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_export_routes;
use serde_json::Value;
use serial_test::serial;
use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;

/// Minimal independent RFC 4180 parser, so the CSV is validated by something other than the writer.
fn parse_csv(input: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => in_quotes = false,
                _ => field.push(c),
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' if chars.peek() == Some(&'\n') => {
                    chars.next();
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                _ => field.push(c),
            }
        }
    }
    assert!(!in_quotes, "unterminated quoted field");
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

struct Fixture {
    token: String,
    user_id: Uuid,
}

async fn user(ctx: &TestContext, name: &str) -> Fixture {
    let (user_id, username) = create_test_user(&ctx.pool, name, "Passw0rd!", false)
        .await
        .unwrap();
    Fixture {
        token: create_test_jwt(user_id, &username).unwrap(),
        user_id,
    }
}

async fn exec(ctx: &TestContext, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) {
    ctx.pool
        .get()
        .await
        .unwrap()
        .execute(sql, params)
        .await
        .unwrap();
}

/// Insert a cigar (optionally in a humidor, optionally with a brand) and return its id.
async fn cigar(
    ctx: &TestContext,
    name: &str,
    notes: Option<&str>,
    quantity: i32,
    humidor: Option<Uuid>,
    brand: Option<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    exec(
        ctx,
        "INSERT INTO cigars (id, name, notes, quantity, humidor_id, brand_id, price, is_active, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, 12.5, true, NOW(), NOW())",
        &[&id, &name, &notes, &quantity, &humidor, &brand],
    )
    .await;
    id
}

async fn brand(ctx: &TestContext, user_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    exec(
        ctx,
        "INSERT INTO brands (id, user_id, name, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW())",
        &[&id, &user_id, &name],
    )
    .await;
    id
}

async fn wish(ctx: &TestContext, user_id: Uuid, cigar_id: Uuid, notes: &str) {
    exec(
        ctx,
        "INSERT INTO wish_list (id, user_id, cigar_id, notes) VALUES ($1, $2, $3, $4)",
        &[&Uuid::new_v4(), &user_id, &cigar_id, &notes],
    )
    .await;
}

async fn favorite(
    ctx: &TestContext,
    user_id: Uuid,
    cigar_id: Option<Uuid>,
    snapshot: Option<&str>,
) {
    exec(
        ctx,
        "INSERT INTO favorites (user_id, cigar_id, snapshot_name) VALUES ($1, $2, $3)",
        &[&user_id, &cigar_id, &snapshot],
    )
    .await;
}

async fn get(
    ctx: &TestContext,
    token: Option<&str>,
    query: &str,
) -> warp::http::Response<warp::hyper::body::Bytes> {
    let api = create_export_routes(ctx.pool.clone()).recover(handle_rejection);
    let mut req = warp::test::request().path(&format!("/api/v1/export{query}"));
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    req.reply(&api).await
}

#[tokio::test]
#[serial]
async fn json_export_contains_only_the_requesters_data() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;
    let b = user(&ctx, "bob").await;

    // --- Alice
    let a_h = create_test_humidor(&ctx.pool, a.user_id, "A-Humidor")
        .await
        .unwrap();
    let a_brand = brand(&ctx, a.user_id, "A-Brand").await;
    let a_c = cigar(
        &ctx,
        "A-Cigar",
        Some("A-notes"),
        3,
        Some(a_h),
        Some(a_brand),
    )
    .await;
    let a_w = cigar(&ctx, "A-Wish", None, 1, None, None).await;
    wish(&ctx, a.user_id, a_w, "A-wish-note").await;
    favorite(&ctx, a.user_id, Some(a_c), None).await;

    // --- Bob (every string contains SECRET)
    let b_h = create_test_humidor(&ctx.pool, b.user_id, "B-HUMIDOR-SECRET")
        .await
        .unwrap();
    let b_brand = brand(&ctx, b.user_id, "B-BRAND-SECRET").await;
    let b_c = cigar(
        &ctx,
        "B-CIGAR-SECRET",
        Some("B-NOTES-SECRET"),
        9,
        Some(b_h),
        Some(b_brand),
    )
    .await;
    let b_w = cigar(&ctx, "B-WISH-SECRET", None, 1, None, None).await;
    wish(&ctx, b.user_id, b_w, "B-WISHNOTE-SECRET").await;
    favorite(&ctx, b.user_id, Some(b_c), None).await;
    favorite(&ctx, b.user_id, None, Some("B-FAVSNAP-SECRET")).await;

    let res = get(&ctx, Some(&a.token), "?format=json").await;
    assert_eq!(res.status(), StatusCode::OK);
    let text = String::from_utf8(res.body().to_vec()).unwrap();
    assert!(
        !text.contains("SECRET"),
        "another user's data leaked into the export:\n{text}"
    );

    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["format_version"], 1);
    assert_eq!(json["humidors"].as_array().unwrap().len(), 1);
    assert_eq!(json["humidors"][0]["name"], "A-Humidor");
    let cigars = json["cigars"].as_array().unwrap();
    assert_eq!(cigars.len(), 1);
    assert_eq!(cigars[0]["name"], "A-Cigar");
    assert_eq!(cigars[0]["brand"], "A-Brand");
    assert_eq!(cigars[0]["humidor"], "A-Humidor");
    assert_eq!(cigars[0]["favorite"], true);
    assert_eq!(cigars[0]["quantity"], 3);
    assert_eq!(json["wish_list"].as_array().unwrap().len(), 1);
    assert_eq!(json["wish_list"][0]["name"], "A-Wish");
    assert_eq!(json["wish_list"][0]["list_notes"], "A-wish-note");
    assert_eq!(json["favorites"].as_array().unwrap().len(), 1);
    // no images in the export
    assert!(!text.contains("image_url"));

    // and the reverse direction: Bob sees none of Alice's data
    let res = get(&ctx, Some(&b.token), "?format=json").await;
    let text = String::from_utf8(res.body().to_vec()).unwrap();
    assert!(!text.contains("A-Humidor") && !text.contains("A-Cigar") && !text.contains("A-Wish"));
    let json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["favorites"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[serial]
async fn favorites_of_deleted_cigars_keep_their_snapshot_in_json() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;
    favorite(&ctx, a.user_id, None, Some("Snapshot Cigar")).await;

    let res = get(&ctx, Some(&a.token), "").await; // default format = json
    let json: Value = serde_json::from_slice(res.body()).unwrap();
    let fav = &json["favorites"][0];
    assert_eq!(fav["name"], "Snapshot Cigar");
    assert_eq!(fav["cigar_still_exists"], false);
    assert!(fav["quantity"].is_null());
}

#[tokio::test]
#[serial]
async fn csv_is_valid_rfc4180_and_round_trips_tricky_fields() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;
    let h = create_test_humidor(&ctx.pool, a.user_id, "Main, \"Big\" One")
        .await
        .unwrap();
    let br = brand(&ctx, a.user_id, "Padrón").await;
    let tricky_notes = "line1\nline2, with \"quotes\"";
    let c = cigar(
        &ctx,
        "Padrón 1964, Anniversary",
        Some(tricky_notes),
        3,
        Some(h),
        Some(br),
    )
    .await;
    favorite(&ctx, a.user_id, Some(c), None).await;
    let w = cigar(&ctx, "Wish Cigar", None, 1, None, None).await;
    wish(&ctx, a.user_id, w, "n").await;

    let res = get(&ctx, Some(&a.token), "?format=csv").await;
    assert_eq!(res.status(), StatusCode::OK);
    let h = res.headers();
    assert_eq!(h["content-type"], "text/csv; charset=utf-8");
    assert_eq!(h["cache-control"], "no-store");
    let disposition = h["content-disposition"].to_str().unwrap();
    assert!(
        disposition.starts_with("attachment; filename=\"humidor-collection-")
            && disposition.ends_with(".csv\""),
        "{disposition}"
    );

    let text = String::from_utf8(res.body().to_vec()).unwrap();
    assert!(text.starts_with('\u{feff}'), "UTF-8 BOM expected");
    assert!(
        text.trim_start_matches('\u{feff}')
            .starts_with("list,humidor,brand,name,")
    );
    assert!(text.ends_with("\r\n"), "records must end with CRLF");

    let rows = parse_csv(text.trim_start_matches('\u{feff}'));
    assert_eq!(
        rows.len(),
        3,
        "header + 1 owned cigar + 1 wish-list item: {rows:?}"
    );
    assert!(
        rows.iter().all(|r| r.len() == 19),
        "every row must have 19 fields: {rows:?}"
    );

    let header = &rows[0];
    let col = |name: &str| header.iter().position(|h| h == name).unwrap();
    let owned = &rows[1];
    assert_eq!(owned[col("list")], "collection");
    assert_eq!(owned[col("humidor")], "Main, \"Big\" One");
    assert_eq!(owned[col("brand")], "Padrón");
    assert_eq!(owned[col("name")], "Padrón 1964, Anniversary");
    assert_eq!(owned[col("notes")], tricky_notes);
    assert_eq!(owned[col("quantity")], "3");
    assert_eq!(owned[col("price")], "12.5");
    assert_eq!(owned[col("in_stock")], "yes");
    assert_eq!(owned[col("favorite")], "yes");

    let wished = &rows[2];
    assert_eq!(wished[col("list")], "wish_list");
    assert_eq!(wished[col("name")], "Wish Cigar");
    assert_eq!(wished[col("humidor")], "");
}

#[tokio::test]
#[serial]
async fn csv_neutralises_spreadsheet_formula_injection() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;
    let h = create_test_humidor(&ctx.pool, a.user_id, "Main")
        .await
        .unwrap();
    cigar(
        &ctx,
        "=HYPERLINK(\"http://evil.example\",\"click\")",
        Some("@SUM(A1:A9)"),
        1,
        Some(h),
        None,
    )
    .await;
    cigar(&ctx, "+1+1", Some("-2+3"), 1, Some(h), None).await;

    let res = get(&ctx, Some(&a.token), "?format=csv").await;
    let text = String::from_utf8(res.body().to_vec()).unwrap();
    let rows = parse_csv(text.trim_start_matches('\u{feff}'));
    let name = rows[0].iter().position(|h| h == "name").unwrap();
    let notes = rows[0].iter().position(|h| h == "notes").unwrap();
    let mut seen = 0;
    for row in &rows[1..] {
        for cell in [&row[name], &row[notes]] {
            if cell.contains("HYPERLINK")
                || cell.contains("SUM(")
                || cell.contains("1+1")
                || cell.contains("-2+3")
            {
                seen += 1;
                assert!(
                    cell.starts_with('\t'),
                    "dangerous cell must start with a tab: {cell:?}"
                );
            }
        }
    }
    assert_eq!(
        seen, 4,
        "all four dangerous cells must be present and neutralised"
    );
}

#[tokio::test]
#[serial]
async fn export_requires_auth_and_validates_format() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;

    assert_eq!(
        get(&ctx, None, "?format=json").await.status(),
        StatusCode::UNAUTHORIZED
    );

    let res = get(&ctx, Some(&a.token), "?format=xml").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body: Value = serde_json::from_slice(res.body()).unwrap();
    assert_eq!(body["error"], "BAD_REQUEST");

    let res = get(&ctx, Some(&a.token), "").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-type"], "application/json");
}

#[tokio::test]
#[serial]
async fn empty_account_exports_cleanly() {
    let ctx = setup_test_db().await;
    let a = user(&ctx, "alice").await;

    let res = get(&ctx, Some(&a.token), "?format=json").await;
    let json: Value = serde_json::from_slice(res.body()).unwrap();
    for key in ["humidors", "cigars", "favorites", "wish_list"] {
        assert_eq!(json[key].as_array().unwrap().len(), 0, "{key}");
    }

    let res = get(&ctx, Some(&a.token), "?format=csv").await;
    let text = String::from_utf8(res.body().to_vec()).unwrap();
    assert_eq!(
        parse_csv(text.trim_start_matches('\u{feff}')).len(),
        1,
        "header only"
    );
}
