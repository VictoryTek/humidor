# PWA Testing Guide

## Quick Test: Is PWA Working?

### 1. Check Service Worker Registration
Open browser console (F12) and look for:
```
[PWA] Service Worker registered successfully: /
```

If you see this message, the service worker is active!

### 2. Test Install Button (Chrome/Edge)
1. Open the app in Chrome or Edge
2. Look for the **"Install App"** button in the header (top right)
3. If it appears, PWA is installable!
4. Click it to see the native install prompt

### 3. Test Offline Mode
1. Open Chrome DevTools (F12)
2. Go to **Application** tab → **Service Workers**
3. Check the **Offline** checkbox
4. Refresh the page
5. You should see the custom offline page with auto-retry

### 4. Validate Manifest
1. Open Chrome DevTools (F12)
2. Go to **Application** tab → **Manifest**
3. Check for:
   - Name: "Humidor - Premium Cigar Collection"
   - 4 icons displayed
   - Theme color: #D4AF37
   - No errors or warnings

## Full PWA Testing Checklist

### Desktop Browser (Chrome/Edge)
- [ ] Service worker registers successfully (check console)
- [ ] Install button appears in header
- [ ] Clicking install button shows native prompt
- [ ] After installing, app opens in standalone window
- [ ] App icon appears on desktop/taskbar
- [ ] Offline mode works (shows offline.html)
- [ ] Cached pages load instantly
- [ ] Update notification appears when service worker changes

### Mobile Browser (Chrome/Edge on Android)
- [ ] Install banner appears automatically after engagement
- [ ] Can manually trigger install from browser menu (Add to Home Screen)
- [ ] Installed app icon appears on home screen
- [ ] Tapping icon opens app in standalone mode
- [ ] Status bar color matches theme (#D4AF37)
- [ ] Splash screen shows app icon and name
- [ ] Offline mode works correctly
- [ ] App feels native (no browser UI)

### iOS Safari
- [ ] Service worker registers (iOS 11.3+)
- [ ] Can add to home screen (Share → Add to Home Screen)
- [ ] Home screen icon uses apple-touch-icon
- [ ] App opens in safari-friendly mode
- [ ] Offline caching works
- [ ] Note: No automatic install prompt (Apple doesn't support it)

### DevTools Checks

#### Application Tab
```
Service Workers:
  ✅ Status: Activated and Running
  ✅ Version: humidor-v1.2.0
  ✅ Scope: /
  
Manifest:
  ✅ No errors
  ✅ 4 icons present
  ✅ Theme color set
  
Cache Storage:
  ✅ humidor-v1.2.0-static (18 items)
  ✅ humidor-v1.2.0-dynamic
  ✅ humidor-v1.2.0-images
```

#### Network Tab
- First load: Mix of network and (from ServiceWorker)
- Subsequent loads: Mostly (from ServiceWorker)
- Offline: All (from ServiceWorker)

### Lighthouse Audit

Run Lighthouse PWA audit:
1. Open Chrome DevTools
2. Go to **Lighthouse** tab
3. Select **Progressive Web App** category
4. Click **Analyze page load**

**Expected Results:**
- ✅ Installable
- ✅ PWA Optimized
- ✅ Fast and reliable offline
- ✅ 90+ score

## Common Issues & Solutions

### Install Button Not Showing
**Problem**: beforeinstallprompt event not firing  
**Possible Causes**:
- App already installed
- Using Firefox (doesn't support install prompt)
- Not served over HTTPS (required for PWA)
- Manifest has errors (check DevTools → Application → Manifest)

**Solution**:
1. Uninstall app if already installed
2. Check manifest for errors
3. Verify HTTPS connection
4. Try different browser (Chrome/Edge)

### Service Worker Not Updating
**Problem**: Changes to app.js not appearing  
**Cause**: Browser using cached service worker

**Solution**:
1. DevTools → Application → Service Workers
2. Click "Update" button
3. Or enable "Update on reload" during development
4. Hard refresh: Ctrl+Shift+R (Windows) or Cmd+Shift+R (Mac)

### Offline Mode Not Working
**Problem**: Offline page not appearing when offline  
**Possible Causes**:
- Service worker not activated yet
- offline.html not cached
- Cache version mismatch

**Solution**:
1. Check if service worker is "Activated and Running"
2. Verify offline.html in STATIC_ASSETS array
3. Clear all caches and reload
4. Check console for errors

### Icons Not Displaying
**Problem**: App icon is blank or default browser icon  
**Cause**: Icon paths incorrect or files missing

**Solution**:
1. Verify icons exist: `/static/icons/icon-*.png`
2. Check manifest.json icon paths
3. Use absolute paths: `/static/icons/icon-192x192.png`
4. Clear cache and reinstall app

## Manual Testing Steps

### Test 1: First Load & Caching
```bash
1. Open app in Chrome (fresh browser session)
2. Open DevTools → Network tab
3. Enable "Disable cache" checkbox
4. Reload page
5. Check Network tab - should see:
   - app.js loaded from network
   - Service worker registration
6. Disable "Disable cache"
7. Reload page again
8. Check Network tab - should see:
   - Most files from "(from ServiceWorker)"
   - Fast load time
```

### Test 2: Offline Functionality
```bash
1. Load app normally (ensure caching works)
2. Open DevTools → Application → Service Workers
3. Check "Offline" box
4. Navigate to different pages
5. Verify:
   - Cached pages load normally
   - Uncached pages show offline.html
   - Auto-retry attempts reconnection
6. Uncheck "Offline" box
7. Click "Retry" on offline page
8. Should navigate back to app
```

### Test 3: Install & Standalone Mode
```bash
1. Click "Install App" button in header
2. Confirm installation in prompt
3. App should:
   - Install to desktop/home screen
   - Open in new standalone window
   - Show app icon (not browser icon)
   - Remove browser chrome (no address bar)
4. Close and reopen app
5. Should open in standalone mode again
```

### Test 4: Update Flow
```bash
1. Make a change to app.js (add console.log)
2. Increment CACHE_VERSION in sw.js
3. Deploy changes
4. In already-open app tab:
5. Wait for hourly check OR manually trigger:
   - DevTools → Application → Service Workers
   - Click "Update"
6. Update banner should appear at top
7. Click "Update Now"
8. Page reloads with new version
9. Console should show new log message
```

## Performance Benchmarks

### Expected Load Times
```
First Load (no cache):
  - HTML: ~50-100ms
  - CSS: ~30-50ms
  - JavaScript: ~100-200ms
  - Total: ~500-1000ms

Subsequent Load (cached):
  - All assets: <50ms (from cache)
  - Total: ~100-200ms

Offline Load (cached):
  - Same as cached load
  - No network delays
```

### Cache Sizes
```
Static Cache: ~2-3 MB
  - HTML, CSS, JS, manifest
  - Icons and placeholders
  - Fonts

Dynamic Cache: Variable
  - API responses
  - Grows with usage

Image Cache: Variable
  - User-uploaded cigar images
  - Grows with usage
```

## Browser DevTools Commands

### Service Worker
```javascript
// Get service worker status
navigator.serviceWorker.getRegistrations()

// Unregister service worker
navigator.serviceWorker.getRegistrations().then(registrations => {
  registrations[0].unregister()
})

// Trigger update check
navigator.serviceWorker.getRegistrations().then(registrations => {
  registrations[0].update()
})
```

### Cache API
```javascript
// List all caches
caches.keys()

// Get items in a cache
caches.open('humidor-v1.2.0-static').then(cache => {
  cache.keys().then(keys => console.log(keys))
})

// Clear all caches
caches.keys().then(names => {
  names.forEach(name => caches.delete(name))
})
```

## CI/CD Testing

### Automated Tests
```bash
# Validate manifest
jq . static/manifest.json

# Check service worker syntax
node -c static/sw.js

# Verify icons exist
ls -lh static/icons/icon-*.png

# Lighthouse CI
lighthouse --only-categories=pwa https://your-domain.com
```

### Deployment Checklist
- [ ] Increment CACHE_VERSION in sw.js
- [ ] Update STATIC_ASSETS if files added/removed
- [ ] Run Lighthouse PWA audit
- [ ] Test install on mobile device
- [ ] Verify offline mode works
- [ ] Check update notification triggers

## Success Criteria

✅ PWA is working correctly when:
1. Service worker registers without errors
2. Install button appears on desktop browsers
3. App can be added to home screen
4. Offline mode shows custom offline page
5. Cached pages load instantly
6. Update notifications appear for new versions
7. Lighthouse PWA audit scores 90+
8. App feels native when installed

🎉 Congratulations! Your PWA is fully functional!
