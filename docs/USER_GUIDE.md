# Humidor User Guide

This guide covers the basic usage of Humidor for managing your cigar collection.

## Table of Contents
- [Getting Started](#getting-started)
- [Managing Humidors](#managing-humidors)
- [Managing Cigars](#managing-cigars)
- [Favorites](#favorites)
- [Wish List](#wish-list)
- [Sharing Humidors](#sharing-humidors)
- [Profile Management](#profile-management)

## Getting Started

### First Time Setup

1. **Access the Application**
   - Navigate to `http://localhost:9898` (or your configured URL)
   - You'll be redirected to the setup page on first launch

2. **Create Your Admin Account**
   - Enter a username (alphanumeric, 3-50 characters)
   - Provide a valid email address
   - Enter your full name
   - Create a strong password (minimum 8 characters)
   - Click "Create Admin Account"

3. **Login**
   - After setup, you'll be redirected to the login page
   - Enter your username and password
   - Click "Login"

### Dashboard Overview

After logging in, you'll see:
- **Top Navigation**: Search bar, Filter options, Profile menu
- **Humidor List**: Your humidors with cigar counts
- **Actions**: Buttons to add humidors and manage your collection

## Managing Humidors

### Creating a Humidor

1. Click the **"Add Humidor"** button on the dashboard
2. Enter a name for your humidor
3. Click **"Add Humidor"**
4. Your new humidor appears in the list

### Viewing Humidor Contents

1. Click on a humidor card to view its contents
2. You'll see:
   - List of all cigars in the humidor
   - Quick actions (edit, delete)
   - Option to add new cigars

### Editing a Humidor

1. Click the **edit icon** (✏️) on a humidor card
2. Modify the humidor name
3. Click **"Update Humidor"**

### Deleting a Humidor

1. Click the **delete icon** (🗑️) on a humidor card
2. Confirm the deletion
3. **Note**: This will delete all cigars in the humidor

### Sharing a Humidor

See the [Humidor Sharing Guide](SHARING.md) for detailed instructions on sharing humidors with other users.

## Managing Cigars

### Adding a Cigar

1. Navigate to a humidor
2. Click **"Add Cigar"**
3. Fill in the cigar details:
   - **Name**: Required
   - **Brand**: Select from dropdown or add new
   - **Size**: Select vitola (e.g., Robusto, Toro)
   - **Strength**: Mild, Medium, Full, etc.
   - **Origin**: Country of origin
   - **Ring Gauge**: Thickness in 64ths of an inch
   - **Length**: Length in inches
   - **Wrapper/Binder/Filler**: Tobacco details
   - **Price**: Purchase price
   - **Purchase Date**: When you acquired it
   - **Quantity**: Number of cigars
   - **Notes**: Personal notes
   - **Retail Link**: Link to online retailer
   - **Image URL**: Link to cigar image
4. Click **"Add Cigar"**

### Editing a Cigar

1. Click the **edit icon** on a cigar
2. Modify any fields
3. Click **"Update Cigar"**

### Moving a Cigar

1. Edit the cigar
2. Change the **Humidor** dropdown to a different humidor
3. Click **"Update Cigar"**

### Deleting a Cigar

1. Click the **delete icon** on a cigar
2. Confirm the deletion

### Search and Filter

- **Search Bar**: Type to search by cigar name, brand, or notes
- **Filter by Brand**: Select a specific brand from dropdown
- **Filter by Strength**: Filter by strength level
- **Filter by Origin**: Filter by country of origin

## Favorites

Mark cigars you particularly enjoy for easy access.

### Adding to Favorites

1. View a cigar's details
2. Click the **star icon** (☆)
3. The star fills (★) to indicate it's favorited
4. Optionally add notes about why you like it

### Viewing Favorites

1. Navigate to **"Favorites"** in the top menu
2. See all your favorited cigars
3. Click on any cigar to view full details

### Removing from Favorites

1. Click the **filled star icon** (★) on a favorited cigar
2. The cigar is removed from favorites

## Wish List

Track cigars you want to try or purchase.

### Adding to Wish List

1. Click **"Add to Wish List"** button
2. Enter cigar details (name, brand, etc.)
3. Add notes about where to find it or why you want it
4. Click **"Add to Wish List"**

### Managing Wish List

1. Navigate to **"Wish List"** in the menu
2. View all cigars on your wish list
3. Edit notes or details as needed
4. Remove items once purchased

### Moving from Wish List to Inventory

1. When you acquire a cigar from your wish list
2. Add it to your humidor as a regular cigar
3. Remove it from the wish list

## Profile Management

### Updating Your Profile

1. Click your username in the top-right corner
2. Select **"Profile"**
3. Update:
   - Email address
   - Full name
   - Password (optional)
4. Click **"Update Profile"**

### Changing Your Password

1. Go to Profile page
2. Enter your current password
3. Enter new password
4. Confirm new password
5. Click **"Update Password"**

## Tips and Best Practices

### Organization
- Create separate humidors for different purposes (e.g., "Daily Smokes", "Special Occasions")
- Use descriptive names for your humidors
- Keep quantity counts updated

### Data Entry
- Add detailed notes - they're helpful when deciding what to smoke
- Include purchase dates to track aging
- Add retail links for easy reordering
- Use the image URL field for visual reference

### Maintenance
- Update quantities as you smoke cigars
- Mark favorites to remember what you enjoyed
- Add cigars to wish list when you discover new ones

## Troubleshooting

### Can't Log In
- Verify your username and password
- Check for typos (username is case-sensitive)
- Use the "Forgot Password" feature if needed

### Cigar Not Appearing
- Check which humidor you added it to
- Clear any active filters
- Refresh the page

### Changes Not Saving
- Ensure you clicked the save/update button
- Check for validation errors in red
- Verify you have edit permissions (for shared humidors)

## Next Steps

- Learn about [User Permissions & Roles](PERMISSIONS.md)
- Discover [Humidor Sharing](SHARING.md) features
- Explore the [API Documentation](API.md) for integrations
