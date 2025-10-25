// Setup Wizard JavaScript
let currentStep = 1;
const totalSteps = 5;

// DOM Elements
const elements = {
    progressFill: document.getElementById('progressFill'),
    prevBtn: document.getElementById('prevBtn'),
    nextBtn: document.getElementById('nextBtn'),
    
    // Forms
    accountForm: document.getElementById('accountForm'),
    humidorForm: document.getElementById('humidorForm'),
    
    // Form fields
    username: document.getElementById('username'),
    fullName: document.getElementById('fullName'),
    email: document.getElementById('email'),
    password: document.getElementById('password'),
    confirmPassword: document.getElementById('confirmPassword'),
    
    humidorName: document.getElementById('humidorName'),
    humidorDescription: document.getElementById('humidorDescription'),
    capacity: document.getElementById('capacity'),
    location: document.getElementById('location'),
    sampleData: document.getElementById('sampleData'),
    
    // Summary elements
    summaryUsername: document.getElementById('summaryUsername'),
    summaryFullName: document.getElementById('summaryFullName'),
    summaryEmail: document.getElementById('summaryEmail'),
    summaryHumidorName: document.getElementById('summaryHumidorName'),
    summaryCapacity: document.getElementById('summaryCapacity'),
    summarySampleData: document.getElementById('summarySampleData'),
    
    setupLoading: document.getElementById('setupLoading'),
    toastContainer: document.getElementById('toastContainer')
};

// Form data storage
let formData = {
    user: {},
    humidor: {}
};

// Initialize the setup wizard
document.addEventListener('DOMContentLoaded', function() {
    console.log('Setup wizard initializing...');
    console.log('Current step:', currentStep);
    
    updateProgressBar();
    updateNavigationButtons();
    updateSteps(); // Make sure steps are properly initialized
    
    // Add event listeners
    console.log('Adding event listeners...');
    console.log('Prev button element:', elements.prevBtn);
    console.log('Next button element:', elements.nextBtn);
    
    elements.prevBtn.addEventListener('click', goToPreviousStep);
    elements.nextBtn.addEventListener('click', handleNextButtonClick);
    
    console.log('Event listeners added successfully');
    
    // Form validation listeners
    elements.password.addEventListener('input', validatePasswords);
    elements.confirmPassword.addEventListener('input', validatePasswords);
    
    // Set default values
    elements.capacity.value = 50;
});

// Navigation functions
function goToNextStep() {
    console.log('goToNextStep called, currentStep:', currentStep, 'totalSteps:', totalSteps);
    if (currentStep <= totalSteps) {
        console.log('Step validation result:', validateCurrentStep());
        if (validateCurrentStep()) {
            if (currentStep === 2) {
                captureUserData();
            } else if (currentStep === 3) {
                captureHumidorData();
                // Update summary when moving to step 4
                updateSummary();
            } else if (currentStep === 4) {
                submitSetup();
                return; // Don't increment step yet, wait for submission
            }
            
            currentStep++;
            updateWizard();
        }
    }
}

function goToPreviousStep() {
    if (currentStep > 1) {
        currentStep--;
        updateWizard();
    }
}

function handleNextButtonClick(event) {
    console.log('=== BUTTON CLICKED ===');
    console.log('Event:', event);
    console.log('Next button clicked, currentStep:', currentStep);
    console.log('Button text:', elements.nextBtn.textContent);
    
    if (currentStep === 5) {
        console.log('On completion step, calling finishSetup');
        finishSetup();
    } else {
        console.log('On regular step, calling goToNextStep');
        goToNextStep();
    }
}

function finishSetup() {
    console.log('finishSetup function called!');
    console.log('Setup completed, redirecting to login...');
    
    try {
        console.log('Redirecting to login page...');
        window.location.href = '/static/login.html';
        console.log('Redirect command executed');
    } catch (error) {
        console.error('Error during redirect:', error);
        alert('Redirect failed: ' + error.message);
    }
}

function updateWizard() {
    updateProgressBar();
    updateSteps();
    updateNavigationButtons();
}

function updateProgressBar() {
    const progress = (currentStep / totalSteps) * 100;
    elements.progressFill.style.width = progress + '%';
}

function updateSteps() {
    console.log('Updating steps, current step:', currentStep);
    
    // Hide all steps
    document.querySelectorAll('.setup-step').forEach(step => {
        step.classList.remove('active');
        console.log('Hiding step:', step.id);
    });
    
    // Show current step
    const stepMap = {
        1: 'welcomeStep',
        2: 'accountStep',
        3: 'humidorStep',
        4: 'summaryStep',
        5: 'completeStep'
    };
    
    const currentStepElement = document.getElementById(stepMap[currentStep]);
    if (currentStepElement) {
        currentStepElement.classList.add('active');
        console.log('Showing step:', currentStepElement.id);
    } else {
        console.error('Could not find step element for step:', currentStep, stepMap[currentStep]);
    }
    
    // Update step indicators
    document.querySelectorAll('.step').forEach((step, index) => {
        step.classList.remove('active', 'completed');
        
        if (index + 1 === currentStep) {
            step.classList.add('active');
        } else if (index + 1 < currentStep) {
            step.classList.add('completed');
        }
    });
}

function updateNavigationButtons() {
    elements.prevBtn.disabled = currentStep === 1;
    
    // ALWAYS enable the next button
    elements.nextBtn.disabled = false;
    
    if (currentStep === 1) {
        elements.nextBtn.textContent = 'Get Started →';
    } else if (currentStep === 4) {
        elements.nextBtn.textContent = 'Create Account & Humidor';
    } else if (currentStep === 5) {
        elements.nextBtn.textContent = 'Enter Humidor →';
        console.log('=== STEP 5 BUTTON UPDATE ===');
        console.log('Button disabled?', elements.nextBtn.disabled);
        console.log('Button text:', elements.nextBtn.textContent);
        console.log('Current step:', currentStep);
    } else {
        elements.nextBtn.textContent = 'Next →';
    }
}

// Validation functions
function validateCurrentStep() {
    switch (currentStep) {
        case 1:
            return true; // Welcome step has no validation
        case 2:
            return validateUserForm();
        case 3:
            return validateHumidorForm();
        case 4:
            return true; // Summary step
        case 5:
            return true; // Complete step
        default:
            return false;
    }
}

function validateUserForm() {
    let isValid = true;
    
    // Clear previous errors
    clearErrors();
    
    // Username validation
    if (!elements.username.value.trim()) {
        showFieldError('usernameError', 'Username is required');
        isValid = false;
    } else if (elements.username.value.length < 3) {
        showFieldError('usernameError', 'Username must be at least 3 characters');
        isValid = false;
    }
    
    // Full name validation
    if (!elements.fullName.value.trim()) {
        showFieldError('fullNameError', 'Full name is required');
        isValid = false;
    }
    
    // Email validation
    if (!elements.email.value.trim()) {
        showFieldError('emailError', 'Email is required');
        isValid = false;
    } else if (!isValidEmail(elements.email.value)) {
        showFieldError('emailError', 'Please enter a valid email address');
        isValid = false;
    }
    
    // Password validation
    if (!elements.password.value) {
        showFieldError('passwordError', 'Password is required');
        isValid = false;
    } else if (elements.password.value.length < 8) {
        showFieldError('passwordError', 'Password must be at least 8 characters');
        isValid = false;
    }
    
    // Confirm password validation
    if (!elements.confirmPassword.value) {
        showFieldError('confirmPasswordError', 'Please confirm your password');
        isValid = false;
    } else if (elements.password.value !== elements.confirmPassword.value) {
        showFieldError('confirmPasswordError', 'Passwords do not match');
        isValid = false;
    }
    
    return isValid;
}

function validateHumidorForm() {
    let isValid = true;
    
    // Clear previous errors
    clearHumidorErrors();
    
    // Humidor name validation
    if (!elements.humidorName.value.trim()) {
        showFieldError('humidorNameError', 'Humidor name is required');
        isValid = false;
    }
    
    return isValid;
}

function validatePasswords() {
    const password = elements.password.value;
    const confirmPassword = elements.confirmPassword.value;
    
    if (password && confirmPassword && password !== confirmPassword) {
        showFieldError('confirmPasswordError', 'Passwords do not match');
    } else {
        clearFieldError('confirmPasswordError');
    }
}

function isValidEmail(email) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

// Error handling
function showFieldError(fieldId, message) {
    const errorElement = document.getElementById(fieldId);
    if (errorElement) {
        errorElement.textContent = message;
        errorElement.style.display = 'block';
    }
}

function clearFieldError(fieldId) {
    const errorElement = document.getElementById(fieldId);
    if (errorElement) {
        errorElement.textContent = '';
        errorElement.style.display = 'none';
    }
}

function clearErrors() {
    const errorFields = ['usernameError', 'fullNameError', 'emailError', 'passwordError', 'confirmPasswordError'];
    errorFields.forEach(fieldId => clearFieldError(fieldId));
}

function clearHumidorErrors() {
    clearFieldError('humidorNameError');
}

// Data capture functions
function captureUserData() {
    formData.user = {
        username: elements.username.value.trim(),
        full_name: elements.fullName.value.trim(),
        email: elements.email.value.trim(),
        password: elements.password.value
    };
    console.log('User data captured (password hidden for security)');
}

function captureHumidorData() {
    formData.humidor = {
        name: elements.humidorName.value.trim(),
        description: elements.humidorDescription.value.trim(),
        capacity: parseInt(elements.capacity.value) || null,
        location: elements.location.value.trim(),
        includeSampleData: elements.sampleData.checked
    };
    console.log('Captured humidor data:', formData.humidor);
}

function updateSummary() {
    console.log('Updating summary with data:', formData);
    
    if (formData.user) {
        elements.summaryUsername.textContent = formData.user.username || 'Not provided';
        elements.summaryFullName.textContent = formData.user.full_name || 'Not provided';
        elements.summaryEmail.textContent = formData.user.email || 'Not provided';
    }
    
    if (formData.humidor) {
        elements.summaryHumidorName.textContent = formData.humidor.name || 'Not provided';
        elements.summaryCapacity.textContent = formData.humidor.capacity ? formData.humidor.capacity + ' cigars' : 'Not specified';
        elements.summarySampleData.textContent = formData.humidor.includeSampleData ? 'Yes' : 'No';
    }
    
    console.log('Summary elements updated');
}

// API submission
async function submitSetup() {
    elements.setupLoading.classList.add('active');
    elements.nextBtn.disabled = true;
    
    try {
        // Create user account and first humidor in one request
        const setupData = {
            user: formData.user,
            humidor: formData.humidor
        };
        
        const userResponse = await fetch('/api/v1/setup/user', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(setupData)
        });
        
        if (!userResponse.ok) {
            console.error('User creation failed:', userResponse.status, userResponse.statusText);
            const responseText = await userResponse.text();
            console.error('Response text:', responseText);
            
            let errorMessage = 'Failed to create user account';
            try {
                const errorData = JSON.parse(responseText);
                errorMessage = errorData.message || errorData.error || errorMessage;
            } catch (parseError) {
                console.error('Error parsing error response:', parseError);
                errorMessage = `Server error (${userResponse.status}): ${responseText.substring(0, 100)}`;
            }
            throw new Error(errorMessage);
        }
        
        const userData = await userResponse.json();
        console.log('User data received:', userData);
        console.log('Include sample data?', formData.humidor.includeSampleData);
        console.log('Humidor ID:', userData.humidor_id);
        
        // If sample data is requested, add sample cigars
        if (formData.humidor.includeSampleData) {
            console.log('Calling addSampleData...');
            await addSampleData(userData.token, userData.humidor_id);
        } else {
            console.log('Sample data not requested');
        }
        
        // Setup successful, move to completion step
        elements.setupLoading.classList.remove('active');
        elements.nextBtn.disabled = false; // Explicitly re-enable the button
        currentStep++;
        updateWizard();
        
        console.log('=== SETUP COMPLETED ===');
        console.log('Button disabled after setup?', elements.nextBtn.disabled);
        console.log('Current step after setup:', currentStep);
        
        showToast('Setup completed successfully!', 'success');
        
    } catch (error) {
        console.error('Setup error:', error);
        elements.setupLoading.classList.remove('active');
        elements.nextBtn.disabled = false;
        
        showToast(error.message || 'Setup failed. Please try again.', 'error');
    }
}

async function addSampleData(token, humidorId) {
    console.log('=== ADDING SAMPLE DATA ===');
    console.log('Token:', token ? 'Present' : 'Missing');
    console.log('Humidor ID:', humidorId);
    
    const sampleCigars = [
        {
            brand: 'Montecristo',
            name: 'No. 2',
            size: 'Torpedo',
            ring_gauge: 52,
            length: 6.1,
            strength: 'Medium',
            origin: 'Cuba',
            wrapper: 'Natural',
            price: 15.99,
            quantity: 5,
            humidor_id: humidorId
        },
        {
            brand: 'Romeo y Julieta',
            name: 'Churchill',
            size: 'Churchill',
            ring_gauge: 47,
            length: 7,
            strength: 'Medium',
            origin: 'Cuba',
            wrapper: 'Natural',
            price: 12.50,
            quantity: 3,
            humidor_id: humidorId
        },
        {
            brand: 'Cohiba',
            name: 'Robusto',
            size: 'Robusto',
            ring_gauge: 50,
            length: 5,
            strength: 'Full',
            origin: 'Cuba',
            wrapper: 'Natural',
            price: 22.00,
            quantity: 2,
            humidor_id: humidorId
        },
        {
            brand: 'Arturo Fuente',
            name: 'Opus X Robusto',
            size: 'Robusto',
            ring_gauge: 50,
            length: 5.5,
            strength: 'Full',
            origin: 'Dominican Republic',
            wrapper: 'Natural',
            price: 28.00,
            quantity: 1,
            humidor_id: humidorId
        }
    ];
    
    let successCount = 0;
    let failCount = 0;
    
    for (const cigar of sampleCigars) {
        try {
            console.log(`Adding sample cigar: ${cigar.brand} ${cigar.name}`);
            const response = await fetch('/api/v1/cigars', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(cigar)
            });
            
            if (response.ok) {
                const result = await response.json();
                // Check if the response contains an error field
                if (result.error) {
                    console.error(`✗ Failed to add ${cigar.brand} ${cigar.name}:`, result.error);
                    failCount++;
                } else {
                    console.log(`✓ Successfully added: ${cigar.brand} ${cigar.name}`, result);
                    successCount++;
                }
            } else {
                const error = await response.text();
                console.error(`✗ Failed to add ${cigar.brand} ${cigar.name}:`, response.status, error);
                failCount++;
            }
        } catch (error) {
            console.error(`✗ Error adding ${cigar.brand} ${cigar.name}:`, error);
            failCount++;
        }
    }
    
    console.log(`=== SAMPLE DATA COMPLETE: ${successCount} added, ${failCount} failed ===`);
}

// Toast notifications
function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    
    elements.toastContainer.appendChild(toast);
    
    // Show toast
    setTimeout(() => {
        toast.classList.add('show');
    }, 100);
    
    // Hide and remove toast
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => {
            elements.toastContainer.removeChild(toast);
        }, 300);
    }, 5000);
}

// Utility functions
function setDefaultHumidorValues() {
    if (!elements.humidorName.value) {
        elements.humidorName.value = `${formData.user.username}'s Humidor`;
    }
}

// Handle enter key navigation
document.addEventListener('keydown', function(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        
        // If we're in a form field, validate and go to next step
        if (document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA') {
            goToNextStep();
        }
    }
});

// Auto-fill humidor name when user data is captured
elements.username.addEventListener('input', function() {
    if (currentStep === 2) {
        const username = elements.username.value.trim();
        if (username && !elements.humidorName.value) {
            // We'll set this when we move to step 3
            setTimeout(() => {
                if (currentStep === 3 && !elements.humidorName.value) {
                    elements.humidorName.value = `${username}'s Humidor`;
                }
            }, 100);
        }
    }
});

// Initialize - ensure this runs after DOM is loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
        console.log('DOM loaded, initializing wizard...');
        updateWizard();
    });
} else {
    console.log('DOM already loaded, initializing wizard immediately...');
    updateWizard();
}