//! Unit tests for type deserialization.
//!
//! These tests run without a live Tandoor instance. They verify that the types
//! correctly handle the API response shapes we've observed in the wild, including
//! the `created_by` field that ships as a full user object rather than an integer.

use mcp_tandoor::client::{PaginatedResponse, Recipe};

// ── Recipe deserialization ────────────────────────────────────────────────────

#[test]
fn recipe_created_by_as_user_object() {
    let json = r#"{
        "id": 1,
        "name": "Test Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": {
            "id": 1,
            "username": "admin",
            "first_name": "Admin",
            "last_name": "",
            "display_name": "Admin",
            "is_staff": true,
            "is_superuser": true,
            "is_active": true
        }
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with user object");
    assert_eq!(recipe.id, 1);
    assert_eq!(recipe.name, "Test Recipe");
    assert!(recipe.created_by.is_some());
}

#[test]
fn recipe_created_by_as_integer() {
    let json = r#"{
        "id": 2,
        "name": "Another Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": 42
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with integer");
    assert_eq!(recipe.id, 2);
    assert!(recipe.created_by.is_some());
}

#[test]
fn recipe_created_by_as_null() {
    let json = r#"{
        "id": 3,
        "name": "Null Creator Recipe",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false,
        "created_by": null
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize with null");
    assert_eq!(recipe.id, 3);
    assert!(recipe.created_by.is_none());
}

#[test]
fn recipe_created_by_absent() {
    let json = r#"{
        "id": 4,
        "name": "No Creator Field",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "internal": false
    }"#;

    let recipe: Recipe = serde_json::from_str(json).expect("should deserialize without field");
    assert_eq!(recipe.id, 4);
    assert!(recipe.created_by.is_none());
}

// ── Keyword deserialization ───────────────────────────────────────────────────

#[test]
fn keyword_with_label_field() {
    // Some Tandoor endpoints return "label" instead of "name"
    let json = r#"{"id": 1, "label": "dinner"}"#;
    let kw: mcp_tandoor::client::Keyword =
        serde_json::from_str(json).expect("should deserialize label as name");
    assert_eq!(kw.name, "dinner");
}

#[test]
fn keyword_with_name_field() {
    let json = r#"{"id": 2, "name": "breakfast"}"#;
    let kw: mcp_tandoor::client::Keyword =
        serde_json::from_str(json).expect("should deserialize name field");
    assert_eq!(kw.name, "breakfast");
}

// ── PaginatedResponse deserialization ────────────────────────────────────────

#[test]
fn paginated_response_with_recipes() {
    let json = r#"{
        "count": 2,
        "next": null,
        "previous": null,
        "results": [
            {
                "id": 1,
                "name": "Pasta",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "internal": false,
                "created_by": {"id": 1, "username": "admin", "first_name": "", "last_name": "", "display_name": "admin", "is_staff": true, "is_superuser": false, "is_active": true}
            },
            {
                "id": 2,
                "name": "Soup",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "internal": false
            }
        ]
    }"#;

    let resp: PaginatedResponse<Recipe> =
        serde_json::from_str(json).expect("should deserialize paginated response");
    assert_eq!(resp.count, 2);
    assert_eq!(resp.results.len(), 2);
    assert_eq!(resp.results[0].name, "Pasta");
    assert_eq!(resp.results[1].name, "Soup");
}

// ── Auth token handling ───────────────────────────────────────────────────────

#[test]
fn auth_token_deserialization() {
    let json = r#"{"token": "tda_00000000_0000_0000_0000_000000000000"}"#;
    let token: mcp_tandoor::client::AuthToken =
        serde_json::from_str(json).expect("should deserialize auth token");
    assert_eq!(token.token, "tda_00000000_0000_0000_0000_000000000000");
}

#[test]
fn tandoor_client_set_token_marks_authenticated() {
    let mut client = mcp_tandoor::TandoorClient::new("http://localhost:8080".to_string());
    assert!(!client.is_authenticated());
    client.set_token("tda_test_token".to_string());
    assert!(client.is_authenticated());
    assert_eq!(client.get_token(), Some("tda_test_token"));
}
