# Features To-Do List

This document tracks the implementation of user management and permissions features.

## Implementation Order & Dependencies

```
Phase 1: Permissions System (Foundation)
    ↓
    ├─→ Phase 2: Admin User Management
    │       ↓
    └─→ Phase 3: Data Isolation Audit
            ↓
        Phase 4: Humidor Sharing
```

---

## Phase 1: User Permissions System ✅ COMPLETED
**Priority:** HIGH - Foundational for all other features  
**Estimated Time:** 1-2 days (SIMPLIFIED!)  
**Status:** ✅ Complete (2025-01-10)

### Overview
Implement a simple two-tier permission system using existing `is_admin` and `is_active` flags.

**Roles:**
- **Admin** - Full system access (user management, all CRUD operations)
- **User** - Standard access (own data only, cannot manage other users)

**Design Decision:** After reviewing Mealie's permission system, we're using a simpler approach that fits Humidor's needs. Mealie needs granular group/household permissions; we just need admin vs user distinction.

### Database Tasks
- [x] No migration needed - Already have `is_admin` and `is_active` in users table!

### Code Tasks
- [x] Update `src/middleware/auth.rs`
  - [x] Add `with_admin()` middleware filter
  - [x] Add helper function `is_admin(&self) -> bool` to `AuthContext`
  - [x] Add helper function `get_user(&self) -> Result<&UserResponse>` to `AuthContext`
  - [x] Keep existing `with_current_user()` for general auth
  
- [x] Update `src/errors.rs`
  - [x] Update `Forbidden` variant to accept message string

### Testing Tasks
- [x] Create `tests/permission_tests.rs`
  - [x] Test admin flag in database
  - [x] Test regular user lacks admin flag
  - [x] Test toggling admin status
  - [x] Test multiple admins allowed
  - [x] Test inactive admin retains admin flag
  - [x] Test default user is not admin
  - [x] Test system has at least one admin

### Documentation
- [ ] Document permission model in README (Admin vs User)
- [ ] Update API documentation with admin-only endpoints

---

## Phase 2: Admin User Management ✅ COMPLETED
**Priority:** MEDIUM - Requires Phase 1  
**Estimated Time:** 2-3 days  
**Status:** ✅ Complete (2025-01-11)  
**Dependencies:** ✅ Phase 1 (Permissions System)

### Overview
Add administrative capabilities to create, edit, and delete users through the settings page.

### Backend Tasks
- [x] Create `src/handlers/admin/users.rs`
  - [x] `create_user()` - Admin creates new user (POST /api/v1/admin/users)
  - [x] `list_users()` - List all users with pagination (GET /api/v1/admin/users)
  - [x] `get_user()` - Get specific user details (GET /api/v1/admin/users/:id)
  - [x] `update_user()` - Update user info (PUT /api/v1/admin/users/:id)
  - [x] `delete_user()` - Soft delete user (DELETE /api/v1/admin/users/:id)
  - [x] `toggle_user_active()` - Enable/disable user (PATCH /api/v1/admin/users/:id/active)
  - [x] `admin_change_password()` - Admin resets user password (PATCH /api/v1/admin/users/:id/password)

- [x] Create `src/routes/admin.rs`
  - [x] Define admin routes with `with_admin()` middleware
  - [x] All endpoints require admin permission

- [x] Update `src/models/user.rs`
  - [x] Add `AdminCreateUserRequest` (includes admin flag)
  - [x] Add `AdminUpdateUserRequest`
  - [x] Add `AdminUserListResponse` (with pagination)

### Frontend Tasks
- [x] Create user management section in settings page
  - [x] Add "User Management" section (admin only)
  - [x] Create user list table with columns:
    - Username, Email, Full Name, Admin Badge, Active Status, Created Date, Actions
  - [x] Add pagination controls
  - [x] Full-width table layout

- [x] Create user creation modal
  - [x] Username input with validation
  - [x] Email input with validation
  - [x] Full name input
  - [x] Password input with strength requirements
  - [x] Admin checkbox
  - [x] Active checkbox
  - [x] Submit button with loading state

- [x] Create user edit modal
  - [x] Pre-populate fields with existing data
  - [x] Allow editing all fields
  - [x] Password field hidden (use reset password instead)
  - [x] Admin checkbox toggle
  - [x] Active/Inactive toggle

- [x] Create password reset modal
  - [x] Admin can reset any user's password
  - [x] No current password required
  - [x] Password confirmation field

- [x] Add delete confirmation dialog
  - [x] Warning about data deletion
  - [x] Soft delete implementation (sets is_active=false)
  - [x] Require confirmation

### Validation Tasks
- [x] Backend validation
  - [x] Username uniqueness check
  - [x] Email format validation
  - [x] Email uniqueness check
  - [x] Password strength requirements (min 8 chars)

- [x] Frontend validation
  - [x] Email format validation
  - [x] Password strength validation
  - [x] Required field validation
  - [x] Password confirmation matching

### Testing Tasks
- [ ] Create `tests/admin_user_tests.rs` (manual testing complete)
  - [x] Test user creation by admin (manually verified)
  - [x] Test user listing with pagination (manually verified)
  - [x] Test user editing (manually verified)
  - [x] Test user deletion (manually verified)
  - [x] Test admin flag toggle (manually verified)
  - [x] Test active/inactive toggle (manually verified)
  - [x] Test password reset (manually verified)

### Implementation Notes
- Simplified from original plan - removed complex RBAC in favor of simple Admin/User roles
- Used existing `is_admin` and `is_active` flags instead of adding new tables
- Soft delete implementation (deactivation) instead of hard delete
- All admin actions properly logged via tracing
- Alert-based notifications (showNotification function not implemented yet)

---

## Phase 3: User Data Isolation Audit 🔍 READY TO START
**Priority:** HIGH - Security Critical  
**Estimated Time:** 1-2 days  
**Status:** ✅ Unblocked - Phase 1 Complete  
**Dependencies:** ✅ Phase 1 (Permissions System)

### Overview
Audit and verify that all data access is properly scoped to the authenticated user or shared with explicit permission.

### Current State
- ✅ Humidors have `user_id` foreign key
- ✅ Favorites have `user_id` foreign key with CASCADE delete
- ✅ Wish list has `user_id` foreign key with CASCADE delete
- ✅ JWT authentication middleware exists
- ❓ Need to verify all handlers respect user boundaries

### Audit Tasks

#### Humidor Handlers (`src/handlers/humidors.rs`)
- [ ] Review `get_humidors()` - filters by `auth.user_id`
- [ ] Review `get_humidor()` - verifies ownership
- [ ] Review `create_humidor()` - sets `user_id` from auth
- [ ] Review `update_humidor()` - verifies ownership
- [ ] Review `delete_humidor()` - verifies ownership

#### Cigar Handlers (`src/handlers/cigars.rs`)
- [ ] Review `get_cigars()` - filters by humidor ownership
- [ ] Review `get_cigar()` - verifies humidor ownership
- [ ] Review `create_cigar()` - verifies humidor ownership
- [ ] Review `update_cigar()` - verifies humidor ownership
- [ ] Review `delete_cigar()` - verifies humidor ownership

#### Favorite Handlers (`src/handlers/favorites.rs`)
- [ ] Review `get_favorites()` - filters by `auth.user_id`
- [ ] Review `add_favorite()` - sets `user_id` from auth
- [ ] Review `remove_favorite()` - verifies ownership
- [ ] Review favorite snapshot data preservation

#### Wish List Handlers (`src/handlers/wish_list.rs`)
- [ ] Review `get_wish_list()` - filters by `auth.user_id`
- [ ] Review `add_to_wish_list()` - sets `user_id` from auth
- [ ] Review `remove_from_wish_list()` - verifies ownership
- [ ] Review `update_wish_list_notes()` - verifies ownership

#### Organizer Handlers (Brands, Sizes, Origins, Strengths, Ring Gauges)
- [ ] Determine if organizers should be:
  - [ ] Global (shared across all users) - Current implementation
  - [ ] User-specific (each user has own organizers)
  - [ ] Hybrid (system defaults + user custom)
- [ ] Document decision and rationale

### Testing Tasks
- [ ] Create `tests/security_isolation_tests.rs`
  - [ ] Test User A cannot access User B's humidors
  - [ ] Test User A cannot access User B's cigars
  - [ ] Test User A cannot access User B's favorites
  - [ ] Test User A cannot access User B's wish list
  - [ ] Test SQL injection attempts on user_id filters
  - [ ] Test JWT token manipulation attempts
  - [ ] Test cascade delete preserves user boundaries

### Code Review Checklist
- [ ] All database queries filter by `user_id` or verify ownership
- [ ] No hardcoded user IDs in queries
- [ ] All foreign key constraints include CASCADE rules
- [ ] Authentication middleware applied to all protected routes
- [ ] Error messages don't leak information about other users' data
- [ ] Logging doesn't expose sensitive user data

### Documentation
- [ ] Document data isolation model
- [ ] Create security architecture diagram
- [ ] Add section to README about multi-user support

---

## Phase 4: Humidor Sharing 🤝 TODO
**Priority:** LOW - Feature Enhancement  
**Estimated Time:** 4-6 days  
**Status:** Blocked by Phases 1, 2, 3  
**Dependencies:** Phase 1 (Permissions), Phase 2 (User Management), Phase 3 (Isolation Audit)

### Overview
Allow users to share their humidors with other users with configurable permission levels.

### Database Tasks
- [ ] Create migration `V13__create_humidor_sharing.sql`
  - [ ] Create `humidor_shares` table
    - `id` UUID PRIMARY KEY
    - `humidor_id` UUID REFERENCES humidors (ON DELETE CASCADE)
    - `shared_with_user_id` UUID REFERENCES users (ON DELETE CASCADE)
    - `shared_by_user_id` UUID REFERENCES users
    - `permission_level` VARCHAR(20) - 'view', 'edit', 'full'
    - `created_at` TIMESTAMPTZ
    - UNIQUE constraint on (humidor_id, shared_with_user_id)
  - [ ] Create indexes on `humidor_id` and `shared_with_user_id`

### Permission Levels
- **view** - Read-only access to cigars in the humidor
- **edit** - Can add and edit cigars (but not delete)
- **full** - Can add, edit, delete cigars and manage sharing

### Backend Tasks
- [ ] Create `src/models/humidor_share.rs`
  - [ ] Define `HumidorShare` struct
  - [ ] Define `ShareHumidorRequest` (user_id, permission_level)
  - [ ] Define `HumidorShareResponse`
  - [ ] Define `PermissionLevel` enum

- [ ] Create `src/handlers/humidor_shares.rs`
  - [ ] `share_humidor()` - Share with user (POST /api/v1/humidors/:id/share)
  - [ ] `revoke_share()` - Revoke access (DELETE /api/v1/humidors/:id/share/:user_id)
  - [ ] `update_share_permission()` - Change permission level (PATCH /api/v1/humidors/:id/share/:user_id)
  - [ ] `get_shared_humidors()` - List humidors shared with me (GET /api/v1/humidors/shared)
  - [ ] `get_humidor_shares()` - List who I've shared with (GET /api/v1/humidors/:id/shares)

- [ ] Update `src/handlers/humidors.rs`
  - [ ] Modify `get_humidors()` to include shared humidors
  - [ ] Modify `get_humidor()` to check share permissions
  - [ ] Add ownership/permission helper functions

- [ ] Update `src/handlers/cigars.rs`
  - [ ] Check share permissions before allowing operations
  - [ ] Respect permission levels (view/edit/full)

- [ ] Create helper functions
  - [ ] `can_view_humidor(user_id, humidor_id)` -> bool
  - [ ] `can_edit_humidor(user_id, humidor_id)` -> bool
  - [ ] `can_manage_humidor(user_id, humidor_id)` -> bool
  - [ ] `get_user_permission_level(user_id, humidor_id)` -> PermissionLevel

### Frontend Tasks
- [ ] Update humidor detail page
  - [ ] Add "Share" button (owner only)
  - [ ] Add shared indicator badge
  - [ ] Display "Shared by [username]" for shared humidors

- [ ] Create share management modal
  - [ ] User search/autocomplete
  - [ ] Permission level dropdown
  - [ ] "Add User" button
  - [ ] List of currently shared users with:
    - Username, Email, Permission Level, Shared Date
    - Edit permission button
    - Remove access button

- [ ] Create "Shared with Me" section
  - [ ] New tab/section showing shared humidors
  - [ ] Display owner information
  - [ ] Show my permission level
  - [ ] Visual distinction from owned humidors

- [ ] Update humidor list
  - [ ] Add visual indicator for shared humidors (icon)
  - [ ] Add filter: "My Humidors" / "Shared with Me" / "All"
  - [ ] Show owner name on shared humidor cards

- [ ] Permission-based UI updates
  - [ ] Hide edit/delete buttons for view-only access
  - [ ] Hide delete button for edit-only access
  - [ ] Hide share management for non-owners

### Validation Tasks
- [ ] Backend validation
  - [ ] Cannot share with yourself
  - [ ] Cannot share if not owner
  - [ ] User to share with must exist
  - [ ] Valid permission level
  - [ ] Check for existing share before creating

- [ ] Frontend validation
  - [ ] Real-time user existence check
  - [ ] Prevent selecting already-shared users
  - [ ] Clear error messages

### Testing Tasks
- [ ] Create `tests/humidor_sharing_tests.rs`
  - [ ] Test sharing humidor with another user
  - [ ] Test different permission levels (view/edit/full)
  - [ ] Test revoking access
  - [ ] Test updating permission levels
  - [ ] Test shared user can access cigars
  - [ ] Test shared user respects permission limits
  - [ ] Test owner can always manage
  - [ ] Test cascading delete when humidor deleted
  - [ ] Test cascading delete when user deleted
  - [ ] Test shared humidor appears in shared list

### Security Considerations
- [ ] Only owner can share humidor
- [ ] Only owner can revoke access
- [ ] Cannot escalate own permissions
- [ ] Audit log for all sharing actions
- [ ] Notification system for share invitations (future enhancement)

### Edge Cases
- [ ] What happens if owner deletes humidor? (CASCADE - shares deleted)
- [ ] What happens if shared user is deleted? (CASCADE - shares deleted)
- [ ] Can owner remove their own ownership? (No - must delete humidor)
- [ ] Can shared users see other shared users? (Design decision needed)

---

## Progress Tracking

### Completed
- ✅ Basic authentication system
- ✅ User-specific humidors
- ✅ User-specific favorites
- ✅ User-specific wish lists
- ✅ **Phase 1: Permissions System (2025-01-10)**

### In Progress
- Nothing currently

### Ready to Start
- 🚀 Phase 2: Admin User Management (unblocked)
- 🚀 Phase 3: Data Isolation Audit (unblocked)

### Blocked
- ⏸️ Phase 4: Humidor Sharing (waiting on Phases 2 & 3)

---

## Notes & Decisions

### Design Decisions to Make
1. **Organizer Scope**: Should brands, sizes, origins, etc. be:
   - Global (all users share same organizers) ← **RECOMMENDED**
   - User-specific (each user maintains their own)
   - Hybrid (system defaults + user additions)
   
2. **Sharing Notifications**: Should users receive notifications when:
   - Humidor is shared with them?
   - Their access is revoked?
   - Shared humidor is deleted?

3. **Soft Delete vs Hard Delete**: 
   - Users - soft delete (deactivate) or hard delete?
   - Humidors - should deleted humidors be recoverable?

4. **Share Visibility**: Can shared users see:
   - Other shared users on the same humidor?
   - Sharing history/audit log?

### Technical Debt
- Consider adding notification system for future enhancements
- Consider adding activity audit log for compliance
- Consider adding user groups for bulk sharing
- Consider adding share expiration dates

---

## Getting Started

To begin Phase 1, run:
```bash
# Create the migration file
touch migrations/V12__create_permissions_system.sql

# Start implementing models
touch src/models/role.rs
touch src/models/permission.rs
```

Update this document as tasks are completed by checking off the boxes: `- [ ]` → `- [x]`
