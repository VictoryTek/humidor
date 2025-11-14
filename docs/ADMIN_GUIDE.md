# Administrator Guide

This guide covers administrative tasks for managing users and maintaining the Humidor application.

## Table of Contents
- [Admin Dashboard](#admin-dashboard)
- [User Management](#user-management)
- [Security Best Practices](#security-best-practices)
- [Maintenance Tasks](#maintenance-tasks)
- [Troubleshooting](#troubleshooting)

## Admin Dashboard

### Accessing Admin Features

1. Log in with an admin account
2. Click your username in the top-right
3. Select **"Settings"**
4. The **"User Management"** section appears (admin only)

### Admin Indicators

Admin accounts have:
- 👑 "Admin" badge on user profile
- User Management section in settings
- Additional API access (see [API Documentation](API.md))

## User Management

### Creating Users

As an admin, you can create accounts for other users.

#### Steps
1. Navigate to **Settings > User Management**
2. Click **"Create New User"**
3. Fill in the form:
   - **Username**: Unique, 3-50 characters, alphanumeric
   - **Email**: Valid email address (required)
   - **Full Name**: User's display name
   - **Password**: Initial password (minimum 8 characters)
   - **Admin**: Check if user should have admin privileges
   - **Active**: Check to enable the account immediately
4. Click **"Create User"**

#### Initial Credentials
- Provide the username and password to the user
- Recommend they change their password immediately
- They can update their profile after first login

### Viewing Users

The user list displays:
- Username and email
- Full name
- Admin status (👑 badge)
- Active/Inactive status
- Account creation date
- Action buttons (Edit, Delete)

#### Filtering and Search
- Use the search box to find users by username or email
- Filter by status (Active/Inactive)
- Sort by username, email, or creation date

### Editing Users

Modify existing user accounts.

#### Steps
1. Find the user in the list
2. Click the **Edit button** (✏️)
3. Update any fields:
   - Email address
   - Full name
   - Admin status (see warnings below)
   - Active status
4. Click **"Update User"**

#### Important Notes
- Cannot edit username (permanent identifier)
- Password changes use separate reset function
- Changing admin status has immediate effect

### Resetting Passwords

Admins can reset any user's password without knowing the current one.

#### Steps
1. Find the user in the list
2. Click **"Reset Password"**
3. Enter the new password
4. Confirm the password
5. Click **"Reset Password"**

#### Best Practices
- Generate strong passwords
- Provide password securely to user
- Advise user to change it after first login
- Use password reset for locked-out users

### Deactivating Users

Temporarily disable a user account without deleting data.

#### Steps
1. Edit the user
2. Uncheck **"Active"**
3. Save changes

#### Effects
- ✅ User cannot log in
- ✅ All user data is preserved
- ✅ Can be reactivated later
- ✅ Shares are removed but can be recreated

#### Use Cases
- Temporary access suspension
- Former employees (preserve data)
- Security incidents (immediate lockout)
- Testing access controls

### Reactivating Users

Re-enable a deactivated account.

#### Steps
1. Edit the deactivated user
2. Check **"Active"**
3. Save changes
4. User can log in immediately
5. Data is fully restored

### Managing Admin Privileges

#### Promoting to Admin
1. Edit the user
2. Check **"Admin"**
3. Confirm the change
4. User immediately gains admin capabilities

#### Demoting from Admin
1. Edit the admin user
2. Uncheck **"Admin"**
3. System prevents if this is the last admin
4. Confirm the change
5. User retains all their data

⚠️ **Warning**: Cannot remove admin privileges from the last admin account. Create another admin first.

## Security Best Practices

### Account Security

✅ **Multiple Admins**
- Maintain at least 2 active admin accounts
- Prevents lockout if one admin is unavailable
- Provides redundancy for critical operations

✅ **Strong Passwords**
- Require minimum 8 characters
- Encourage complexity (mix of letters, numbers, symbols)
- Recommend password managers

✅ **Regular Audits**
- Review user list monthly
- Deactivate unused accounts
- Verify admin privileges are appropriate

✅ **Principle of Least Privilege**
- Grant admin access only when necessary
- Most users should be standard users
- Review admin list regularly

### Data Protection

✅ **Backups**
- Regular database backups (see Docker volumes)
- Test restore procedures
- Keep backups secure and encrypted

✅ **Access Logs**
- Monitor application logs for suspicious activity
- Watch for failed login attempts
- Review admin actions in logs

✅ **Environment Security**
- Keep `JWT_SECRET` confidential
- Use strong database passwords
- Enable HTTPS in production
- Configure CORS appropriately

### Incident Response

If a security incident occurs:

1. **Immediate Actions**
   - Deactivate compromised account(s)
   - Review access logs
   - Check for unauthorized changes

2. **Investigation**
   - Identify scope of access
   - Review recent activity
   - Check shared humidors

3. **Remediation**
   - Reset affected passwords
   - Revoke suspicious sessions (restart app)
   - Update security measures

4. **Prevention**
   - Update passwords
   - Review access controls
   - Document lessons learned

## Maintenance Tasks

### Routine Maintenance

#### Weekly
- Review new user accounts
- Check for deactivation requests
- Monitor application logs

#### Monthly
- Audit admin accounts
- Review inactive users
- Check database backups
- Update dependencies (if applicable)

#### Quarterly
- Full security review
- Update documentation
- Test disaster recovery

### Database Management

#### Backing Up Data
```bash
# PostgreSQL backup
docker-compose exec db pg_dump -U humidor_user humidor_db > backup_$(date +%Y%m%d).sql
```

#### Restoring Data
```bash
# PostgreSQL restore
docker-compose exec -T db psql -U humidor_user humidor_db < backup_20250113.sql
```

#### Checking Database Size
```bash
# View database size
docker-compose exec db psql -U humidor_user -d humidor_db -c "SELECT pg_size_pretty(pg_database_size('humidor_db'));"
```

### Log Management

#### Viewing Logs
```bash
# Application logs
docker-compose logs -f web

# Database logs
docker-compose logs -f db

# Last 100 lines
docker-compose logs --tail=100 web
```

#### Log Levels
Adjust `RUST_LOG` environment variable:
- `error` - Errors only
- `warn` - Warnings and errors
- `info` - General information (recommended)
- `debug` - Detailed debugging
- `trace` - Very verbose (development only)

### Updates and Migrations

#### Updating the Application
1. Pull latest code/image
2. Check migration files
3. Backup database
4. Run migrations (automatic on startup)
5. Restart services
6. Verify functionality

#### Database Migrations
See [MIGRATIONS.md](MIGRATIONS.md) for detailed migration information.

## Troubleshooting

### Common Issues

#### Users Can't Log In

**Symptoms**: "Invalid credentials" error

**Solutions**:
1. Verify account is active
2. Check username (case-sensitive)
3. Reset password if forgotten
4. Check application logs for errors

#### Admin Can't Access User Management

**Symptoms**: User Management section not visible

**Solutions**:
1. Verify user has admin flag in database
2. Refresh browser/clear cache
3. Check JWT token is valid
4. Log out and log back in

#### Can't Create Last Admin

**Symptoms**: Error when trying to create only admin account

**Solutions**:
1. This is a safety feature - maintain multiple admins
2. Create another admin account first
3. Then remove admin from original account

#### Database Connection Errors

**Symptoms**: "Database unavailable" or connection errors

**Solutions**:
1. Check database container is running: `docker-compose ps`
2. Verify DATABASE_URL is correct
3. Check database logs: `docker-compose logs db`
4. Restart services: `docker-compose restart`

#### Migration Failures

**Symptoms**: Application won't start after update

**Solutions**:
1. Check migration logs in application output
2. Verify database state
3. Restore from backup if needed
4. Contact support with error messages

### Performance Issues

#### Slow Queries

**Indicators**:
- Slow page loads
- Timeout errors
- High database CPU

**Solutions**:
1. Check database indexes (see [QUERY_OPTIMIZATION.md](QUERY_OPTIMIZATION.md))
2. Review slow query logs
3. Consider database vacuum/analyze
4. Increase database resources

#### Memory Issues

**Indicators**:
- Container restarts
- Out of memory errors
- Slow performance

**Solutions**:
1. Check container memory limits
2. Review application logs for leaks
3. Increase memory allocation in docker-compose.yml
4. Monitor with `docker stats`

### Getting Help

1. Check application logs first
2. Review this documentation
3. Check GitHub issues
4. Contact maintainers with:
   - Error messages
   - Log excerpts
   - Steps to reproduce
   - Environment details

## Related Documentation

- [User Permissions & Roles](PERMISSIONS.md)
- [Security Model](SECURITY_MODEL.md)
- [Database Migrations](MIGRATIONS.md)
- [API Documentation](API.md)
