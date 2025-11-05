// Toast notification function (shared utility)
function showToast(message, type = 'info') {
    const container = document.getElementById('toastContainer');
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('toast-show');
    }, 100);

    setTimeout(() => {
        toast.classList.remove('toast-show');
        setTimeout(() => toast.remove(), 300);
    }, 5000);
}

// Get token from URL
const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');

// Check if token exists
if (!token) {
    document.getElementById('resetPasswordForm').style.display = 'none';
    document.getElementById('errorMessage').style.display = 'block';
    document.getElementById('errorText').textContent = 'No reset token provided';
}

// Form validation
const passwordInput = document.getElementById('password');
const confirmPasswordInput = document.getElementById('confirmPassword');
const passwordError = document.getElementById('passwordError');
const confirmPasswordError = document.getElementById('confirmPasswordError');

passwordInput.addEventListener('input', () => {
    const password = passwordInput.value;
    if (password.length > 0 && password.length < 8) {
        passwordError.textContent = 'Password must be at least 8 characters';
    } else {
        passwordError.textContent = '';
    }
    
    // Check if passwords match when both are filled
    if (confirmPasswordInput.value.length > 0) {
        if (password !== confirmPasswordInput.value) {
            confirmPasswordError.textContent = 'Passwords do not match';
        } else {
            confirmPasswordError.textContent = '';
        }
    }
});

confirmPasswordInput.addEventListener('input', () => {
    const password = passwordInput.value;
    const confirmPassword = confirmPasswordInput.value;
    
    if (confirmPassword.length > 0 && password !== confirmPassword) {
        confirmPasswordError.textContent = 'Passwords do not match';
    } else {
        confirmPasswordError.textContent = '';
    }
});

// Form submission handler
document.getElementById('resetPasswordForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const password = passwordInput.value;
    const confirmPassword = confirmPasswordInput.value;
    const resetBtn = document.getElementById('resetBtn');
    const form = document.getElementById('resetPasswordForm');
    const successMessage = document.getElementById('successMessage');
    const errorMessage = document.getElementById('errorMessage');
    
    // Validate passwords match
    if (password !== confirmPassword) {
        confirmPasswordError.textContent = 'Passwords do not match';
        return;
    }
    
    // Validate password length
    if (password.length < 8) {
        passwordError.textContent = 'Password must be at least 8 characters';
        return;
    }
    
    // Disable button during submission
    resetBtn.disabled = true;
    resetBtn.textContent = 'Resetting...';
    
    try {
        const response = await fetch('/api/v1/auth/reset-password', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ 
                token: token,
                password: password 
            }),
        });
        
        const data = await response.json();
        
        if (response.ok) {
            // Hide form and show success message
            form.style.display = 'none';
            successMessage.style.display = 'block';
            showToast('Password reset successfully!', 'success');
        } else {
            // Show error message
            form.style.display = 'none';
            errorMessage.style.display = 'block';
            document.getElementById('errorText').textContent = data.error || 'Failed to reset password';
            showToast(data.error || 'Failed to reset password', 'error');
        }
    } catch (error) {
        console.error('Error:', error);
        form.style.display = 'none';
        errorMessage.style.display = 'block';
        document.getElementById('errorText').textContent = 'Network error. Please try again.';
        showToast('Network error. Please try again.', 'error');
    }
});
