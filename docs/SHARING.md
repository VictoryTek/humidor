# Humidor Sharing Guide

Share your cigar collection with friends, family, or other users with fine-grained permission control.

## Overview

Humidor sharing allows you to:
- 👥 Share your humidors with other users
- 🔐 Control what they can do with 3 permission levels
- 👀 See humidors others have shared with you
- 🔄 Update or revoke access at any time

## Permission Levels

### View (Read-Only)
**Best for**: Showing your collection to friends, collectors

Shared users can:
- ✅ See all cigars in the humidor
- ✅ View cigar details and notes
- ✅ Search and filter cigars
- ❌ Cannot add, edit, or delete cigars
- ❌ Cannot manage sharing

**Example**: Share your collection with a friend who wants to see what you have.

### Edit
**Best for**: Joint collections, family humidors

Shared users can:
- ✅ Everything in "View" level
- ✅ Add new cigars to the humidor
- ✅ Edit existing cigar details
- ✅ Update quantities
- ❌ Cannot delete cigars
- ❌ Cannot manage sharing

**Example**: Share with a spouse or roommate who can add cigars they purchase.

### Full (Manage)
**Best for**: Co-owners, trusted collaborators

Shared users can:
- ✅ Everything in "Edit" level
- ✅ Delete cigars from the humidor
- ✅ See who the humidor is shared with
- ❌ Cannot delete the humidor itself
- ❌ Cannot share with additional users (only owner can)

**Example**: Share with a business partner managing a shared collection.

## Sharing a Humidor

### Step-by-Step

1. **Open Share Dialog**
   - Navigate to your humidors
   - Find the humidor you want to share
   - Click the **Share button** (👥 icon)

2. **Select User**
   - Choose a user from the dropdown
   - Only active users are shown
   - You cannot share with yourself

3. **Choose Permission Level**
   - Select View, Edit, or Full
   - Consider what the user needs to do
   - You can change this later

4. **Add User**
   - Click **"Add User"**
   - The user appears in the shared users list
   - They can immediately access the humidor

### Managing Shared Access

The share dialog shows:
- **Currently shared users**
- **Their permission levels**
- **Options to edit or remove**

#### Change Permissions
1. Find the user in the list
2. Select a new permission level from the dropdown
3. Changes apply immediately

#### Revoke Access
1. Find the user in the list
2. Click the **Remove** button (🗑️)
3. Confirm the removal
4. User immediately loses access

## Accessing Shared Humidors

### Finding Shared Humidors

Shared humidors appear in your humidor list with:
- 👥 Share icon indicator
- Owner's name displayed
- Your permission level shown

### Working with Shared Humidors

When viewing a shared humidor:
- **View permission**: Read-only mode, no edit buttons
- **Edit permission**: Can add/edit cigars, no delete buttons
- **Full permission**: All operations except sharing management

### Limitations

You **cannot**:
- ❌ Delete a humidor you don't own
- ❌ Share someone else's humidor
- ❌ Change the humidor's name (owner only)
- ❌ Escalate your own permissions

## Examples and Use Cases

### Family Collection
**Scenario**: You and your spouse maintain a shared humidor

**Setup**:
- Create a humidor called "Family Collection"
- Share it with your spouse with **Edit** permission
- Both can add purchases
- You retain ability to delete (Full control as owner)

### Show Off Your Collection
**Scenario**: Display your collection to fellow enthusiasts

**Setup**:
- Share your humidor with **View** permission
- Friends can browse but not modify
- They can see what you have for trading
- Your collection stays protected

### Cigar Club
**Scenario**: Group of friends share a communal locker

**Setup**:
- One person creates the humidor
- Share with all members at **Full** permission
- Everyone can add, edit, and remove
- Owner maintains ultimate control

### Business Partnership
**Scenario**: Co-owners of a cigar lounge inventory

**Setup**:
- Create humidors for different sections
- Share with business partner at **Full** level
- Both manage inventory
- Both track quantities

## Security and Privacy

### Ownership Rules

1. **Only the owner can**:
   - Delete the humidor
   - Share with new users
   - Remove users' access
   - Change sharing permissions

2. **Shared users cannot**:
   - Take ownership
   - See the owner's other humidors
   - Share with additional users
   - Change their own permissions

### Data Protection

- ✅ Sharing only affects the specific humidor
- ✅ Your other humidors remain private
- ✅ Favorites and wish lists are never shared
- ✅ Personal notes on cigars are visible to shared users
- ✅ All changes are logged with user information

### What Happens If...

**Owner deletes the humidor?**
- All shares are automatically removed
- Shared users lose access immediately
- No notification is sent (currently)

**Owner is deactivated?**
- Humidor remains accessible to shared users
- Existing permissions continue working
- No new shares can be created

**Shared user is deactivated?**
- Their shares are automatically removed
- Owner can share with them again if reactivated

**Owner deletes a cigar?**
- Shared users lose access to that cigar
- No permission level can prevent owner actions

## Tips and Best Practices

### For Owners

✅ **Start with View permission**
- Upgrade to Edit/Full as trust is established
- Easier to add permissions than remove them

✅ **Use descriptive humidor names**
- Makes it clear what's being shared
- Helps shared users understand the collection

✅ **Review shared access regularly**
- Remove users who no longer need access
- Adjust permissions as needs change

✅ **Document expectations**
- Tell shared users what you expect
- Clarify who buys what, who smokes what

### For Shared Users

✅ **Respect permission levels**
- Don't try to circumvent restrictions
- Ask the owner if you need more access

✅ **Communicate changes**
- Let the owner know when you add cigars
- Discuss before making major changes

✅ **Keep accurate quantities**
- Update counts when you smoke
- Help maintain accurate inventory

✅ **Add helpful notes**
- Share your tasting notes
- Document purchase locations

## Common Questions

### Can I share with multiple users?
Yes! Share with as many users as you want, each with different permission levels.

### Can I share multiple humidors with the same person?
Yes! Each humidor can be shared independently with different permissions.

### Will shared users see my favorites?
No. Favorites and wish lists are always private.

### Can shared users favorite cigars in a shared humidor?
Yes, but their favorites are private to them. You won't see their favorite markers.

### Can I prevent someone from seeing certain cigars?
No. Sharing is all-or-nothing for a humidor. Consider creating a separate humidor for private cigars.

### What if two people edit the same cigar at once?
The last save wins. Database updates are atomic, so no data corruption, but one person's changes might overwrite the other's.

### Can admins see my shared humidors?
No. Admins have no special access to user data unless you explicitly share with them.

### Is there a limit to how many people I can share with?
No hard limit, but consider using Edit permission sparingly to avoid conflicts.

## Notifications (Future Feature)

Currently, sharing is silent - users must check their humidor list to see new shares.

Planned enhancements:
- Email notifications when a humidor is shared with you
- Alerts when access is revoked
- Notifications when shared humidor is deleted

## Related Documentation

- [User Permissions & Roles](PERMISSIONS.md)
- [User Guide](USER_GUIDE.md)
- [Security Model](SECURITY_MODEL.md)
- [API Documentation](API.md#humidor-sharing)
