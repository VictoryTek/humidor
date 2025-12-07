// Profile Page JavaScript

let currentUser = null;
let authToken = null;

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    checkAuth();
    initializeEventListeners();
    initializeSidebarHandlers();
    initializeMobileMenu();
    loadUserProfile();
});

// Mobile Menu Management
function initializeMobileMenu() {
    const mobileMenuToggle = document.getElementById('mobileMenuToggle');
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!mobileMenuToggle || !sidebar || !backdrop) {
        console.log('Mobile menu elements not found');
        return;
    }
    
    // Toggle sidebar on button click
    mobileMenuToggle.addEventListener('click', () => {
        toggleMobileMenu();
    });
    
    // Close sidebar on backdrop click
    backdrop.addEventListener('click', () => {
        closeMobileMenu();
    });
    
    // Close sidebar when navigation item is clicked
    const navItems = sidebar.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.addEventListener('click', () => {
            // Small delay to allow navigation to complete
            setTimeout(() => {
                closeMobileMenu();
            }, 100);
        });
    });
    
    // Close sidebar on window resize if going above mobile breakpoint
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            if (window.innerWidth > 1024) {
                closeMobileMenu();
            }
        }, 250);
    });
}

function toggleMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    const isOpen = sidebar.classList.contains('open');
    
    if (isOpen) {
        closeMobileMenu();
    } else {
        openMobileMenu();
    }
}

function openMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    sidebar.classList.add('open');
    backdrop.classList.add('show');
    
    // Prevent body scroll when menu is open
    document.body.style.overflow = 'hidden';
}

function closeMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    sidebar.classList.remove('open');
    backdrop.classList.remove('show');
    
    // Restore body scroll
    document.body.style.overflow = '';
}

// Authentication Functions
function checkAuth() {
    authToken = localStorage.getItem('humidor_token');
    if (!authToken) {
        window.location.href = '/setup.html';
        return;
    }
}

function logout() {
    localStorage.removeItem('humidor_token');
    localStorage.removeItem('humidor_user');
    window.location.href = '/setup.html';
}

// Helper function to get user initials
function getInitials(name) {
    if (!name) return '?';
    const parts = name.split(' ');
    if (parts.length >= 2) {
        return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }
    return name.substring(0, 2).toUpperCase();
}

// API Request Helper
async function makeAuthenticatedRequest(url, options = {}) {
    const token = localStorage.getItem('humidor_token');
    
    const defaultOptions = {
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        }
    };
    
    const mergedOptions = {
        ...defaultOptions,
        ...options,
        headers: {
            ...defaultOptions.headers,
            ...options.headers
        }
    };
    
    try {
        const response = await fetch(url, mergedOptions);
        
        if (response.status === 401) {
            // Token expired or invalid
            localStorage.removeItem('humidor_token');
            localStorage.removeItem('humidor_user');
            window.location.href = '/setup.html';
            return null;
        }
        
        return response;
    } catch (error) {
        console.error('Request failed:', error);
        throw error;
    }
}

// Toast Notification
function showToast(message, type = 'info') {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.className = `toast toast-${type} show`;
    
    setTimeout(() => {
        toast.className = 'toast';
    }, 3000);
}

// Initialize Event Listeners
function initializeEventListeners() {
    // User dropdown toggle
    const userMenuTrigger = document.getElementById('userMenuTrigger');
    const userDropdownMenu = document.getElementById('userDropdownMenu');
    
    userMenuTrigger.addEventListener('click', (e) => {
        e.stopPropagation();
        userDropdownMenu.classList.toggle('show');
    });
    
    // Close dropdown when clicking outside
    document.addEventListener('click', () => {
        userDropdownMenu.classList.remove('show');
    });
    
    // Logout
    document.getElementById('logoutBtn').addEventListener('click', (e) => {
        e.preventDefault();
        logout();
    });
    
    // Save Profile button
    document.getElementById('saveProfileBtn').addEventListener('click', saveProfile);
    
    // Change Password button
    document.getElementById('changePasswordBtn').addEventListener('click', changePassword);
}

// Initialize Sidebar Handlers
function initializeSidebarHandlers() {
    // Organizers dropdown toggle
    const organizersToggle = document.getElementById('organizersToggle');
    const organizersDropdown = document.getElementById('organizersDropdown');
    
    if (organizersToggle && organizersDropdown) {
        organizersToggle.addEventListener('click', (e) => {
            e.preventDefault();
            organizersDropdown.classList.toggle('show');
            organizersToggle.classList.toggle('active');
        });
    }
    
    // Add Cigar button - redirect to home
    const addCigarBtn = document.getElementById('addCigarBtnNav');
    if (addCigarBtn) {
        addCigarBtn.addEventListener('click', () => {
            window.location.href = '/';
        });
    }
}

// Load User Profile
async function loadUserProfile() {
    try {
        const response = await makeAuthenticatedRequest('/api/v1/users/self');
        
        if (response && response.ok) {
            currentUser = await response.json();
            displayUserProfile(currentUser);
        } else {
            showToast('Failed to load user profile', 'error');
        }
    } catch (error) {
        console.error('Error loading profile:', error);
        showToast('Error loading profile', 'error');
    }
}

// Display User Profile
function displayUserProfile(user) {
    // Update header button
    document.getElementById('userName').textContent = user.username;
    const initials = user.full_name ? getInitials(user.full_name) : getInitials(user.username);
    document.getElementById('userAvatar').textContent = initials;
    
    // Update dropdown header
    document.getElementById('userDropdownAvatar').textContent = initials;
    document.getElementById('userDropdownName').textContent = user.full_name || user.username;
    document.getElementById('userDropdownUsername').textContent = '@' + user.username;
    
    // Fill form fields
    document.getElementById('profileUsername').value = user.username || '';
    document.getElementById('profileEmail').value = user.email || '';
    document.getElementById('profileFullName').value = user.full_name || '';
}

// Save Profile
async function saveProfile() {
    const username = document.getElementById('profileUsername').value.trim();
    const email = document.getElementById('profileEmail').value.trim();
    const fullName = document.getElementById('profileFullName').value.trim();
    
    // Validation
    if (!username) {
        showToast('Username is required', 'error');
        return;
    }
    
    if (!email) {
        showToast('Email is required', 'error');
        return;
    }
    
    if (!email.includes('@')) {
        showToast('Please enter a valid email address', 'error');
        return;
    }
    
    const updateData = {
        username,
        email,
        full_name: fullName || null
    };
    
    try {
        const saveBtn = document.getElementById('saveProfileBtn');
        saveBtn.disabled = true;
        saveBtn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Saving...';
        
        const response = await makeAuthenticatedRequest('/api/v1/users/self', {
            method: 'PUT',
            body: JSON.stringify(updateData)
        });
        
        if (response && response.ok) {
            const updatedUser = await response.json();
            currentUser = updatedUser;
            displayUserProfile(updatedUser);
            showToast('Profile updated successfully', 'success');
        } else {
            const error = await response.json();
            showToast(error.error || 'Failed to update profile', 'error');
        }
    } catch (error) {
        console.error('Error updating profile:', error);
        showToast('Error updating profile', 'error');
    } finally {
        const saveBtn = document.getElementById('saveProfileBtn');
        saveBtn.disabled = false;
        saveBtn.innerHTML = '<i class="mdi mdi-content-save"></i> Save Profile';
    }
}

// Change Password
async function changePassword() {
    const currentPassword = document.getElementById('currentPassword').value;
    const newPassword = document.getElementById('newPassword').value;
    const confirmPassword = document.getElementById('confirmPassword').value;
    
    // Validation
    if (!currentPassword) {
        showToast('Current password is required', 'error');
        return;
    }
    
    if (!newPassword) {
        showToast('New password is required', 'error');
        return;
    }
    
    if (newPassword.length < 8) {
        showToast('New password must be at least 8 characters', 'error');
        return;
    }
    
    if (newPassword !== confirmPassword) {
        showToast('New passwords do not match', 'error');
        return;
    }
    
    const passwordData = {
        current_password: currentPassword,
        new_password: newPassword
    };
    
    try {
        const changeBtn = document.getElementById('changePasswordBtn');
        changeBtn.disabled = true;
        changeBtn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Changing...';
        
        const response = await makeAuthenticatedRequest('/api/v1/users/password', {
            method: 'PUT',
            body: JSON.stringify(passwordData)
        });
        
        if (response && response.ok) {
            const result = await response.json();
            showToast(result.message || 'Password changed successfully', 'success');
            
            // Clear password fields
            document.getElementById('currentPassword').value = '';
            document.getElementById('newPassword').value = '';
            document.getElementById('confirmPassword').value = '';
        } else {
            const error = await response.json();
            showToast(error.error || 'Failed to change password', 'error');
        }
    } catch (error) {
        console.error('Error changing password:', error);
        showToast('Error changing password', 'error');
    } finally {
        const changeBtn = document.getElementById('changePasswordBtn');
        changeBtn.disabled = false;
        changeBtn.innerHTML = '<i class="mdi mdi-lock-check"></i> Change Password';
    }
}