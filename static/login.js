// Login JavaScript
document.addEventListener('DOMContentLoaded', function() {
    const loginForm = document.getElementById('loginForm');
    const loginBtn = document.getElementById('loginBtn');
    const usernameField = document.querySelector('input[name="login"]');
    const passwordField = document.getElementById('password');
    const toastContainer = document.getElementById('toastContainer');

    // Check if user is already logged in
    checkExistingAuth();

    loginForm.addEventListener('submit', handleLogin);

    async function handleLogin(event) {
        event.preventDefault();
        
        const username = usernameField.value.trim();
        const password = passwordField.value;

        if (!username || !password) {
            showToast('Please enter both username and password', 'error');
            return;
        }

        loginBtn.disabled = true;
        loginBtn.textContent = 'Signing In...';

        try {
            const response = await fetch('/api/v1/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    username: username,
                    password: password
                })
            });

            if (response.ok) {
                const data = await response.json();
                
                // Store authentication data
                localStorage.setItem('humidor_token', data.token);
                localStorage.setItem('humidor_user', JSON.stringify(data.user));
                
                showToast('Login successful! Redirecting...', 'success');
                
                // Redirect to main app
                setTimeout(() => {
                    window.location.href = '/';
                }, 1500);
                
            } else {
                const errorData = await response.text();
                let errorMessage = 'Login failed';
                
                try {
                    const parsedError = JSON.parse(errorData);
                    errorMessage = parsedError.error || errorMessage;
                } catch (parseError) {
                    errorMessage = `Login failed (${response.status})`;
                }
                
                showToast(errorMessage, 'error');
            }
            
        } catch (error) {
            console.error('Login error:', error);
            showToast('Network error. Please try again.', 'error');
        } finally {
            loginBtn.disabled = false;
            loginBtn.textContent = 'Sign In';
        }
    }

    function checkExistingAuth() {
        const token = localStorage.getItem('humidor_token');
        const user = localStorage.getItem('humidor_user');
        
        if (token && user) {
            // User is already logged in, redirect to main app
            window.location.href = '/';
        }
    }

    function showToast(message, type = 'info') {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.textContent = message;
        
        toastContainer.appendChild(toast);
        
        // Show toast
        setTimeout(() => {
            toast.classList.add('show');
        }, 100);
        
        // Hide and remove toast
        setTimeout(() => {
            toast.classList.remove('show');
            setTimeout(() => {
                if (toastContainer.contains(toast)) {
                    toastContainer.removeChild(toast);
                }
            }, 300);
        }, 5000);
    }
});