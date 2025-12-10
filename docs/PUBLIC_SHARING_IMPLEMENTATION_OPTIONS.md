# Public Humidor Sharing - Implementation Options

## Current State Analysis

**Existing Private Sharing System** (✅ Fully Implemented):
- User-to-user sharing via `humidor_shares` table
- Three permission levels: `view`, `edit`, `full`
- Requires both users to have accounts
- Authentication required for all access
- Row-level security enforced

**New Requirement**:
Add public sharing capability - generate a shareable link that allows **anyone** (authenticated or not) to view a humidor and its cigars in read-only mode without requiring a user account.

---

## Option 1: Minimal Implementation (Mealie-Inspired Pattern)

### Overview
Simple token-based public sharing using UUID as the access key. Mirrors Mealie's proven recipe sharing pattern with minimal database changes.

### Architecture

#### Database Schema
```sql
-- V15 Migration: Create public share tokens table
CREATE TABLE IF NOT EXISTS public_humidor_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    humidor_id UUID NOT NULL REFERENCES humidors(id) ON DELETE CASCADE,
    created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent multiple tokens per humidor (one active public share at a time)
    CONSTRAINT unique_public_humidor_share UNIQUE (humidor_id)
);

CREATE INDEX idx_public_shares_humidor ON public_humidor_shares(humidor_id);
CREATE INDEX idx_public_shares_expiry ON public_humidor_shares(expires_at);

COMMENT ON TABLE public_humidor_shares IS 'Public share tokens for anonymous read-only humidor access';
COMMENT ON COLUMN public_humidor_shares.id IS 'UUID serves as both primary key and public access token';
COMMENT ON COLUMN public_humidor_shares.expires_at IS 'Automatic expiration date, defaults to 30 days from creation';
```

**Key Design Decisions**:
- UUID token is sufficient security (128-bit = ~3.4×10³⁸ possibilities)
- No separate token field needed - use `id` as the token
- UNIQUE constraint ensures only one active public share per humidor
- CASCADE delete removes token when humidor or user is deleted
- Expiration checked at access time (no background jobs needed)

#### Backend Routes

**Public Route (No Authentication Required)**:
```rust
// GET /api/v1/shared/humidors/:token
// Returns humidor + cigars for valid, non-expired token
pub async fn get_public_humidor(
    token_id: Uuid,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError("Failed to connect".to_string()))
    })?;

    // Verify token exists and not expired (delete if expired)
    let token_row = client
        .query_opt(
            "SELECT humidor_id FROM public_humidor_shares 
             WHERE id = $1 AND expires_at > NOW()",
            &[&token_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check share token: {}", e);
            reject::custom(AppError::DatabaseError("Failed to verify token".to_string()))
        })?;

    let humidor_id: Uuid = match token_row {
        Some(row) => row.get(0),
        None => {
            // Cleanup expired token if it exists
            let _ = client.execute(
                "DELETE FROM public_humidor_shares WHERE id = $1 AND expires_at <= NOW()",
                &[&token_id]
            ).await;
            
            return Err(reject::custom(AppError::NotFound(
                "Share link not found or expired".to_string()
            )));
        }
    };

    // Fetch humidor details WITHOUT user_id filter (bypass ownership check)
    let humidor_row = client
        .query_opt(
            "SELECT h.id, h.name, h.description, h.image_url, h.created_at,
                    u.username, u.email, u.full_name
             FROM humidors h
             INNER JOIN users u ON h.user_id = u.id
             WHERE h.id = $1",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch humidor: {}", e);
            reject::custom(AppError::DatabaseError("Failed to fetch humidor".to_string()))
        })?;

    let humidor_row = match humidor_row {
        Some(row) => row,
        None => return Err(reject::custom(AppError::NotFound(
            "Humidor not found".to_string()
        ))),
    };

    // Fetch cigars in humidor WITHOUT user_id filter
    let cigar_rows = client
        .query(
            "SELECT c.id, c.name, c.brand, c.origin, c.wrapper, c.strength,
                    c.ring_gauge, c.length_inches, c.quantity, c.notes,
                    c.retail_link, c.created_at, c.updated_at
             FROM cigars c
             WHERE c.humidor_id = $1
             ORDER BY c.name",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch cigars: {}", e);
            reject::custom(AppError::DatabaseError("Failed to fetch cigars".to_string()))
        })?;

    // Build response
    let humidor = PublicHumidorResponse {
        id: humidor_row.get(0),
        name: humidor_row.get(1),
        description: humidor_row.get(2),
        image_url: humidor_row.get(3),
        created_at: humidor_row.get(4),
        owner: PublicUserInfo {
            username: humidor_row.get(5),
            full_name: humidor_row.get(7),
        },
        cigars: cigar_rows.iter().map(|row| /* map to CigarResponse */).collect(),
        cigar_count: cigar_rows.len(),
    };

    Ok(reply::json(&humidor))
}
```

**Authenticated Token Management Routes**:
```rust
// POST /api/v1/humidors/:id/public-share
// Create or update public share token (owner only)
pub async fn create_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    request: CreatePublicShareRequest, // { expires_at?: DateTime }
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    // Verify owner
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("Not humidor owner".to_string())));
    }

    let expires_at = request.expires_at.unwrap_or_else(|| {
        Utc::now() + chrono::Duration::days(30)
    });

    let client = pool.get().await?;
    let token_id = Uuid::new_v4();

    // Upsert token (replace existing if present)
    client.execute(
        "INSERT INTO public_humidor_shares (id, humidor_id, created_by_user_id, expires_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (humidor_id) DO UPDATE 
         SET id = $1, expires_at = $4, created_at = NOW()",
        &[&token_id, &humidor_id, &auth.user_id, &expires_at],
    ).await?;

    Ok(reply::json(&PublicShareResponse {
        token_id,
        share_url: format!("https://{}/shared/humidors/{}", env::var("DOMAIN").unwrap_or_default(), token_id),
        expires_at,
    }))
}

// GET /api/v1/humidors/:id/public-share
// Get current public share token if exists (owner only)
pub async fn get_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    // Verify owner or can manage
    if !can_manage_humidor(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("No permission".to_string())));
    }

    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, expires_at, created_at 
         FROM public_humidor_shares 
         WHERE humidor_id = $1 AND expires_at > NOW()",
        &[&humidor_id],
    ).await?;

    match row {
        Some(row) => Ok(reply::json(&PublicShareResponse {
            token_id: row.get(0),
            share_url: format!("https://{}/shared/humidors/{}", env::var("DOMAIN").unwrap_or_default(), row.get(0)),
            expires_at: row.get(1),
        })),
        None => Err(reject::custom(AppError::NotFound("No public share active".to_string()))),
    }
}

// DELETE /api/v1/humidors/:id/public-share
// Revoke public share (owner only)
pub async fn revoke_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("Not owner".to_string())));
    }

    let client = pool.get().await?;
    let rows = client.execute(
        "DELETE FROM public_humidor_shares WHERE humidor_id = $1",
        &[&humidor_id],
    ).await?;

    if rows == 0 {
        return Err(reject::custom(AppError::NotFound("No active public share".to_string())));
    }

    Ok(reply::json(&serde_json::json!({ "message": "Public share revoked" })))
}
```

#### Frontend Implementation

**Add to Share Modal** (`static/index.html`):
```html
<div class="share-section">
    <h3 class="share-section-title">
        <span class="mdi mdi-link"></span>
        Public Share Link
    </h3>
    
    <div id="publicShareContainer">
        <!-- When no active share -->
        <div id="noPublicShare" class="public-share-empty">
            <p class="text-muted">
                <span class="mdi mdi-information-outline"></span>
                No public share link active. Create one to allow anyone to view this humidor without logging in.
            </p>
            
            <div class="form-group">
                <label for="publicShareExpiry">Expiration Date:</label>
                <input type="datetime-local" id="publicShareExpiry" />
            </div>
            
            <button class="btn btn-primary" id="createPublicShareBtn">
                <span class="mdi mdi-link-plus"></span>
                Create Public Link
            </button>
        </div>
        
        <!-- When share exists -->
        <div id="activePublicShare" class="public-share-active" style="display: none;">
            <div class="share-link-box">
                <input type="text" id="publicShareUrl" readonly class="share-url-input" />
                <button class="btn-icon" id="copyPublicShareBtn" title="Copy Link">
                    <span class="mdi mdi-content-copy"></span>
                </button>
            </div>
            
            <div class="share-info">
                <span class="mdi mdi-clock-outline"></span>
                Expires: <strong id="publicShareExpiry"></strong>
            </div>
            
            <button class="btn btn-danger" id="revokePublicShareBtn">
                <span class="mdi mdi-link-off"></span>
                Revoke Public Link
            </button>
        </div>
    </div>
</div>
```

**JavaScript Functions** (`static/app.js`):
```javascript
async function loadPublicShare(humidorId) {
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-share`,
            { method: 'GET' }
        );
        
        if (response.ok) {
            const data = await response.json();
            document.getElementById('noPublicShare').style.display = 'none';
            document.getElementById('activePublicShare').style.display = 'block';
            document.getElementById('publicShareUrl').value = data.share_url;
            document.getElementById('publicShareExpiry').textContent = 
                new Date(data.expires_at).toLocaleString();
        } else if (response.status === 404) {
            // No active share
            document.getElementById('noPublicShare').style.display = 'block';
            document.getElementById('activePublicShare').style.display = 'none';
            
            // Set default expiry to 30 days from now
            const defaultExpiry = new Date();
            defaultExpiry.setDate(defaultExpiry.getDate() + 30);
            document.getElementById('publicShareExpiry').value = 
                defaultExpiry.toISOString().slice(0, 16);
        }
    } catch (error) {
        console.error('Failed to load public share:', error);
    }
}

async function createPublicShare() {
    const humidorId = currentShareHumidorId;
    const expiryInput = document.getElementById('publicShareExpiry').value;
    const expiresAt = expiryInput ? new Date(expiryInput).toISOString() : null;
    
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-share`,
            {
                method: 'POST',
                body: JSON.stringify({ expires_at: expiresAt })
            }
        );
        
        if (response.ok) {
            showToast('Public share link created!', 'success');
            await loadPublicShare(humidorId);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to create share link', 'error');
        }
    } catch (error) {
        console.error('Failed to create public share:', error);
        showToast('Failed to create share link', 'error');
    }
}

async function copyPublicShareLink() {
    const urlInput = document.getElementById('publicShareUrl');
    
    try {
        // Try Web Share API first (mobile-friendly)
        if (navigator.share) {
            await navigator.share({
                title: 'Humidor Share Link',
                url: urlInput.value
            });
            showToast('Share link copied!', 'success');
        } else {
            // Fallback to clipboard
            await navigator.clipboard.writeText(urlInput.value);
            showToast('Link copied to clipboard!', 'success');
        }
    } catch (error) {
        // Manual fallback
        urlInput.select();
        document.execCommand('copy');
        showToast('Link copied to clipboard!', 'success');
    }
}

async function revokePublicShare() {
    if (!confirm('Are you sure you want to revoke the public share link? This link will stop working immediately.')) {
        return;
    }
    
    const humidorId = currentShareHumidorId;
    
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-share`,
            { method: 'DELETE' }
        );
        
        if (response.ok) {
            showToast('Public share link revoked', 'success');
            await loadPublicShare(humidorId);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to revoke link', 'error');
        }
    } catch (error) {
        console.error('Failed to revoke public share:', error);
        showToast('Failed to revoke link', 'error');
    }
}

// Event listeners
document.getElementById('createPublicShareBtn')?.addEventListener('click', createPublicShare);
document.getElementById('copyPublicShareBtn')?.addEventListener('click', copyPublicShareLink);
document.getElementById('revokePublicShareBtn')?.addEventListener('click', revokePublicShare);

// Load public share status when modal opens
const originalOpenShareModal = openShareHumidorModal;
openShareHumidorModal = async function(humidorId, humidorName) {
    await originalOpenShareModal(humidorId, humidorName);
    await loadPublicShare(humidorId);
};
```

**Public View Page** (`static/shared-humidor.html` - NEW FILE):
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Shared Humidor - Humidor</title>
    <link rel="icon" type="image/png" href="/static/logo.png">
    <link rel="stylesheet" href="/static/styles.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@7.4.47/css/materialdesignicons.min.css">
</head>
<body class="public-view">
    <div class="public-header">
        <div class="logo">
            <img src="/static/logo.png" alt="Humidor" />
            <h1>Humidor</h1>
        </div>
        <a href="/login.html" class="btn-secondary">
            <span class="mdi mdi-login"></span>
            Login
        </a>
    </div>
    
    <main class="public-content">
        <div id="loadingState" class="loading-state">
            <span class="mdi mdi-loading mdi-spin"></span>
            Loading humidor...
        </div>
        
        <div id="errorState" class="error-state" style="display: none;">
            <span class="mdi mdi-alert-circle"></span>
            <h2>Share Link Not Found</h2>
            <p>This link may have expired or been revoked.</p>
            <a href="/login.html" class="btn-primary">Go to Login</a>
        </div>
        
        <div id="humidorContent" style="display: none;">
            <div class="humidor-header">
                <h1 id="humidorName"></h1>
                <p id="humidorDescription" class="humidor-description"></p>
                <div class="humidor-meta">
                    <span class="mdi mdi-account"></span>
                    Owned by <strong id="humidorOwner"></strong>
                </div>
                <div class="humidor-meta">
                    <span class="mdi mdi-cigar"></span>
                    <strong id="cigarCount"></strong> cigars
                </div>
            </div>
            
            <div class="cigars-grid" id="cigarsGrid">
                <!-- Cigars populated via JavaScript -->
            </div>
        </div>
    </main>
    
    <script>
        // Extract token from URL path
        const pathParts = window.location.pathname.split('/');
        const token = pathParts[pathParts.length - 1];
        
        async function loadSharedHumidor() {
            try {
                const response = await fetch(`/api/v1/shared/humidors/${token}`);
                
                if (!response.ok) {
                    showError();
                    return;
                }
                
                const data = await response.json();
                displayHumidor(data);
            } catch (error) {
                console.error('Failed to load humidor:', error);
                showError();
            }
        }
        
        function displayHumidor(data) {
            document.getElementById('loadingState').style.display = 'none';
            document.getElementById('humidorContent').style.display = 'block';
            
            document.getElementById('humidorName').textContent = data.name;
            document.getElementById('humidorDescription').textContent = data.description || '';
            document.getElementById('humidorOwner').textContent = data.owner.full_name || data.owner.username;
            document.getElementById('cigarCount').textContent = data.cigar_count;
            
            const cigarsGrid = document.getElementById('cigarsGrid');
            cigarsGrid.innerHTML = data.cigars.map(cigar => `
                <div class="cigar-card">
                    <h3>${cigar.name}</h3>
                    <div class="cigar-details">
                        <div><strong>Brand:</strong> ${cigar.brand || 'N/A'}</div>
                        <div><strong>Origin:</strong> ${cigar.origin || 'N/A'}</div>
                        <div><strong>Strength:</strong> ${cigar.strength || 'N/A'}</div>
                        <div><strong>Quantity:</strong> ${cigar.quantity}</div>
                    </div>
                    ${cigar.notes ? `<p class="cigar-notes">${cigar.notes}</p>` : ''}
                </div>
            `).join('');
        }
        
        function showError() {
            document.getElementById('loadingState').style.display = 'none';
            document.getElementById('errorState').style.display = 'block';
        }
        
        loadSharedHumidor();
    </script>
</body>
</html>
```

### Pros of Option 1
✅ **Simple & Proven**: Based on Mealie's production-tested pattern
✅ **Minimal Database Changes**: Single new table, no changes to existing schema
✅ **No Breaking Changes**: Existing private sharing unaffected
✅ **Secure by Default**: UUID provides sufficient entropy (128-bit)
✅ **Self-Cleaning**: Expired tokens auto-deleted on access
✅ **Easy to Implement**: ~300 lines of backend + ~200 lines frontend
✅ **User-Friendly**: Direct link sharing, Web Share API support
✅ **No Background Jobs**: Expiration checked at access time

### Cons of Option 1
❌ **One Token Per Humidor**: Can't have multiple public links with different expirations
❌ **Token in URL**: Visible in browser history, server logs
❌ **No Access Analytics**: Can't track who viewed or when
❌ **No Password Protection**: Anyone with link has access (until expiration)
❌ **Cascade Delete Risk**: User deletion revokes all their public shares

---

## Option 2: Enhanced Implementation (Feature-Rich)

### Overview
More sophisticated approach with multiple concurrent tokens, access logging, optional password protection, and analytics capabilities.

### Architecture

#### Extended Database Schema
```sql
-- V15 Migration: Enhanced public sharing
CREATE TABLE IF NOT EXISTS public_humidor_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    humidor_id UUID NOT NULL REFERENCES humidors(id) ON DELETE CASCADE,
    created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL, -- Preserve token after user deletion
    token_name VARCHAR(100), -- User-friendly label like "Shared with forum"
    password_hash VARCHAR(255), -- Optional bcrypt hash for password protection
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    max_views INTEGER, -- Optional view limit (NULL = unlimited)
    view_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_public_shares_humidor ON public_humidor_shares(humidor_id);
CREATE INDEX idx_public_shares_active ON public_humidor_shares(is_active, expires_at);

-- Access log for analytics
CREATE TABLE IF NOT EXISTS public_share_access_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    share_token_id UUID NOT NULL REFERENCES public_humidor_shares(id) ON DELETE CASCADE,
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT,
    country_code VARCHAR(2) -- From IP geolocation if available
);

CREATE INDEX idx_access_log_token ON public_share_access_log(share_token_id);
CREATE INDEX idx_access_log_time ON public_share_access_log(accessed_at);
```

#### Backend Models
```rust
// src/models/public_share.rs
#[derive(Debug, Serialize)]
pub struct PublicShareToken {
    pub id: Uuid,
    pub humidor_id: Uuid,
    pub token_name: Option<String>,
    pub has_password: bool, // Don't expose actual hash
    pub expires_at: DateTime<Utc>,
    pub max_views: Option<i32>,
    pub view_count: i32,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePublicShareRequest {
    pub token_name: Option<String>,
    pub password: Option<String>, // Will be hashed
    pub expires_at: Option<DateTime<Utc>>,
    pub max_views: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AccessPublicShareRequest {
    pub password: Option<String>, // If token is password-protected
}

#[derive(Debug, Serialize)]
pub struct PublicShareAccessStats {
    pub total_views: i32,
    pub unique_ips: i32,
    pub recent_access: Vec<AccessLogEntry>,
}

#[derive(Debug, Serialize)]
pub struct AccessLogEntry {
    pub accessed_at: DateTime<Utc>,
    pub country_code: Option<String>,
}
```

#### Enhanced Backend Routes

**Public Access with Password Support**:
```rust
// POST /api/v1/shared/humidors/:token (password sent in body if needed)
pub async fn access_public_humidor(
    token_id: Uuid,
    request: Option<AccessPublicShareRequest>,
    pool: Pool,
    remote_addr: Option<IpAddr>, // From warp::addr::remote()
    user_agent: Option<String>, // From warp::header("user-agent")
) -> Result<impl Reply, Rejection> {
    let client = pool.get().await?;

    // Fetch token with all constraints
    let token_row = client.query_opt(
        "SELECT humidor_id, password_hash, max_views, view_count, is_active
         FROM public_humidor_shares
         WHERE id = $1 AND expires_at > NOW()",
        &[&token_id],
    ).await?;

    let token_row = match token_row {
        Some(row) => row,
        None => {
            // Cleanup expired/inactive
            let _ = client.execute(
                "DELETE FROM public_humidor_shares 
                 WHERE id = $1 AND (expires_at <= NOW() OR is_active = false)",
                &[&token_id],
            ).await;
            return Err(reject::custom(AppError::NotFound("Link expired or revoked".to_string())));
        }
    };

    let humidor_id: Uuid = token_row.get(0);
    let password_hash: Option<String> = token_row.get(1);
    let max_views: Option<i32> = token_row.get(2);
    let view_count: i32 = token_row.get(3);
    let is_active: bool = token_row.get(4);

    // Check if token is active
    if !is_active {
        return Err(reject::custom(AppError::Forbidden("Link has been revoked".to_string())));
    }

    // Check password if protected
    if let Some(hash) = password_hash {
        let provided_password = request
            .and_then(|r| r.password)
            .ok_or_else(|| reject::custom(AppError::Unauthorized("Password required".to_string())))?;
        
        let valid = bcrypt::verify(&provided_password, &hash)
            .map_err(|_| reject::custom(AppError::InternalError("Password verification failed".to_string())))?;
        
        if !valid {
            return Err(reject::custom(AppError::Unauthorized("Invalid password".to_string())));
        }
    }

    // Check view limit
    if let Some(max) = max_views {
        if view_count >= max {
            return Err(reject::custom(AppError::Forbidden("View limit reached".to_string())));
        }
    }

    // Log access
    let _ = client.execute(
        "INSERT INTO public_share_access_log (share_token_id, ip_address, user_agent)
         VALUES ($1, $2, $3)",
        &[&token_id, &remote_addr, &user_agent],
    ).await;

    // Increment view count
    let _ = client.execute(
        "UPDATE public_humidor_shares 
         SET view_count = view_count + 1, last_accessed_at = NOW()
         WHERE id = $1",
        &[&token_id],
    ).await;

    // Fetch humidor data (same as Option 1)
    // ... (omitted for brevity)

    Ok(reply::json(&humidor_data))
}
```

**Token Management with Analytics**:
```rust
// GET /api/v1/humidors/:id/public-shares (list all tokens)
pub async fn list_public_shares(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    if !can_manage_humidor(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("No permission".to_string())));
    }

    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, token_name, password_hash IS NOT NULL as has_password,
                expires_at, max_views, view_count, last_accessed_at, created_at, is_active
         FROM public_humidor_shares
         WHERE humidor_id = $1
         ORDER BY created_at DESC",
        &[&humidor_id],
    ).await?;

    let tokens: Vec<PublicShareToken> = rows.iter().map(|row| {
        PublicShareToken {
            id: row.get(0),
            humidor_id,
            token_name: row.get(1),
            has_password: row.get(2),
            expires_at: row.get(3),
            max_views: row.get(4),
            view_count: row.get(5),
            last_accessed_at: row.get(6),
            created_at: row.get(7),
            is_active: row.get(8),
        }
    }).collect();

    Ok(reply::json(&tokens))
}

// GET /api/v1/humidors/:id/public-shares/:token_id/stats
pub async fn get_share_stats(
    humidor_id: Uuid,
    token_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    if !can_manage_humidor(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("No permission".to_string())));
    }

    let client = pool.get().await?;
    
    let stats_row = client.query_one(
        "SELECT 
            COUNT(*) as total_views,
            COUNT(DISTINCT ip_address) as unique_ips
         FROM public_share_access_log
         WHERE share_token_id = $1",
        &[&token_id],
    ).await?;

    let recent_access = client.query(
        "SELECT accessed_at, country_code
         FROM public_share_access_log
         WHERE share_token_id = $1
         ORDER BY accessed_at DESC
         LIMIT 50",
        &[&token_id],
    ).await?;

    Ok(reply::json(&PublicShareAccessStats {
        total_views: stats_row.get::<_, i64>(0) as i32,
        unique_ips: stats_row.get::<_, i64>(1) as i32,
        recent_access: recent_access.iter().map(|row| AccessLogEntry {
            accessed_at: row.get(0),
            country_code: row.get(1),
        }).collect(),
    }))
}

// PATCH /api/v1/humidors/:id/public-shares/:token_id/toggle
pub async fn toggle_share_active(
    humidor_id: Uuid,
    token_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id).await? {
        return Err(reject::custom(AppError::Forbidden("Not owner".to_string())));
    }

    let client = pool.get().await?;
    client.execute(
        "UPDATE public_humidor_shares
         SET is_active = NOT is_active
         WHERE id = $1 AND humidor_id = $2",
        &[&token_id, &humidor_id],
    ).await?;

    Ok(reply::json(&serde_json::json!({ "message": "Token status toggled" })))
}
```

#### Enhanced Frontend

**Advanced Share Modal**:
```html
<div class="share-section">
    <h3 class="share-section-title">
        <span class="mdi mdi-link-variant"></span>
        Public Share Links
        <button class="btn-icon" id="refreshPublicSharesBtn" title="Refresh">
            <span class="mdi mdi-refresh"></span>
        </button>
    </h3>
    
    <!-- Create New Token Form -->
    <div class="create-token-form">
        <div class="form-row">
            <div class="form-group">
                <label for="tokenName">Link Name (optional):</label>
                <input type="text" id="tokenName" placeholder="e.g., Forum Share" />
            </div>
            
            <div class="form-group">
                <label for="tokenExpiry">Expires:</label>
                <input type="datetime-local" id="tokenExpiry" />
            </div>
        </div>
        
        <div class="form-row">
            <div class="form-group">
                <label for="tokenPassword">Password (optional):</label>
                <input type="password" id="tokenPassword" placeholder="Leave blank for no password" />
            </div>
            
            <div class="form-group">
                <label for="tokenMaxViews">Max Views (optional):</label>
                <input type="number" id="tokenMaxViews" min="1" placeholder="Unlimited" />
            </div>
        </div>
        
        <button class="btn btn-primary" id="createPublicTokenBtn">
            <span class="mdi mdi-link-plus"></span>
            Create New Link
        </button>
    </div>
    
    <!-- Active Tokens List -->
    <div id="publicTokensList" class="tokens-list">
        <!-- Populated via JavaScript -->
    </div>
</div>
```

**Token Card Template**:
```javascript
function renderTokenCard(token) {
    const expiresIn = Math.ceil((new Date(token.expires_at) - new Date()) / (1000 * 60 * 60 * 24));
    const isExpiringSoon = expiresIn <= 7;
    
    return `
        <div class="token-card ${!token.is_active ? 'token-inactive' : ''}">
            <div class="token-header">
                <div class="token-name">
                    <span class="mdi ${token.has_password ? 'mdi-lock' : 'mdi-link'}"></span>
                    ${token.token_name || 'Unnamed Link'}
                </div>
                <div class="token-status">
                    ${token.is_active ? 
                        `<span class="badge badge-success">Active</span>` : 
                        `<span class="badge badge-secondary">Paused</span>`
                    }
                </div>
            </div>
            
            <div class="token-stats">
                <div class="stat">
                    <span class="mdi mdi-eye"></span>
                    ${token.view_count} views${token.max_views ? ` / ${token.max_views}` : ''}
                </div>
                <div class="stat ${isExpiringSoon ? 'stat-warning' : ''}">
                    <span class="mdi mdi-clock-outline"></span>
                    Expires in ${expiresIn} days
                </div>
            </div>
            
            <div class="token-url">
                <input type="text" readonly value="${generateShareUrl(token.id)}" class="share-url-input" />
                <button class="btn-icon" onclick="copyTokenUrl('${token.id}')" title="Copy">
                    <span class="mdi mdi-content-copy"></span>
                </button>
            </div>
            
            <div class="token-actions">
                <button class="btn-icon" onclick="viewTokenStats('${token.id}')" title="View Statistics">
                    <span class="mdi mdi-chart-line"></span>
                </button>
                <button class="btn-icon" onclick="toggleTokenActive('${token.id}')" 
                        title="${token.is_active ? 'Pause' : 'Activate'}">
                    <span class="mdi ${token.is_active ? 'mdi-pause' : 'mdi-play'}"></span>
                </button>
                <button class="btn-icon btn-danger" onclick="deleteToken('${token.id}')" title="Delete">
                    <span class="mdi mdi-delete"></span>
                </button>
            </div>
        </div>
    `;
}
```

**Statistics Modal**:
```html
<div class="modal" id="tokenStatsModal">
    <div class="modal-content">
        <div class="modal-header">
            <h2>
                <span class="mdi mdi-chart-line"></span>
                Share Link Statistics
            </h2>
            <button class="close-btn" onclick="closeStatsModal()">&times;</button>
        </div>
        <div class="modal-body">
            <div class="stats-summary">
                <div class="stat-box">
                    <div class="stat-value" id="statTotalViews">0</div>
                    <div class="stat-label">Total Views</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="statUniqueIPs">0</div>
                    <div class="stat-label">Unique Visitors</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="statAvgViewsPerDay">0</div>
                    <div class="stat-label">Avg. Views/Day</div>
                </div>
            </div>
            
            <h3>Recent Access</h3>
            <div class="access-log" id="accessLogList">
                <!-- Populated via JavaScript -->
            </div>
        </div>
    </div>
</div>
```

**Public View with Password Prompt**:
```javascript
// In shared-humidor.html
async function loadSharedHumidor() {
    try {
        // First attempt without password
        let response = await fetch(`/api/v1/shared/humidors/${token}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
        });
        
        if (response.status === 401) {
            // Password required
            const password = await showPasswordPrompt();
            if (!password) return showError('Password required');
            
            response = await fetch(`/api/v1/shared/humidors/${token}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ password }),
            });
            
            if (response.status === 401) {
                return showError('Invalid password');
            }
        }
        
        if (!response.ok) {
            return showError();
        }
        
        const data = await response.json();
        displayHumidor(data);
    } catch (error) {
        console.error('Failed to load humidor:', error);
        showError();
    }
}

function showPasswordPrompt() {
    return new Promise((resolve) => {
        const password = prompt('This link is password-protected. Please enter the password:');
        resolve(password);
    });
}
```

### Pros of Option 2
✅ **Multiple Tokens**: Create different links for different audiences
✅ **Password Protection**: Secure links with optional passwords
✅ **Access Analytics**: Track views, unique visitors, geographic data
✅ **View Limits**: Auto-disable after N views
✅ **Pause/Resume**: Temporarily disable without deleting
✅ **Named Tokens**: User-friendly labels for organization
✅ **Expiration Warnings**: Frontend alerts for expiring links
✅ **Detailed Logging**: IP addresses, user agents, timestamps
✅ **Flexible Management**: Toggle, delete, or monitor each token independently

### Cons of Option 2
❌ **Complexity**: ~800 lines backend + ~500 lines frontend
❌ **Database Overhead**: Two additional tables, more indexes
❌ **Privacy Concerns**: IP logging may require GDPR compliance
❌ **Performance Impact**: Access logging on every view
❌ **Maintenance Burden**: More code to test and maintain
❌ **Feature Creep**: Most users may not need advanced analytics

---

## Recommendation: **Option 1 (Minimal Implementation)**

### Rationale

For the initial implementation, **Option 1** is strongly recommended for the following reasons:

1. **Proven Pattern**: Mealie's implementation has been battle-tested in production with thousands of users. The pattern is solid and well-understood.

2. **Fast Development**: Can be implemented in 1-2 days vs. 5-7 days for Option 2. Get the feature shipped quickly and gather user feedback.

3. **Sufficient Security**: 128-bit UUID provides ~3.4×10³⁸ possible values, making brute-force attacks infeasible. For a cigar inventory app, this level of security is appropriate - we're not protecting nuclear launch codes.

4. **User Simplicity**: Most users just want "share this humidor with my friend via link." The one-click copy-paste workflow in Option 1 is ideal.

5. **Maintainability**: Less code = fewer bugs. The codebase stays clean and easy to understand.

6. **Scalability**: The simple token table scales efficiently. Even with millions of shares, lookups are O(1) with UUID primary key.

7. **Iterative Enhancement**: If users request password protection or analytics later, you can add them incrementally without rewriting the foundation. The database schema can be extended with ALTER TABLE.

8. **No Over-Engineering**: Remember YAGNI (You Aren't Gonna Need It). Build what's needed now, not what might be needed someday.

### Migration Path to Option 2 (if needed later)

If analytics or password protection become requested features, the upgrade path is straightforward:

```sql
-- Add optional fields to existing table
ALTER TABLE public_humidor_shares 
    ADD COLUMN token_name VARCHAR(100),
    ADD COLUMN password_hash VARCHAR(255),
    ADD COLUMN max_views INTEGER,
    ADD COLUMN view_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN last_accessed_at TIMESTAMPTZ,
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;

-- Remove UNIQUE constraint to allow multiple tokens
ALTER TABLE public_humidor_shares DROP CONSTRAINT unique_public_humidor_share;

-- Create access log table
CREATE TABLE public_share_access_log (...);
```

This allows incremental feature addition without breaking existing shares.

### Implementation Timeline (Option 1)

**Day 1 - Backend (4-5 hours)**:
- [ ] Create migration V15
- [ ] Add models to `src/models/public_share.rs`
- [ ] Implement handlers in `src/handlers/public_shares.rs`
- [ ] Add routes in `src/routes/` (public + authenticated)
- [ ] Write unit tests
- [ ] Test manually with curl/Postman

**Day 2 - Frontend (4-5 hours)**:
- [ ] Add public share section to existing share modal
- [ ] Implement JavaScript functions (create, copy, revoke)
- [ ] Create `static/shared-humidor.html` public view page
- [ ] Add CSS styles for new components
- [ ] Test end-to-end flow
- [ ] Cross-browser testing

**Day 3 - Polish & Deploy (2-3 hours)**:
- [ ] Integration testing with real database
- [ ] Security audit (SQL injection, XSS checks)
- [ ] Update API documentation
- [ ] Write user guide section
- [ ] Deploy to production
- [ ] Monitor logs for errors

**Total Estimated Time**: 10-13 hours

---

## Security Considerations (Both Options)

1. **Token Entropy**: UUID v4 provides 122 bits of randomness (6 bits reserved). Collision probability: ~2.7×10⁻¹⁸ for 1 billion tokens. Safe.

2. **HTTPS Required**: Tokens in URLs are visible to man-in-the-middle attacks without TLS. Enforce HTTPS in production.

3. **Rate Limiting**: Consider adding rate limiting to public endpoints to prevent abuse:
   ```rust
   // In middleware
   .and(rate_limit(10, Duration::from_secs(60))) // 10 requests/minute per IP
   ```

4. **SQL Injection**: Use parameterized queries exclusively (already done with tokio-postgres `$1, $2` syntax).

5. **XSS Prevention**: Escape all user-provided data in HTML (humidor names, descriptions). Use `textContent` not `innerHTML` in JavaScript.

6. **CSRF Protection**: Not needed for GET requests. POST requests to public endpoints don't have session state to hijack.

7. **Token Revocation**: Option 1 deletes token immediately, but browsers with cached pages may still display data until refresh. Document this behavior.

8. **Expiration Grace Period**: Consider adding a "soft delete" period where expired tokens return "Link expired" message before deletion, allowing users to renew if needed.

---

## Testing Checklist

### Functional Tests
- [ ] Create public share token
- [ ] Access humidor via valid token
- [ ] Access humidor via expired token (should fail)
- [ ] Access humidor via non-existent token (should fail)
- [ ] Revoke token (should immediately stop working)
- [ ] Multiple tokens per humidor (Option 2 only)
- [ ] Password-protected access (Option 2 only)

### Security Tests
- [ ] SQL injection attempts in token parameter
- [ ] XSS attempts in humidor name/description
- [ ] Brute-force token guessing (rate limiting)
- [ ] Access after humidor deletion
- [ ] Access after owner deletion

### Edge Cases
- [ ] Token expires exactly at expiration timestamp
- [ ] Humidor with 0 cigars
- [ ] Humidor with 10,000+ cigars (performance)
- [ ] Concurrent access by multiple users
- [ ] Token creation with past expiration date
- [ ] Token creation with far-future expiration (2100)

### Browser Compatibility
- [ ] Chrome/Edge (Chromium)
- [ ] Firefox
- [ ] Safari (macOS/iOS)
- [ ] Mobile browsers (responsive design)
- [ ] Web Share API fallback

---

## Conclusion

**Recommended Decision**: Implement **Option 1** immediately to ship the feature quickly with proven patterns. Monitor user feedback for 2-3 months. If users request password protection or analytics, incrementally add Option 2 features using database migrations.

This approach balances:
- ✅ **Speed to market**: Feature available in days, not weeks
- ✅ **Code quality**: Simple, maintainable, testable
- ✅ **User value**: Solves the core need (shareable links)
- ✅ **Future flexibility**: Easy to enhance later

The philosophy: **Build the simplest thing that could possibly work, then iterate based on real user needs, not imagined ones.**

---

## Questions for Consideration

Before implementation, please confirm:

1. **Default Expiration**: Is 30 days appropriate, or would you prefer 7/14/90 days?
2. **Domain**: What's the production domain for generating share URLs? (e.g., `humidor.example.com`)
3. **HTTPS**: Is production environment configured for HTTPS? (Required for secure token transmission)
4. **One Token Per Humidor**: Is the UNIQUE constraint acceptable, or do you want multiple concurrent tokens from the start?
5. **Owner-Only Creation**: Should only humidor owners create public shares, or should users with "full" permission level also be allowed?

Please review both options and let me know which you'd like to proceed with!
