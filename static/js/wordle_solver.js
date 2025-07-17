document.addEventListener('DOMContentLoaded', function() {
    const greenInput = document.getElementById('green-input');
    const greyInput = document.getElementById('grey-input');
    const yellowInput = document.getElementById('yellow-input');
    const solveBtn = document.getElementById('solve-btn');
    const resultContainer = document.getElementById('result-container');
    const resultContent = document.getElementById('result-content');
    const buttonText = document.querySelector('.button-text');
    const loadingSpinner = document.querySelector('.loading-spinner');

    // Handle form submission
    solveBtn.addEventListener('click', async function() {
        const green = greenInput.value.trim();
        const grey = greyInput.value.trim();
        const yellow = yellowInput.value.trim();

        // Basic validation
        if (!green || green.length !== 5) {
            showError('Please enter exactly 5 characters for green letters (use ? for unknown positions)');
            return;
        }

        // Show loading state
        setLoadingState(true);
        hideResult();

        try {
            const response = await fetch('/api/wordle-solve', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    green_letters: green,
                    grey_letters: grey,
                    yellow_letters: yellow
                })
            });

            const data = await response.json();

            if (data.success) {
                showSuccess(data);
            } else {
                showError(data.message);
            }
        } catch (error) {
            console.error('Error:', error);
            showError('Failed to connect to the server. Please try again.');
        } finally {
            setLoadingState(false);
        }
    });

    // Handle Enter key press on any input
    [greenInput, greyInput, yellowInput].forEach(input => {
        input.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                solveBtn.click();
            }
        });
    });

    // Input validation and formatting
    greenInput.addEventListener('input', function() {
        // Allow only letters and ? for green input, max 5 characters
        this.value = this.value.replace(/[^a-zA-Z?]/g, '').substring(0, 5);

        // Auto-complete with ? if less than 5 characters when user stops typing
        clearTimeout(this.timeout);
        this.timeout = setTimeout(() => {
            if (this.value.length > 0 && this.value.length < 5) {
                this.value = this.value.padEnd(5, '?');
            }
        }, 1000);
    });

    greyInput.addEventListener('input', function() {
        // Allow only letters for grey input
        this.value = this.value.replace(/[^a-zA-Z]/g, '');
    });

    yellowInput.addEventListener('input', function() {
        // Allow only letters for yellow input
        this.value = this.value.replace(/[^a-zA-Z]/g, '');
    });

    // Auto-focus next input
    greenInput.addEventListener('input', function() {
        if (this.value.length === 5) {
            greyInput.focus();
        }
    });

    function setLoadingState(loading) {
        const formContainer = document.querySelector('.form-container');

        if (loading) {
            solveBtn.disabled = true;
            solveBtn.classList.add('loading');
            buttonText.style.display = 'none';
            loadingSpinner.style.display = 'inline';
            formContainer.classList.add('loading');

            // Add subtle input field animation
            [greenInput, greyInput, yellowInput].forEach(input => {
                input.style.opacity = '0.7';
                input.style.transform = 'scale(0.98)';
            });
        } else {
            solveBtn.disabled = false;
            solveBtn.classList.remove('loading');
            buttonText.style.display = 'inline';
            loadingSpinner.style.display = 'none';
            formContainer.classList.remove('loading');

            // Restore input fields
            [greenInput, greyInput, yellowInput].forEach(input => {
                input.style.opacity = '1';
                input.style.transform = 'scale(1)';
            });
        }
    }

    function showSuccess(data) {
        const { suggestions, message } = data;

        resultContent.className = 'result-content result-success';

        let suggestionsHtml = '';
        if (suggestions && suggestions.length > 0) {
            suggestionsHtml = `
                <div class="suggestions-grid">
                    ${suggestions.map(word => `
                        <div class="suggestion-word" onclick="copyToClipboard('${word}')" title="Click to copy">
                            ${word}
                        </div>
                    `).join('')}
                </div>
            `;
        }

        resultContent.innerHTML = `
            <div class="result-title">🎯 Solutions Found!</div>
            <div class="result-message">${message}</div>
            ${suggestionsHtml}
            ${suggestions && suggestions.length > 0 ? '<p style="margin-top: 20px; font-size: 0.9rem; opacity: 0.8;">💡 Click on any word to copy it</p>' : ''}
        `;

        resultContainer.style.display = 'block';

        // Scroll to result
        resultContainer.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }

    function showError(message) {
        resultContent.className = 'result-content result-error';
        resultContent.innerHTML = `
            <div class="result-title">❌ Error</div>
            <div class="result-message">${message}</div>
        `;
        resultContainer.style.display = 'block';

        // Scroll to result
        resultContainer.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }

    function hideResult() {
        resultContainer.style.display = 'none';
    }

    // Copy to clipboard function
    window.copyToClipboard = function(text) {
        navigator.clipboard.writeText(text).then(function() {
            // Show temporary feedback
            const event = new CustomEvent('wordCopied', { detail: text });
            document.dispatchEvent(event);

            // Visual feedback
            showToast(`Copied "${text}" to clipboard!`);
        }).catch(function(err) {
            console.error('Could not copy text: ', err);
            showToast('Failed to copy to clipboard');
        });
    };

    // Toast notification function
    function showToast(message) {
        // Remove existing toast
        const existingToast = document.querySelector('.toast');
        if (existingToast) {
            existingToast.remove();
        }

        // Create new toast
        const toast = document.createElement('div');
        toast.className = 'toast';
        toast.textContent = message;
        toast.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #6aaa64;
            color: white;
            padding: 12px 20px;
            border-radius: 8px;
            font-weight: 500;
            z-index: 1000;
            animation: slideIn 0.3s ease;
        `;

        // Add animation keyframes if not already added
        if (!document.querySelector('#toast-styles')) {
            const style = document.createElement('style');
            style.id = 'toast-styles';
            style.textContent = `
                @keyframes slideIn {
                    from { transform: translateX(100%); opacity: 0; }
                    to { transform: translateX(0); opacity: 1; }
                }
                @keyframes slideOut {
                    from { transform: translateX(0); opacity: 1; }
                    to { transform: translateX(100%); opacity: 0; }
                }
            `;
            document.head.appendChild(style);
        }

        document.body.appendChild(toast);

        // Remove toast after 3 seconds
        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 3000);
    }

    // Add visual feedback for inputs
    [greenInput, greyInput, yellowInput].forEach(input => {
        input.addEventListener('focus', function() {
            this.parentElement.style.transform = 'scale(1.01)';
        });

        input.addEventListener('blur', function() {
            this.parentElement.style.transform = 'scale(1)';
        });
    });

    // Add click animation to button
    solveBtn.addEventListener('mousedown', function() {
        this.style.transform = 'scale(0.98)';
    });

    solveBtn.addEventListener('mouseup', function() {
        this.style.transform = 'scale(1)';
    });

    solveBtn.addEventListener('mouseleave', function() {
        this.style.transform = 'scale(1)';
    });

    // Auto-focus first input on page load
    greenInput.focus();
}); 