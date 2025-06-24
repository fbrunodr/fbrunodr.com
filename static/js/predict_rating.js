document.addEventListener('DOMContentLoaded', function() {
    const handleInput = document.getElementById('handle-input');
    const predictBtn = document.getElementById('predict-btn');
    const resultContainer = document.getElementById('result-container');
    const resultContent = document.getElementById('result-content');
    const buttonText = document.querySelector('.button-text');
    const loadingSpinner = document.querySelector('.loading-spinner');

    // Handle form submission
    predictBtn.addEventListener('click', async function() {
        const handle = handleInput.value.trim();

        if (!handle) {
            showError('Please enter a Codeforces handle');
            return;
        }

        // Show loading state
        setLoadingState(true);
        hideResult();

        try {
            const response = await fetch('/api/predict-codeforces-rating', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ handle: handle })
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

    // Handle Enter key press
    handleInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            predictBtn.click();
        }
    });

    // Handle input validation
    handleInput.addEventListener('input', function() {
        // Remove any non-alphanumeric characters except underscores and hyphens
        this.value = this.value.replace(/[^a-zA-Z0-9_-]/g, '');
    });

    function setLoadingState(loading) {
        if (loading) {
            predictBtn.disabled = true;
            buttonText.style.display = 'none';
            loadingSpinner.style.display = 'inline';
        } else {
            predictBtn.disabled = false;
            buttonText.style.display = 'inline';
            loadingSpinner.style.display = 'none';
        }
    }

    function showSuccess(data) {
        const { current_rating, predicted_rating, rating_change, motivational_message } = data;

        // Determine the color and icon based on rating change
        let changeColor, changeIcon, changeClass;
        if (rating_change > 0) {
            changeColor = '#5dfa5c';
            changeIcon = '📈';
            changeClass = 'rating-increase';
        } else if (rating_change < 0) {
            changeColor = '#ff6b6b';
            changeIcon = '📉';
            changeClass = 'rating-decrease';
        } else {
            changeColor = '#ffc107';
            changeIcon = '➡️';
            changeClass = 'rating-stable';
        }

        resultContent.className = `result-content result-success ${changeClass}`;
        resultContent.innerHTML = `
            <div class="result-title">✅ Prediction Successful</div>
            <div class="result-message">${data.message}</div>

            <div class="rating-comparison">
                <div class="current-rating">
                    <div class="rating-label">Current Rating</div>
                    <div class="rating-value">${current_rating}</div>
                </div>

                <div class="rating-arrow">
                    <div class="arrow-icon">${changeIcon}</div>
                    <div class="change-amount" style="color: ${changeColor}">
                        ${rating_change > 0 ? '+' : ''}${rating_change}
                    </div>
                </div>

                <div class="predicted-rating">
                    <div class="rating-label">Predicted Rating</div>
                    <div class="rating-value predicted">${predicted_rating}</div>
                </div>
            </div>

            <div class="motivational-message">
                ${motivational_message}
            </div>

            <div class="timeline-info">
                <div class="timeline-icon">⏰</div>
                <div class="timeline-text">Prediction for 6 months from now</div>
            </div>
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

    // Add some visual feedback for the input
    handleInput.addEventListener('focus', function() {
        this.parentElement.style.transform = 'scale(1.02)';
    });

    handleInput.addEventListener('blur', function() {
        this.parentElement.style.transform = 'scale(1)';
    });

    // Add click animation to button
    predictBtn.addEventListener('mousedown', function() {
        this.style.transform = 'scale(0.98)';
    });

    predictBtn.addEventListener('mouseup', function() {
        this.style.transform = 'scale(1)';
    });

    predictBtn.addEventListener('mouseleave', function() {
        this.style.transform = 'scale(1)';
    });
}); 