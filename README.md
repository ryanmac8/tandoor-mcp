# Tandoor Recipes MCP Server

A Model Context Protocol (MCP) server for [Tandoor Recipes](https://tandoor.dev) that provides comprehensive recipe management, shopping lists, meal planning, and inventory tracking capabilities.

## Features

- **Recipe Management**: Search, create, and manage recipes
- **Shopping Lists**: Add items, mark as purchased, and manage shopping workflows
- **Meal Planning**: Plan meals and manage meal schedules
- **Inventory Tracking**: Monitor pantry items and get recipe suggestions
- **Keywords & Tags**: Organize recipes with keywords and categories
- **Cooking Logs**: Track cooking history and ratings

## Transport

This server uses the **Streamable HTTP** MCP transport. The MCP endpoint is `/mcp` (not `/sse`). Streamable HTTP is required for Claude custom connectors — legacy SSE triggers an OAuth discovery probe that Claude cannot satisfy.

## Quick Start

1. **Clone the repository**:

```bash
git clone https://github.com/ryanmac8/tandoor-mcp
cd tandoor-mcp
```

2. **Configure environment variables**:

```bash
# Preferred: use a pre-existing API token (avoids the 10-req/day login rate limit)
export TANDOOR_BASE_URL="http://localhost:8080"
export TANDOOR_AUTH_TOKEN="tda_your_token_here"

# Alternative: username/password (subject to Tandoor rate limiting)
export TANDOOR_BASE_URL="http://localhost:8080"
export TANDOOR_USERNAME="admin"
export TANDOOR_PASSWORD="your-password"
```

3. **Run the server**:

```bash
cargo run
# Listens at 0.0.0.0:3001 by default; MCP endpoint at /mcp
```

## Docker

```bash
docker run -d \
  -e TANDOOR_BASE_URL="http://your-tandoor:8080" \
  -e TANDOOR_AUTH_TOKEN="tda_your_token_here" \
  -e BIND_ADDR="0.0.0.0:3001" \
  -p 3001:3001 \
  ghcr.io/ryanmac8/tandoor-mcp:latest
```

## Adding to Claude as a Custom Connector

1. In Claude settings, go to **Integrations → Custom connectors → Add custom connector**
2. Set the URL to your server's `/mcp` endpoint:
   ```
   https://tandoor-mcp.example.com/mcp
   ```
3. No OAuth Client ID is needed — Streamable HTTP doesn't trigger OAuth probes.

### Securing the connector with a query-key gate (recommended)

If you're exposing the server publicly, add a key gate at your reverse proxy rather than relying on OAuth:

1. Generate a random key: `openssl rand -base64 30`
2. Configure your proxy to return 403 when the key is absent (e.g. Nginx `$arg_key != "..."` → `return 403`)
3. Give Claude the URL with the key appended: `https://tandoor-mcp.example.com/mcp?key=YOUR_KEY`

Claude holds the full URL including the query param and passes it on every request.

## Tandoor Configuration

⚠️ **IMPORTANT**: Tandoor uses a multi-tenant permission system that requires specific setup for API access to work properly. Without proper space and group configuration, you'll get permission errors even with valid authentication.

### Method 1: Web Interface Setup (Recommended)

1. **Access Tandoor admin interface**:

   - Go to `http://your-tandoor-url/admin/`
   - Login with superuser credentials

2. **Create/Verify Groups**:

   - Navigate to **Authentication and Authorization** → **Groups**
   - Ensure these groups exist: `admin`, `user`, `guest`
   - If missing, create them (names must match exactly)

3. **Create a Space**:

   - Navigate to **Cookbook** → **Spaces**
   - Click **Add Space**
   - Fill in:
     - **Name**: Your organization/space name (e.g., "Production", "Family")
     - **Max recipes**: 0 (unlimited)
     - **Max users**: 0 (unlimited)
     - **Max file storage mb**: 0 (unlimited)
     - **Allow sharing**: Checked
   - Click **Save**

4. **Create User Space Association**:
   - Navigate to **Cookbook** → **User spaces**
   - Click **Add User space**
   - Select:
     - **User**: Your admin user
     - **Space**: The space you just created
     - **Active**: Must be checked
   - Click **Save**
   - After saving, click on the created User space entry
   - In **Groups**, select `admin` and add it
   - Click **Save**

### Method 2: Command Line Setup

```bash
docker exec -it your-tandoor-container /opt/recipes/venv/bin/python manage.py shell
```

```python
from cookbook.models import Space, UserSpace
from django.contrib.auth.models import User, Group

admin_user = User.objects.get(username='admin')

space, created = Space.objects.get_or_create(
    name='Production',
    defaults={
        'created_by': admin_user,
        'max_recipes': 0,
        'max_users': 0,
        'max_file_storage_mb': 0,
        'allow_sharing': True
    }
)

user_space, created = UserSpace.objects.get_or_create(
    user=admin_user,
    space=space,
    defaults={'active': True}
)

if not created and not user_space.active:
    user_space.active = True
    user_space.save()

admin_group = Group.objects.get(name='admin')
user_space.groups.add(admin_group)
```

### Verification

```bash
# Get an API token
curl -X POST http://your-tandoor-url/api-token-auth/ \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'

# Test API access
curl -X GET http://your-tandoor-url/api/keyword/ \
  -H "Authorization: Bearer <YOUR_TOKEN>"
```

## Configuration

### Environment Variables

| Variable             | Description                                               | Default                  |
| -------------------- | --------------------------------------------------------- | ------------------------ |
| `TANDOOR_BASE_URL`   | Full URL to your Tandoor instance                         | `http://localhost:8080`  |
| `TANDOOR_AUTH_TOKEN` | Pre-existing Tandoor API token (**preferred**)            | —                        |
| `TANDOOR_USERNAME`   | Username (only used when `TANDOOR_AUTH_TOKEN` is not set) | `admin`                  |
| `TANDOOR_PASSWORD`   | Password (only used when `TANDOOR_AUTH_TOKEN` is not set) | `admin`                  |
| `BIND_ADDR`          | Address and port for the MCP server                       | `0.0.0.0:3001`           |
| `RUST_LOG`           | Log level (`info`, `debug`, `trace`, …)                   | `info`                   |

`TANDOOR_AUTH_TOKEN` is strongly preferred. Tandoor limits the `/api-token-auth/` login endpoint to **10 requests per day per IP**; a token bypasses this entirely.

### Getting a Tandoor API token

```bash
curl -X POST http://your-tandoor/api-token-auth/ \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
# → {"token": "tda_..."}
```

## Available Tools

### Recipe Management

- `search_recipes` — Search for recipes with flexible querying
- `get_recipe_details` — Get comprehensive recipe information with scaled ingredients
- `create_recipe` — Create a new recipe
- `import_recipe_from_url` — Import a recipe from an external URL

### Shopping Lists

- `add_to_shopping_list` — Add items to shopping list with intelligent consolidation
- `get_shopping_list` — Get current shopping list organized by store section
- `check_shopping_items` — Mark shopping list items as checked/purchased
- `clear_shopping_list` — Clear checked items from shopping list and update pantry

### Food & Inventory

- `search_foods` — Search for foods/ingredients with fuzzy name matching
- `update_pantry` — Update pantry inventory status
- `suggest_from_inventory` — Get recipe suggestions based on current inventory

### Meal Planning

- `get_meal_plans` — Get meal plans for a date range
- `create_meal_plan` — Create a new meal plan
- `delete_meal_plan` — Delete a meal plan
- `get_meal_types` — Get available meal types

### Metadata

- `get_keywords` — Get all available recipe keywords/tags
- `get_units` — Get available measurement units

### Cooking History

- `get_cook_log` — Get cooking history
- `log_cooked_recipe` — Log a cooked recipe

## Troubleshooting

### "Couldn't register with Tandoor's sign-in service" (Claude connector)

This happens when the connector URL points to a legacy SSE endpoint (`/sse`). Claude probes for OAuth and fails. Fix: use the Streamable HTTP endpoint (`/mcp`).

### "Authentication credentials were not provided"

The `Authorization: Bearer` header is missing or the token is invalid. Verify `TANDOOR_AUTH_TOKEN` is set and correct.

### "You do not have permission to perform this action"

Tandoor permissions issue. Check that the user has an active `UserSpace` with an `admin` group assignment (see [Tandoor Configuration](#tandoor-configuration) above).

### "Request was throttled" (429)

Tandoor limits authentication to 10 requests per day per IP. Switch to `TANDOOR_AUTH_TOKEN` to bypass it. If already throttled, wait until midnight UTC or restart Tandoor to reset the counter.

### Django Scopes Error

```
ScopeError: A scope on dimension(s) space needs to be active for this query
```

The user has no active space. Follow the Tandoor Configuration section to set one up.

### Connection Refused / Network Errors

- Verify `TANDOOR_BASE_URL` is reachable from where the MCP server runs
- Check if Tandoor is running: `curl http://your-tandoor-url/`

## Development

### Building from Source

```bash
cargo build
RUST_LOG=debug cargo run
```

### Running Tests

```bash
# Unit tests — no network or Docker required
cargo test --test test_types

# Integration tests — requires a live Tandoor instance
TANDOOR_BASE_URL=http://localhost:8080 \
TANDOOR_AUTH_TOKEN=tda_your_token \
cargo test -- --test-threads=1
```

The integration test suite can also be driven through `./scripts/test.sh` which manages a Docker-based Tandoor instance:

```bash
./scripts/test.sh test           # full run: up, test, down
./scripts/test.sh test --keep-running
./scripts/test.sh up / down / logs
```

**Testing requirements**: Docker + Docker Compose installed, ports 8080 and 5432 available.

### CI

GitHub Actions runs on every push and pull request:

| Job               | What it does                                                         |
| ----------------- | -------------------------------------------------------------------- |
| `unit-test`       | fmt check, clippy, `cargo test --test test_types` (no Docker/network) |
| `integration-test`| spins up Postgres + Tandoor, creates a test user, runs all tests     |
| `security`        | `cargo audit` for known vulnerabilities                              |
| `docker-build`    | builds the Docker image (no push) to verify the `Dockerfile`        |

The `publish` workflow builds a multi-platform image (`linux/amd64`, `linux/arm64`) and pushes it to `ghcr.io/ryanmac8/tandoor-mcp` on push to `main` or a version tag.

## Upstream Patches

This fork of [ChristopherJMiller/tandoor-mcp](https://github.com/ChristopherJMiller/tandoor-mcp) includes three bug fixes:

1. **`TANDOOR_AUTH_TOKEN` support** — The upstream README documented this env var but the code never checked for it. The token path is now implemented in `main.rs` and bypasses the rate-limited login endpoint.
2. **`GLOBAL_AUTH` never read** — `ensure_authenticated()` only checked `GLOBAL_CREDENTIALS`; a token set at startup was silently ignored on every per-session tool call. Fixed in `server.rs` to check `GLOBAL_AUTH` first.
3. **`created_by` type mismatch** — Tandoor's API returns a full user object for `created_by`, not an integer. All affected structs in `client/types.rs` now use `serde_json::Value` to handle both shapes.
