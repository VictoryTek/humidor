# User Permissions & Roles

Humidor uses a simple two-tier permission system to control access and capabilities.

## Permission Levels

### Admin
**Full system access with user management capabilities**

Admins can:
- ✅ Create, edit, and delete users
- ✅ Assign or revoke admin privileges
- ✅ Activate or deactivate user accounts
- ✅ Reset any user's password
- ✅ Access all standard user features
- ✅ Manage global reference data (brands, sizes, origins, etc.)

**Use Case**: System administrators, application owners

### User (Standard)
**Standard access for managing personal cigar inventory**

Users can:
- ✅ Create and manage their own humidors
- ✅ Add, edit, and delete their own cigars
- ✅ Mark cigars as favorites
- ✅ Maintain a wish list
- ✅ Share their humidors with other users
- ✅ Access humidors shared with them (with appropriate permissions)
- ✅ Update their own profile and password
- ✅ View global reference data (brands, sizes, origins)
- ✅ Add new reference data for everyone to use

**Use Case**: Standard application users

## Data Isolation

### Ownership Model

Each user has complete control over their own data:

**Owned by User:**
- ✅ Humidors
- ✅ Cigars (through humidors)
- ✅ Favorites
- ✅ Wish list items

**Shared Resources:**
- 🌍 Brands
- 🌍 Sizes (vitolas)
- 🌍 Origins (countries)
- 🌍 Strengths
- 🌍 Ring gauges

### Security Guarantees

1. **Users cannot access other users' data** unless explicitly shared
2. **All API requests are authenticated** via JWT tokens
3. **Database queries filter by user_id** to enforce isolation
4. **Shared humidors respect permission levels** (view/edit/full)
5. **Ownership is verified** on all create/update/delete operations

### What This Means

- User A's cigars are completely invisible to User B
- User A cannot modify User B's humidors
- User A cannot favorite User B's cigars
- User A cannot see User B's wish list

**Exception**: When User B explicitly shares a humidor with User A (see [Sharing Guide](SHARING.md))

## User Account States

### Active
- Can log in and use the application
- All features are available
- Data is accessible

### Inactive (Deactivated)
- Cannot log in
- Account is "soft deleted"
- Data is preserved
- Can be reactivated by admin
- **Use Case**: Temporarily disable access without losing data

## Admin Capabilities

### User Management

Admins can access the **User Management** section in Settings to:

#### Create Users
- Set username, email, full name
- Generate initial password
- Assign admin privileges
- Set account status (active/inactive)

#### Edit Users
- Update user information
- Toggle admin privileges
- Activate or deactivate accounts
- **Note**: Cannot demote the last admin

#### Reset Passwords
- Reset any user's password
- No current password required
- User is notified (if email configured)

#### View User List
- See all users (active and inactive)
- Filter and search users
- Paginated for large user bases

### Safety Mechanisms

1. **Last Admin Protection**: Cannot demote or deactivate the last admin
2. **Self-Protection**: Admins receive warnings before changing their own status
3. **Audit Trail**: All admin actions are logged

## Permission Checks

### Backend (API)
All endpoints enforce permissions:

```
Standard Endpoints: Require authentication
Admin Endpoints:    Require admin role
Public Endpoints:   Reference data (read-only)
```

### Frontend (UI)
The interface adapts based on permissions:

- Admin-only sections are hidden for standard users
- Action buttons respect permissions
- Shared humidor controls respect sharing permissions

## Best Practices

### For Admins
- ✅ Create users with appropriate initial permissions
- ✅ Regularly review active users
- ✅ Use deactivation instead of deletion to preserve data
- ✅ Maintain at least 2 admin accounts
- ✅ Keep admin privileges limited to trusted users

### For Users
- ✅ Keep your profile information up to date
- ✅ Use strong passwords
- ✅ Don't share your account credentials
- ✅ Use humidor sharing instead of sharing accounts
- ✅ Report any security concerns to your admin

## Upgrading/Downgrading

### Making a User an Admin
1. Admin navigates to User Management
2. Clicks edit on the user
3. Checks the "Admin" checkbox
4. Saves changes
5. User immediately gains admin privileges

### Removing Admin Privileges
1. Admin navigates to User Management
2. Clicks edit on the admin user
3. Unchecks the "Admin" checkbox
4. Confirms the change
5. User retains all their data but loses admin capabilities

**Note**: The system prevents removing admin privileges from the last admin account.

## Common Questions

### Can I have multiple admins?
Yes! You can have as many admin users as needed. This is actually recommended for redundancy.

### What happens to data when a user is deactivated?
All data is preserved. Humidors, cigars, favorites, and wish list items remain in the database. If the account is reactivated, everything is restored.

### Can users see each other?
Users can only see other users when:
- Sharing a humidor (to select who to share with)
- Viewing who has access to their shared humidors
- In the admin user list (admins only)

### Can users promote themselves to admin?
No. Only existing admins can grant admin privileges.

### What if I forget my password?
Use the "Forgot Password" feature on the login page. If email is configured, you'll receive a reset link. Otherwise, contact your admin.

## Related Documentation

- [Admin User Management Guide](ADMIN_GUIDE.md)
- [Humidor Sharing](SHARING.md)
- [Security Model](SECURITY_MODEL.md)
- [API Authentication](API.md#authentication)
