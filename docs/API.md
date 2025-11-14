# API Documentation

Complete REST API reference for Humidor.

## Base URL

```
http://localhost:9898/api/v1
```

## Authentication

### Overview

Most API endpoints require authentication using JWT (JSON Web Tokens).

### Getting a Token

**POST** `/auth/login`

```json
{
  "username": "your_username",
  "password": "your_password"
}
```

**Response**: `200 OK`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "uuid",
    "username": "your_username",
    "email": "user@example.com",
    "full_name": "Your Name",
    "is_admin": false
  }
}
```

### Using the Token

Include the token in the `Authorization` header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token Expiration

Tokens expire after 24 hours. When a token expires, you'll receive `401 Unauthorized`.

## Error Responses

All errors follow a consistent format:

```json
{
  "error": "Error message describing what went wrong"
}
```

### Common Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid input or validation error |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Authenticated but not authorized |
| 404 | Not Found | Resource doesn't exist |
| 500 | Internal Server Error | Server error |

---

## Endpoints

### Authentication

#### Setup Admin Account

**POST** `/auth/setup`

Create the first admin account during initial setup.

**Request Body**:
```json
{
  "username": "admin",
  "email": "admin@example.com",
  "full_name": "Administrator",
  "password": "securepassword123"
}
```

**Response**: `200 OK`
```json
{
  "message": "Setup complete"
}
```

**Notes**:
- Only works if no users exist
- Creates admin with `is_admin=true`
- Returns 400 if setup already completed

#### Login

**POST** `/auth/login`

Authenticate and receive JWT token.

**Request Body**:
```json
{
  "username": "your_username",
  "password": "your_password"
}
```

**Response**: `200 OK` - See [Authentication](#authentication) section

#### Request Password Reset

**POST** `/auth/forgot-password`

Request a password reset token.

**Request Body**:
```json
{
  "email": "user@example.com"
}
```

**Response**: `200 OK`
```json
{
  "message": "If the email exists, a password reset link has been sent"
}
```

**Notes**:
- Returns success even if email doesn't exist (security)
- Token valid for 1 hour
- If SMTP not configured, reset URL logged to console

#### Reset Password

**POST** `/auth/reset-password`

Reset password using token from email.

**Request Body**:
```json
{
  "token": "reset_token_from_email",
  "new_password": "newSecurePassword123"
}
```

**Response**: `200 OK`
```json
{
  "message": "Password reset successful"
}
```

---

### User Profile

**Requires Authentication**

#### Get Current User

**GET** `/users/me`

Get authenticated user's profile.

**Response**: `200 OK`
```json
{
  "id": "uuid",
  "username": "user123",
  "email": "user@example.com",
  "full_name": "John Doe",
  "is_admin": false,
  "is_active": true,
  "created_at": "2025-01-13T10:00:00Z"
}
```

#### Update Profile

**PUT** `/users/me`

Update authenticated user's profile.

**Request Body**:
```json
{
  "email": "newemail@example.com",
  "full_name": "New Name"
}
```

**Response**: `200 OK`
```json
{
  "message": "Profile updated successfully"
}
```

#### Change Password

**PATCH** `/users/me/password`

Change your password.

**Request Body**:
```json
{
  "current_password": "oldPassword",
  "new_password": "newPassword123"
}
```

**Response**: `200 OK`
```json
{
  "message": "Password updated successfully"
}
```

---

### Humidors

**Requires Authentication**

#### List Humidors

**GET** `/humidors`

Get all humidors owned by or shared with current user.

**Query Parameters**:
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 20, max: 100)

**Response**: `200 OK`
```json
{
  "humidors": [
    {
      "id": "uuid",
      "name": "My Collection",
      "user_id": "uuid",
      "cigar_count": 42,
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-13T00:00:00Z"
    }
  ],
  "total": 5,
  "page": 1,
  "page_size": 20
}
```

#### Get Shared Humidors

**GET** `/humidors/shared`

Get humidors shared with current user (not owned).

**Response**: `200 OK`
```json
{
  "humidors": [
    {
      "id": "uuid",
      "name": "Friend's Collection",
      "owner_id": "uuid",
      "owner_username": "friend123",
      "permission_level": "view",
      "cigar_count": 15,
      "shared_at": "2025-01-10T00:00:00Z"
    }
  ]
}
```

#### Create Humidor

**POST** `/humidors`

Create a new humidor.

**Request Body**:
```json
{
  "name": "New Humidor"
}
```

**Response**: `201 Created`
```json
{
  "id": "uuid",
  "name": "New Humidor",
  "user_id": "uuid",
  "cigar_count": 0,
  "created_at": "2025-01-13T10:00:00Z",
  "updated_at": "2025-01-13T10:00:00Z"
}
```

#### Get Humidor

**GET** `/humidors/:id`

Get a specific humidor.

**Response**: `200 OK`
```json
{
  "id": "uuid",
  "name": "My Collection",
  "user_id": "uuid",
  "cigar_count": 42,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-13T00:00:00Z"
}
```

#### Update Humidor

**PUT** `/humidors/:id`

Update a humidor. Must be owner.

**Request Body**:
```json
{
  "name": "Updated Name"
}
```

**Response**: `200 OK`
```json
{
  "message": "Humidor updated successfully"
}
```

#### Delete Humidor

**DELETE** `/humidors/:id`

Delete a humidor. Must be owner.

**Response**: `200 OK`
```json
{
  "message": "Humidor deleted successfully"
}
```

---

### Humidor Sharing

**Requires Authentication**

#### Share Humidor

**POST** `/humidors/:id/share`

Share a humidor with another user. Must be owner.

**Request Body**:
```json
{
  "user_id": "uuid",
  "permission_level": "view"
}
```

**Permission Levels**: `view`, `edit`, `full`

**Response**: `201 Created`
```json
{
  "message": "Humidor shared successfully"
}
```

#### Get Humidor Shares

**GET** `/humidors/:id/shares`

List all users the humidor is shared with. Must be owner or have view permission.

**Response**: `200 OK`
```json
{
  "shares": [
    {
      "id": "uuid",
      "shared_with_user_id": "uuid",
      "shared_with_username": "friend123",
      "shared_with_email": "friend@example.com",
      "permission_level": "view",
      "shared_at": "2025-01-10T00:00:00Z"
    }
  ]
}
```

#### Update Share Permission

**PATCH** `/humidors/:id/share/:user_id`

Update permission level for a shared user. Must be owner.

**Request Body**:
```json
{
  "permission_level": "edit"
}
```

**Response**: `200 OK`
```json
{
  "message": "Permission updated successfully"
}
```

#### Revoke Share

**DELETE** `/humidors/:id/share/:user_id`

Revoke sharing access. Must be owner.

**Response**: `200 OK`
```json
{
  "message": "Share revoked successfully"
}
```

---

### Cigars

**Requires Authentication**

#### List Cigars

**GET** `/cigars`

Get all cigars in user's humidors (owned and shared).

**Query Parameters**:
- `page` (optional): Page number
- `page_size` (optional): Items per page
- `humidor_id` (optional): Filter by humidor
- `brand_id` (optional): Filter by brand
- `search` (optional): Search term

**Response**: `200 OK`
```json
{
  "cigars": [
    {
      "id": "uuid",
      "name": "Serie D No. 4",
      "brand_id": "uuid",
      "brand_name": "Partagás",
      "size_id": "uuid",
      "size_name": "Robusto",
      "strength_id": "uuid",
      "strength_name": "Full",
      "origin_id": "uuid",
      "origin_name": "Cuba",
      "ring_gauge_id": "uuid",
      "ring_gauge": 50,
      "length": 4.875,
      "quantity": 5,
      "humidor_id": "uuid",
      "price": 15.00,
      "is_favorite": false,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "page_size": 20
}
```

#### Create Cigar

**POST** `/cigars`

Add a new cigar to a humidor.

**Request Body**:
```json
{
  "name": "Serie D No. 4",
  "brand_id": "uuid",
  "size_id": "uuid",
  "strength_id": "uuid",
  "origin_id": "uuid",
  "ring_gauge_id": "uuid",
  "humidor_id": "uuid",
  "quantity": 5,
  "wrapper": "Habano",
  "binder": "Cuban",
  "filler": "Cuban",
  "price": 15.00,
  "purchase_date": "2025-01-13",
  "notes": "Excellent smoke",
  "retail_link": "https://example.com/cigar",
  "image_url": "https://example.com/image.jpg",
  "length": 4.875
}
```

**Required fields**: `name`, `humidor_id`

**Response**: `201 Created`
```json
{
  "id": "uuid",
  "message": "Cigar created successfully"
}
```

#### Get Cigar

**GET** `/cigars/:id`

Get a specific cigar. Must have view access to its humidor.

**Response**: `200 OK`
```json
{
  "id": "uuid",
  "name": "Serie D No. 4",
  "brand_id": "uuid",
  "brand_name": "Partagás",
  // ... full cigar details
}
```

#### Update Cigar

**PUT** `/cigars/:id`

Update a cigar. Must have edit permission on the humidor.

**Request Body**: Same as Create (all fields optional)

**Response**: `200 OK`
```json
{
  "message": "Cigar updated successfully"
}
```

#### Delete Cigar

**DELETE** `/cigars/:id`

Delete a cigar. Must have full permission on the humidor.

**Response**: `200 OK`
```json
{
  "message": "Cigar deleted successfully"
}
```

---

### Favorites

**Requires Authentication**

#### Get Favorites

**GET** `/favorites`

List all favorited cigars.

**Response**: `200 OK`
```json
{
  "favorites": [
    {
      "cigar_id": "uuid",
      "cigar_name": "Serie D No. 4",
      "brand_name": "Partagás",
      "notes": "Best robusto ever!",
      "created_at": "2025-01-10T00:00:00Z"
    }
  ]
}
```

#### Add Favorite

**POST** `/favorites`

Mark a cigar as favorite.

**Request Body**:
```json
{
  "cigar_id": "uuid",
  "notes": "Optional notes about why you like it"
}
```

**Response**: `201 Created`
```json
{
  "message": "Added to favorites"
}
```

#### Remove Favorite

**DELETE** `/favorites/:cigar_id`

Remove cigar from favorites.

**Response**: `200 OK`
```json
{
  "message": "Removed from favorites"
}
```

#### Check if Favorite

**GET** `/favorites/:cigar_id/check`

Check if a cigar is favorited.

**Response**: `200 OK`
```json
{
  "is_favorite": true
}
```

---

### Wish List

**Requires Authentication**

#### Get Wish List

**GET** `/wish-list`

List all cigars on wish list.

**Response**: `200 OK`
```json
{
  "items": [
    {
      "id": "uuid",
      "cigar_id": "uuid",
      "cigar_name": "Padrón 1964",
      "brand_name": "Padrón",
      "notes": "Must try this!",
      "created_at": "2025-01-12T00:00:00Z"
    }
  ]
}
```

#### Add to Wish List

**POST** `/wish-list`

Add a cigar to wish list.

**Request Body**:
```json
{
  "cigar_id": "uuid",
  "notes": "Want to try this"
}
```

**Response**: `201 Created`
```json
{
  "message": "Added to wish list"
}
```

#### Update Wish List Notes

**PATCH** `/wish-list/:cigar_id`

Update notes for a wish list item.

**Request Body**:
```json
{
  "notes": "Updated notes"
}
```

**Response**: `200 OK`
```json
{
  "message": "Wish list item updated"
}
```

#### Remove from Wish List

**DELETE** `/wish-list/:cigar_id`

Remove from wish list.

**Response**: `200 OK`
```json
{
  "message": "Removed from wish list"
}
```

---

### Reference Data (Organizers)

**Requires Authentication for CUD, Read-only without auth**

These endpoints manage global reference data.

#### Brands

**GET** `/brands` - List all brands  
**POST** `/brands` - Create brand: `{"name": "Brand Name"}`  
**GET** `/brands/:id` - Get specific brand  
**PUT** `/brands/:id` - Update brand: `{"name": "New Name"}`  
**DELETE** `/brands/:id` - Delete brand

#### Sizes (Vitolas)

**GET** `/sizes` - List all sizes  
**POST** `/sizes` - Create size: `{"name": "Robusto"}`  
**GET** `/sizes/:id` - Get specific size  
**PUT** `/sizes/:id` - Update size  
**DELETE** `/sizes/:id` - Delete size

#### Origins (Countries)

**GET** `/origins` - List all origins  
**POST** `/origins` - Create origin: `{"name": "Nicaragua"}`  
**GET** `/origins/:id` - Get specific origin  
**PUT** `/origins/:id` - Update origin  
**DELETE** `/origins/:id` - Delete origin

#### Strengths

**GET** `/strengths` - List all strengths  
**POST** `/strengths` - Create strength: `{"name": "Medium"}`  
**GET** `/strengths/:id` - Get specific strength  
**PUT** `/strengths/:id` - Update strength  
**DELETE** `/strengths/:id` - Delete strength

#### Ring Gauges

**GET** `/ring-gauges` - List all ring gauges  
**POST** `/ring-gauges` - Create gauge: `{"gauge": 50}`  
**GET** `/ring-gauges/:id` - Get specific gauge  
**PUT** `/ring-gauges/:id` - Update gauge  
**DELETE** `/ring-gauges/:id` - Delete gauge

---

### Admin Endpoints

**Requires Admin Authentication**

#### List Users

**GET** `/admin/users`

List all users in the system.

**Query Parameters**:
- `page` (optional)
- `page_size` (optional)

**Response**: `200 OK`
```json
{
  "users": [
    {
      "id": "uuid",
      "username": "user123",
      "email": "user@example.com",
      "full_name": "John Doe",
      "is_admin": false,
      "is_active": true,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 10,
  "page": 1,
  "page_size": 20
}
```

#### Create User

**POST** `/admin/users`

Create a new user account.

**Request Body**:
```json
{
  "username": "newuser",
  "email": "new@example.com",
  "full_name": "New User",
  "password": "initialPassword123",
  "is_admin": false,
  "is_active": true
}
```

**Response**: `201 Created`
```json
{
  "id": "uuid",
  "username": "newuser",
  "message": "User created successfully"
}
```

#### Get User

**GET** `/admin/users/:id`

Get a specific user's details.

**Response**: `200 OK`
```json
{
  "id": "uuid",
  "username": "user123",
  "email": "user@example.com",
  "full_name": "John Doe",
  "is_admin": false,
  "is_active": true,
  "created_at": "2025-01-01T00:00:00Z"
}
```

#### Update User

**PUT** `/admin/users/:id`

Update user account details.

**Request Body**:
```json
{
  "email": "newemail@example.com",
  "full_name": "Updated Name",
  "is_admin": false,
  "is_active": true
}
```

**Response**: `200 OK`
```json
{
  "message": "User updated successfully"
}
```

#### Reset User Password

**PATCH** `/admin/users/:id/password`

Reset a user's password (admin bypass).

**Request Body**:
```json
{
  "new_password": "newPassword123"
}
```

**Response**: `200 OK`
```json
{
  "message": "Password reset successfully"
}
```

#### Delete User

**DELETE** `/admin/users/:id`

Deactivate a user account (soft delete).

**Response**: `200 OK`
```json
{
  "message": "User deactivated successfully"
}
```

---

## Rate Limiting

Currently not implemented. Future versions will include:
- Login attempts: 5 per 15 minutes per IP
- API requests: 100 per minute per authenticated user
- Password resets: 3 per hour per email

## Pagination

List endpoints support pagination:

**Query Parameters**:
- `page`: Page number (1-indexed, default: 1)
- `page_size`: Items per page (default: 20, max: 100)

**Response includes**:
```json
{
  "items": [...],
  "total": 150,
  "page": 1,
  "page_size": 20,
  "total_pages": 8
}
```

## Webhooks

Not currently supported. Feature request welcome!

## SDK / Client Libraries

Official client libraries are not yet available. Contributions welcome!

**Community Libraries**:
- None yet - be the first!

## Related Documentation

- [User Guide](USER_GUIDE.md)
- [Authentication & Security](SECURITY_MODEL.md)
- [Admin Guide](ADMIN_GUIDE.md)
- [Humidor Sharing](SHARING.md)
