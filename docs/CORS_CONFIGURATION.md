# CORS Configuration Guide

## Overview

The Humidor application uses Cross-Origin Resource Sharing (CORS) to control which client origins can access the API. The application supports **two CORS modes** to balance security with ease of use:

- **Permissive Mode** (default): Allows access from any IP/domain - perfect for self-hosted environments with dynamic IPs
- **Strict Mode**: Only allows explicitly listed origins - recommended for production with fixed domains

## Quick Start

### Self-Hosted / Dynamic IPs (Recommended Default)

Use **permissive mode** - works with any IP address (Tailscale, LAN, localhost):

```env
CORS_MODE=permissive
```

✅ Access from localhost  
✅ Access from LAN IP (192.168.x.x)  
✅ Access from Tailscale (100.x.x.x)  
✅ Access from any VPN IP  
✅ IP changes don't break access  

**No configuration needed** - just set `CORS_MODE=permissive` (or use the default).

### Production with Fixed Domain

Use **strict mode** with explicit domain list:

```env
CORS_MODE=strict
ALLOWED_ORIGINS=https://humidor.example.com,https://www.humidor.example.com
```

✅ Maximum security  
✅ Only your domains can access API  
❌ Must update config if domains change  

## Understanding the Error

**Error Message**: `CORS request forbidden: origin not allowed`

**What it means**: The client is trying to access the API from an origin (protocol + domain + port) that is not allowed by the current CORS configuration.

## Common Scenarios

### Scenario 1: Accessing via Tailscale or VPN IP
**Problem**: Accessing `http://100.93.238.62:9898` but server only allows `localhost`  
**Solution**: Add your Tailscale/VPN IP to `ALLOWED_ORIGINS`

### Scenario 2: Docker Deployment on Remote Server
**Problem**: Accessing `http://192.168.1.100:9898` but defaults don't include LAN IPs  
**Solution**: Configure `ALLOWED_ORIGINS` with your server's IP

### Scenario 3: Multiple Access Points
**Problem**: Need to access from localhost AND LAN IP AND domain name  
**Solution**: List all origins comma-separated

## Configuration Methods

### Method 1: Permissive Mode (Recommended for Self-Hosted)

**Best for**: Dynamic IPs, Tailscale, home networks, development

```bash
# Just set the mode - no origin list needed
export CORS_MODE=permissive
```

Or in Docker:
```yaml
services:
  humidor:
    environment:
      CORS_MODE: permissive
```

That's it! Access from any IP address without further configuration.

### Method 2: Strict Mode (Recommended for Production)

**Best for**: Fixed domain names, production deployments

```bash
# Enable strict mode and list allowed domains
export CORS_MODE=strict
export ALLOWED_ORIGINS="https://humidor.example.com,https://www.humidor.example.com"
```

Or in Docker:
```yaml
services:
  humidor:
    environment:
      CORS_MODE: strict
      ALLOWED_ORIGINS: "https://humidor.example.com,https://www.humidor.example.com"
```

## Configuration Examples

### Self-Hosted (Home Server, Tailscale, NAS)
```env
# Works with any IP - no need to list them
CORS_MODE=permissive
```

**Why this works**: Reflects back the Origin header, so whatever IP/domain you use to access the app is automatically allowed.

### Development
```env
# Allow any origin for easy testing
CORS_MODE=permissive
```

### Production with Domain
```env
# Lock down to specific domain
CORS_MODE=strict
ALLOWED_ORIGINS=https://humidor.example.com
```

### Production with Multiple Domains
```env
CORS_MODE=strict
ALLOWED_ORIGINS=https://humidor.example.com,https://www.humidor.example.com,https://app.humidor.example.com
```

## Validation Rules

The server validates all CORS origins with the following rules:

1. **Must start with `http://` or `https://`**
   - ✅ Valid: `http://localhost:9898`
   - ❌ Invalid: `localhost:9898`

## Validation Rules

### Permissive Mode
- No validation needed
- Automatically allows any origin
- Perfect for self-hosted setups

### Strict Mode
The server validates all CORS origins with the following rules:

1. **Must start with `http://` or `https://`**
   - ✅ Valid: `http://localhost:9898`
   - ❌ Invalid: `localhost:9898`
## Troubleshooting

### Check Current Configuration

View the server logs at startup:

```bash
docker compose logs humidor | grep -i cors
```

**Permissive mode output:**
```
CORS mode set to 'permissive' - all origins are allowed
```

**Strict mode output:**
```
CORS configuration loaded and validated mode="strict" allowed_origins=["https://humidor.example.com"]
```

### Common Issues

#### 1. Still Getting CORS Error with Permissive Mode
**Symptom**: 403 error even with `CORS_MODE=permissive`  
**Fix**: Restart containers to pick up new configuration

```bash
docker compose down
docker compose up -d
docker compose logs humidor | grep -i cors  # Verify mode
```

#### 2. IP Address Changed (When Using Strict Mode)
**Symptom**: CORS error after network change  
**Fix**: Switch to permissive mode instead

```env
# Instead of maintaining IP lists:
CORS_MODE=permissive

# Not needed anymore:
# ALLOWED_ORIGINS=http://192.168.1.100:9898,http://100.93.238.62:9898,...
```

#### 3. Using Wrong Protocol
**Symptom**: 403 error in strict mode  
**Fix**: Match the protocol exactly (`http` vs `https`)

```bash
# If accessing via http://example.com:9898
ALLOWED_ORIGINS=http://example.com:9898  # Correct

# NOT
ALLOWED_ORIGINS=https://example.com:9898  # Wrong protocol
```
# Wrong (missing http://)
ALLOWED_ORIGINS=localhost:9898,192.168.1.100:9898
```

## Security Considerations

### Development vs Production

## Security Considerations

### Permissive vs Strict Mode

**Permissive Mode** (`CORS_MODE=permissive`):
- ✅ Works with any IP/domain automatically
- ✅ Perfect for self-hosted, home networks, Tailscale
- ✅ No maintenance when IPs change
- ⚠️ Less restrictive, but still requires valid requests
- ⚠️ Consider using strict mode for internet-facing deployments

**Strict Mode** (`CORS_MODE=strict`):
- ✅ Maximum security - explicit allow list
- ✅ Best for production with fixed domains
- ❌ Requires maintenance if IPs/domains change
- ❌ Can break access if configuration is wrong

### When to Use Each Mode

| Scenario | Recommended Mode | Reason |
|----------|-----------------|--------|
| Self-hosted (home/office) | Permissive | IPs may change |
| Tailscale/VPN | Permissive | Dynamic IPs |
| Development | Permissive | Convenience |
| Production (public internet) | Strict | Maximum security |
| Fixed domain name | Strict | Explicit control |
| Docker on NAS | Permissive | May access via multiple IPs |

### Best Practices

1. **Self-Hosted Environments**
   ```env
   # Use permissive - it's secure enough and maintenance-free
   CORS_MODE=permissive
   ```

2. **Production Internet-Facing**
   ```env
   # Use strict with HTTPS
   CORS_MODE=strict
## Dynamic IP Scenarios

### Problem: IP Changes Frequently

**Solution: Use Permissive Mode** (Recommended)

```env
CORS_MODE=permissive
```

This automatically works with any IP - no configuration updates needed when:
- DHCP assigns new IP
- Tailscale IP changes
- Accessing from different networks
- Using multiple devices/IPs

### Alternative: Use Strict Mode with DNS

If you prefer strict mode for added security:

## Applying Configuration Changes

### Docker Compose
```bash
# Edit .env or docker-compose.yml
# Set: CORS_MODE=permissive (for self-hosted)
# Or: CORS_MODE=strict with ALLOWED_ORIGINS (for production)

# Restart services
docker compose down
docker compose up -d

# Verify
docker compose logs humidor | grep -i cors
```

### Running Container
```bash
# Stop container
docker stop humidor

# Start with permissive mode
docker run -d \
  -e CORS_MODE=permissive \
  -p 9898:9898 \
  ghcr.io/victorytek/humidor:latest

# Or strict mode
docker run -d \
  -e CORS_MODE=strict \
  -e ALLOWED_ORIGINS="https://humidor.example.com" \
  -p 9898:9898 \
  ghcr.io/victorytek/humidor:latest

# Verify logs
docker logs humidor 2>&1 | grep -i cors
```
# Edit .env or docker-compose.yml to add your IP to ALLOWED_ORIGINS

# Restart services
docker compose up -d

# Verify
docker compose logs humidor | grep -i cors
```

### Running Container
```bash
# Stop container
docker stop humidor

# Start with new origin
docker run -d \
  -e ALLOWED_ORIGINS="http://localhost:9898,http://100.93.238.62:9898" \
  -p 9898:9898 \
  ghcr.io/victorytek/humidor:latest

# Verify logs
docker logs humidor 2>&1 | grep -i cors
```

## Testing CORS Configuration
## Quick Fix Reference

| Access Method | Configuration |
|--------------|---------------|
| Any IP (self-hosted) | `CORS_MODE=permissive` |
| Tailscale (any IP) | `CORS_MODE=permissive` |
| LAN (dynamic DHCP) | `CORS_MODE=permissive` |
| Fixed domain | `CORS_MODE=strict` + `ALLOWED_ORIGINS=https://domain.com` |
| Multiple domains | `CORS_MODE=strict` + `ALLOWED_ORIGINS=https://d1.com,https://d2.com` |
| Development | `CORS_MODE=permissive` |

# Look for:
# < Access-Control-Allow-Origin: http://100.93.238.62:9898
```

### Using Browser DevTools
1. Open browser DevTools (F12)
2. Go to Network tab
3. Attempt the request
4. Look for:
   - Request Headers: `Origin: http://100.93.238.62:9898`
   - Response Headers: `Access-Control-Allow-Origin: http://100.93.238.62:9898`

### Success Indicators
✅ Response header includes `Access-Control-Allow-Origin` matching your origin  
✅ Status code is NOT 403  
✅ Server logs show your origin in the allowed list

### Failure Indicators
❌ 403 Forbidden status  
❌ Error message: "CORS request forbidden: origin not allowed"  
❌ Missing `Access-Control-Allow-Origin` header in response  
❌ Server logs show empty or different allowed origins

## Quick Fix Reference

| Access Method | ALLOWED_ORIGINS Value |
|--------------|----------------------|
| localhost | `http://localhost:9898` |
| 127.0.0.1 | `http://127.0.0.1:9898` |
| LAN IP | `http://192.168.1.100:9898` |
| Tailscale | `http://100.x.x.x:9898` |
| Domain | `https://humidor.example.com` |
| All (dev only) | `*` |

## Related Configuration

### JWT Secret
CORS works with JWT authentication. Ensure `JWT_SECRET` is also configured:
```env
JWT_SECRET=your-secret-here-min-32-chars
```

### BASE_URL
Should match your primary access method:
```env
BASE_URL=http://100.93.238.62:9898
ALLOWED_ORIGINS=http://100.93.238.62:9898,http://localhost:9898
```

## Further Reading

- [MDN: CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [OWASP: CORS](https://owasp.org/www-community/attacks/CORS_OriginHeaderScrutiny)
- [Security Model Documentation](./SECURITY_MODEL.md)
