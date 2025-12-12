# Ownership Transfer Feature

## Overview
The ownership transfer feature allows administrators to transfer all humidors and cigars from one user to another. This is particularly useful when a user needs to be removed from the system but their data should be preserved.

## Use Cases
1. **User Departure**: When a user leaves but their cigar collection data should be kept
2. **Account Consolidation**: Merging multiple accounts into one
3. **Data Preservation**: Before deleting a user account, transfer their data to another user

## API Endpoint

### POST /api/v1/admin/transfer-ownership
**Authentication**: Admin only (requires JWT with `is_admin: true`)

**Request Body**:
```json
{
  "from_user_id": "uuid-of-source-user",
  "to_user_id": "uuid-of-destination-user"
}
```

**Response** (200 OK):
```json
{
  "humidors_transferred": 5,
  "cigars_transferred": 127
}
```

**Error Responses**:
- `403 Forbidden`: Non-admin user attempted transfer
- `404 Not Found`: One or both users do not exist
- `422 Unprocessable Entity`: Source and destination users are the same
- `500 Internal Server Error`: Database or transaction error

## What Gets Transferred

### Transferred Items
- ✅ All humidors owned by source user
- ✅ All cigars owned by source user
- ✅ Updated timestamps on transferred items

### Cleaned Up Items
- 🗑️ Humidor shares (shares from source user's humidors are deleted)

### Unchanged Items
- ℹ️ Favorites (remain with original users)
- ℹ️ Wish list items (remain with original users)
- ℹ️ User organizers (brands, origins, etc. - stay with creators)
- ℹ️ Password reset tokens (tied to user accounts)

## Frontend Usage

### Admin Panel
1. Navigate to the Admin section (Profile → Admin Panel)
2. Click the "Transfer Ownership" button next to "Create User"
3. Select the **source user** (FROM) - the user losing ownership
4. Select the **destination user** (TO) - the user receiving ownership
5. Review the warning about what will be transferred
6. Click "Transfer Ownership" to confirm

### UI Elements
- **Button**: Located in the User Management section header
- **Icon**: `mdi-swap-horizontal` (swap arrows)
- **Modal**: Two dropdown selects for user selection with warning box
- **Toast Notification**: Shows transfer results (e.g., "Successfully transferred 5 humidor(s) and 127 cigar(s)")

## Technical Implementation

### Backend (Rust)

#### Handler Function
Location: `src/handlers/admin/users.rs`

```rust
pub async fn transfer_ownership(
    request: TransferOwnershipRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection>
```

**Transaction Steps**:
1. Validate source ≠ destination
2. Begin database transaction
3. Verify both users exist
4. Update `humidors.user_id` WHERE `user_id = from_user_id`
5. Update `cigars.user_id` WHERE `user_id = from_user_id`
6. Delete `humidor_shares` for transferred humidors
7. Commit transaction
8. Return transfer counts

#### Route
Location: `src/routes/admin.rs`

```rust
POST /api/v1/admin/transfer-ownership
    .and(with_admin(db_pool.clone()))
    .and_then(admin::transfer_ownership)
```

#### Models
Location: `src/models/user.rs`

```rust
pub struct TransferOwnershipRequest {
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
}

pub struct TransferOwnershipResponse {
    pub humidors_transferred: i64,
    pub cigars_transferred: i64,
}
```

### Frontend (JavaScript)

#### Functions
Location: `static/app.js`

- `showTransferOwnershipModal()`: Loads users and displays modal
- `handleTransferOwnershipSubmit(e)`: Validates and submits transfer request
- Event listeners attached in `initializeUserManagementHandlers()`

#### HTML
Location: `static/index.html`

- Button: `#transferOwnershipBtn`
- Modal: `#transferOwnershipModal`
- Form: `#transferOwnershipForm`
- Selects: `#fromUserId`, `#toUserId`

## Database Schema Impact

### Tables Modified
```sql
-- Humidors ownership change
UPDATE humidors SET user_id = $to_user_id WHERE user_id = $from_user_id;

-- Cigars ownership change
UPDATE cigars SET user_id = $to_user_id WHERE user_id = $from_user_id;

-- Cleanup shares
DELETE FROM humidor_shares 
WHERE humidor_id IN (SELECT id FROM humidors WHERE user_id = $to_user_id);
```

### Foreign Key Constraints
- `cigars.user_id` → `users.id` (CASCADE DELETE preserved)
- `humidors.user_id` → `users.id` (CASCADE DELETE preserved)
- `cigars.humidor_id` → `humidors.id` (CASCADE DELETE preserved)

## Security Considerations

### Authentication
- ✅ Admin-only endpoint (verified via `with_admin` middleware)
- ✅ JWT token required in Authorization header
- ✅ Admin flag checked on every request

### Authorization
- ✅ Only users with `is_admin = true` can transfer ownership
- ✅ Regular users receive 403 Forbidden

### Data Integrity
- ✅ Transaction ensures atomic operation (all-or-nothing)
- ✅ User existence validated before transfer
- ✅ Source ≠ destination validation prevents self-transfer
- ✅ Foreign key constraints maintained throughout

### Audit Trail
- ✅ Transfer logged to console/file logs with:
  - Admin user ID performing transfer
  - Source user ID
  - Destination user ID
  - Number of humidors transferred
  - Number of cigars transferred
  - Timestamp

Example log:
```
INFO humidor::handlers::admin::users: Ownership transferred successfully 
  admin_id=<uuid> from_user_id=<uuid> to_user_id=<uuid> 
  humidors_transferred=5 cigars_transferred=127
```

## Testing

### Test Suite
Location: `tests/ownership_transfer_tests.rs`

Tests include:
1. ✅ `test_transfer_humidor_ownership` - Basic transfer functionality
2. ✅ `test_non_admin_cannot_transfer_ownership` - Authorization check
3. ✅ `test_transfer_with_invalid_users` - Error handling for missing users
4. ✅ `test_transfer_to_same_user` - Validation error for same source/destination
5. ✅ `test_transfer_cleans_up_shares` - Share cleanup verification
6. ✅ `test_safe_user_deletion_after_transfer` - End-to-end workflow

### Running Tests
```bash
# Set test database connection
$env:TEST_DATABASE_URL = "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db"

# Run ownership transfer tests
cargo test ownership_transfer --tests -- --nocapture

# Run all tests
cargo test --tests
```

## Recommended Workflow

### Before Deleting a User
1. **Backup First**: Create system backup
2. **Transfer Ownership**: Use transfer feature to move data to another user
3. **Verify Transfer**: Check that humidors/cigars now belong to destination user
4. **Delete User**: Safe to delete source user (no cascade deletion of data)

### Example Scenario
**Problem**: User "john_doe" is leaving but has 50 cigars in 3 humidors worth preserving.

**Solution**:
1. Admin logs in and navigates to User Management
2. Click "Transfer Ownership"
3. Select "john_doe" as source user
4. Select "admin" (or another user) as destination user
5. Click "Transfer Ownership"
6. System transfers 3 humidors and 50 cigars
7. Now safe to delete "john_doe" account
8. Data preserved under "admin" account

## Limitations

### Current Limitations
- ⚠️ **No Undo**: Transfer is permanent and cannot be reversed (except by another manual transfer)
- ⚠️ **All or Nothing**: Cannot selectively transfer specific humidors - transfers ALL
- ⚠️ **Share Reset**: Existing shares are deleted (new owner must reshare)
- ⚠️ **Batch Operations**: Each transfer is one source → one destination (no multi-user transfers)

### Future Enhancements
- 📋 Selective humidor transfer (choose specific humidors to transfer)
- 📋 Transfer preview (show what will be transferred before confirming)
- 📋 Transfer history log (audit table of all transfers)
- 📋 Bulk operations (transfer from multiple users in one operation)
- 📋 Share preservation option (keep existing shares after transfer)

## Troubleshooting

### Issue: Transfer button not visible
**Solution**: Ensure logged-in user has `is_admin = true` flag set

### Issue: Transfer fails with 404 error
**Solution**: Verify both user UUIDs are valid and users exist in database

### Issue: Transfer succeeds but shows 0 items transferred
**Solution**: Source user had no humidors or cigars to transfer (expected behavior)

### Issue: Transaction error during transfer
**Solution**: 
1. Check database connectivity
2. Verify foreign key constraints are intact
3. Check database logs for detailed error
4. Ensure no concurrent modifications to affected records

## Related Documentation
- [Security Model](SECURITY_MODEL.md) - Authentication and authorization patterns
- [Permissions](PERMISSIONS.md) - User permission system
- [Admin Guide](ADMIN_GUIDE.md) - Admin user management
- [API Documentation](API.md) - Complete API reference
