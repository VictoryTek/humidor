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

// Check if email is configured on page load
async function checkEmailConfig() {
    try {
        const response = await fetch('/api/v1/auth/email-config');
        const data = await response.json();
        
        if (!data.email_configured) {
            // Show warning banner
            const warningBanner = document.createElement('div');
            warningBanner.style.cssText = `
                background: linear-gradient(135deg, #ff6b6b 0%, #ee5a6f 100%);
                color: white;
                padding: 12px 20px;
                border-radius: 8px;
                margin-bottom: 20px;
                font-size: 14px;
                line-height: 1.5;
                box-shadow: 0 2px 8px rgba(238, 90, 111, 0.3);
            `;
            warningBanner.innerHTML = `
                <strong>⚠️ Email Not Configured</strong><br>
                SMTP settings are not configured. Reset links will be logged to the server console instead of being sent via email.
            `;
            
            const form = document.getElementById('forgotPasswordForm');
            form.parentNode.insertBefore(warningBanner, form);
        }
    } catch (error) {
        console.error('Failed to check email configuration:', error);
    }
}

// Check email config when page loads
checkEmailConfig();

// Form submission handler
document.getElementById('forgotPasswordForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const email = document.getElementById('email').value.trim();
    const resetBtn = document.getElementById('resetBtn');
    const form = document.getElementById('forgotPasswordForm');
    const successMessage = document.getElementById('successMessage');
    
    // Disable button during submission
    resetBtn.disabled = true;
    resetBtn.textContent = 'Sending...';
    
    try {
        const response = await fetch('/api/v1/auth/forgot-password', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ email }),
        });
        
        const data = await response.json();
        
        if (response.ok) {
            // Hide form and show success message
            form.style.display = 'none';
            successMessage.style.display = 'block';
            showToast('Reset link sent! Check your email.', 'success');
        } else {
            showToast(data.error || 'Failed to send reset link', 'error');
            resetBtn.disabled = false;
            resetBtn.textContent = 'Send Reset Link';
        }
    } catch (error) {
        console.error('Error:', error);
        showToast('Network error. Please try again.', 'error');
        resetBtn.disabled = false;
        resetBtn.textContent = 'Send Reset Link';
    }
});

// Input validation
document.getElementById('email').addEventListener('input', (e) => {
    const emailError = document.getElementById('emailError');
    const email = e.target.value.trim();
    
    if (email && !email.match(/^[^\s@]+@[^\s@]+\.[^\s@]+$/)) {
        emailError.textContent = 'Please enter a valid email address';
    } else {
        emailError.textContent = '';
    }
});
