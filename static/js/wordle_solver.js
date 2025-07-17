document.addEventListener('DOMContentLoaded', function() {
    const wordleGrid = document.getElementById('wordle-grid');
    const addRowBtn = document.getElementById('add-row-btn');
    const solveBtn = document.getElementById('solve-btn');
    const resultContainer = document.getElementById('result-container');
    const resultContent = document.getElementById('result-content');
    const buttonText = document.querySelector('.button-text');
    const loadingSpinner = document.querySelector('.loading-spinner');

    let rowCount = 0;

    // Color states for tiles: 0 = grey, 1 = yellow, 2 = green
    const TILE_STATES = ['empty', 'grey', 'yellow', 'green'];
    const STATE_CODES = { 'empty': '', 'grey': '0', 'yellow': '1', 'green': '2' };

    // Initialize with one row
    addRow();

    // Add row functionality
    addRowBtn.addEventListener('click', function() {
        addRow();
    });

    // Solve button functionality
    solveBtn.addEventListener('click', async function() {
        const validationResult = validateAllRows();

        if (!validationResult.isValid) {
            showError(validationResult.errorMessage);
            return;
        }

        const guesses = validationResult.completeGuesses;

        // If no complete guesses, get initial suggestions
        if (guesses.length === 0) {
            await makeApiCall([]);
            return;
        }

        await makeApiCall(guesses);
    });

    async function makeApiCall(guesses) {
        // Show loading state
        setLoadingState(true);
        hideResult();

        try {
            const response = await fetch('/api/wordle', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ guesses: guesses })
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
    }

    function addRow() {
        rowCount++;
        const row = createWordleRow(rowCount);
        wordleGrid.appendChild(row);

        // Focus on first tile of new row
        const firstTile = row.querySelector('.wordle-tile input');
        firstTile.focus();
    }

    function createWordleRow(rowId) {
        const row = document.createElement('div');
        row.className = 'wordle-row';
        row.dataset.rowId = rowId;

        // Create 5 tiles
        for (let i = 0; i < 5; i++) {
            const tile = createWordleTile(rowId, i);
            row.appendChild(tile);
        }

        // Always add remove button (allow deletion of any row including first)
        const actions = document.createElement('div');
        actions.className = 'row-actions';

        const removeBtn = document.createElement('button');
        removeBtn.className = 'remove-row-btn';
        removeBtn.innerHTML = '×';
        removeBtn.title = 'Remove this row';
        removeBtn.addEventListener('click', () => removeRow(rowId));

        actions.appendChild(removeBtn);
        row.appendChild(actions);

        return row;
    }

    function createWordleTile(rowId, tileIndex) {
        const tile = document.createElement('div');
        tile.className = 'wordle-tile empty';
        tile.dataset.state = 'empty';
        tile.dataset.rowId = rowId;
        tile.dataset.tileIndex = tileIndex;

        const input = document.createElement('input');
        input.type = 'text';
        input.maxLength = 1;
        input.addEventListener('input', handleTileInput);
        input.addEventListener('keydown', handleTileKeydown);

        tile.appendChild(input);

        // Click to cycle colors (only when tile has a letter)
        tile.addEventListener('click', function() {
            if (input.value.trim() !== '') {
                cycleTileState(tile);
            }
        });

        return tile;
    }

    function handleTileInput(event) {
        const input = event.target;
        const tile = input.parentElement;

        // Only allow letters
        input.value = input.value.replace(/[^a-zA-Z]/g, '').toUpperCase();

        if (input.value) {
            // Set to grey by default when letter is entered
            if (tile.dataset.state === 'empty') {
                setTileState(tile, 'grey');
            }

            // Move to next tile
            moveToNextTile(tile);
        } else {
            // Reset to empty if no letter
            setTileState(tile, 'empty');
        }
    }

    function handleTileKeydown(event) {
        const input = event.target;
        const tile = input.parentElement;

        if (event.key === 'Backspace' && input.value === '') {
            // Move to previous tile if current is empty
            moveToPrevTile(tile);
        } else if (event.key === 'ArrowLeft') {
            event.preventDefault();
            moveToPrevTile(tile);
        } else if (event.key === 'ArrowRight') {
            event.preventDefault();
            moveToNextTile(tile);
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            moveToTileAbove(tile);
        } else if (event.key === 'ArrowDown') {
            event.preventDefault();
            moveToTileBelow(tile);
        } else if (event.key === ' ' || event.key === 'Enter') {
            event.preventDefault();
            if (input.value.trim() !== '') {
                cycleTileState(tile);
            }
        }
    }

    function cycleTileState(tile) {
        const currentState = tile.dataset.state;
        let nextStateIndex;

        if (currentState === 'empty' || currentState === 'grey') {
            nextStateIndex = 2; // yellow
        } else if (currentState === 'yellow') {
            nextStateIndex = 3; // green
        } else if (currentState === 'green') {
            nextStateIndex = 1; // grey
        }

        const nextState = TILE_STATES[nextStateIndex];
        setTileState(tile, nextState);
    }

    function setTileState(tile, state) {
        // Remove all state classes
        TILE_STATES.forEach(s => tile.classList.remove(s));

        // Add new state
        tile.classList.add(state);
        tile.dataset.state = state;
    }

    function moveToNextTile(currentTile) {
        const rowId = currentTile.dataset.rowId;
        const tileIndex = parseInt(currentTile.dataset.tileIndex);
        const nextTile = document.querySelector(`[data-row-id="${rowId}"][data-tile-index="${tileIndex + 1}"] input`);

        if (nextTile) {
            nextTile.focus();
        }
    }

    function moveToPrevTile(currentTile) {
        const rowId = currentTile.dataset.rowId;
        const tileIndex = parseInt(currentTile.dataset.tileIndex);
        const prevTile = document.querySelector(`[data-row-id="${rowId}"][data-tile-index="${tileIndex - 1}"] input`);

        if (prevTile) {
            prevTile.focus();
        }
    }

    function moveToTileAbove(currentTile) {
        const rowId = parseInt(currentTile.dataset.rowId);
        const tileIndex = currentTile.dataset.tileIndex;
        const aboveTile = document.querySelector(`[data-row-id="${rowId - 1}"][data-tile-index="${tileIndex}"] input`);

        if (aboveTile) {
            aboveTile.focus();
        }
    }

    function moveToTileBelow(currentTile) {
        const rowId = parseInt(currentTile.dataset.rowId);
        const tileIndex = currentTile.dataset.tileIndex;
        const belowTile = document.querySelector(`[data-row-id="${rowId + 1}"][data-tile-index="${tileIndex}"] input`);

        if (belowTile) {
            belowTile.focus();
        }
    }

    function removeRow(rowId) {
        const row = document.querySelector(`[data-row-id="${rowId}"]`);
        if (row) {
            row.remove();
        }
    }

    function validateAllRows() {
        const rows = document.querySelectorAll('.wordle-row');
        const completeGuesses = [];

        // Special case: if no rows exist, allow for initial suggestions
        if (rows.length === 0) {
            return {
                isValid: true,
                completeGuesses: []
            };
        }

        for (let rowIndex = 0; rowIndex < rows.length; rowIndex++) {
            const row = rows[rowIndex];
            const tiles = row.querySelectorAll('.wordle-tile');

            let word = '';
            let feedback = '';
            let letterCount = 0;
            let feedbackCount = 0;

            tiles.forEach(tile => {
                const input = tile.querySelector('input');
                const letter = input.value.trim().toLowerCase();
                const state = tile.dataset.state;

                if (letter) {
                    letterCount++;
                    word += letter;

                    if (state !== 'empty') {
                        feedbackCount++;
                        feedback += STATE_CODES[state] || '';
                    } else {
                        feedback += '';
                    }
                } else {
                    word += '';
                    feedback += '';
                }
            });

            // ALL visible rows must be complete (5 letters + 5 colors)
            const isCompleteRow = letterCount === 5 && feedbackCount === 5;

            if (!isCompleteRow) {
                if (letterCount === 0 && feedbackCount === 0) {
                    return {
                        isValid: false,
                        errorMessage: `Row ${rowIndex + 1}: This row is empty. Please fill it completely or remove it using the × button`
                    };
                } else if (letterCount < 5) {
                    return {
                        isValid: false,
                        errorMessage: `Row ${rowIndex + 1}: Please enter exactly 5 letters (currently has ${letterCount})`
                    };
                } else if (feedbackCount < 5) {
                    return {
                        isValid: false,
                        errorMessage: `Row ${rowIndex + 1}: Please set feedback colors for all 5 letters (currently ${feedbackCount}/5 have colors)`
                    };
                }
            }

            // Add complete row to guesses
            completeGuesses.push({ word, feedback });
        }

        return {
            isValid: true,
            completeGuesses: completeGuesses
        };
    }

    function setLoadingState(loading) {
        const formContainer = document.querySelector('.form-container');

        if (loading) {
            solveBtn.disabled = true;
            solveBtn.classList.add('loading');
            buttonText.style.display = 'none';
            loadingSpinner.style.display = 'inline';
            formContainer.classList.add('loading');
            addRowBtn.disabled = true;
        } else {
            solveBtn.disabled = false;
            solveBtn.classList.remove('loading');
            buttonText.style.display = 'inline';
            loadingSpinner.style.display = 'none';
            formContainer.classList.remove('loading');
            addRowBtn.disabled = false;
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
            <div class="result-title">🎯 Best Suggestions</div>
            <div class="result-message">${message}</div>
            ${suggestionsHtml}
            ${suggestions && suggestions.length > 0 ? '<p style="margin-top: 15px; font-size: 0.9rem; opacity: 0.8;">💡 Click on any word to copy it</p>' : ''}
        `;

        resultContainer.style.display = 'block';
        resultContainer.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }

    function showError(message) {
        resultContent.className = 'result-content result-error';
        resultContent.innerHTML = `
            <div class="result-title">❌ Error</div>
            <div class="result-message">${message}</div>
        `;
        resultContainer.style.display = 'block';
        resultContainer.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }

    function hideResult() {
        resultContainer.style.display = 'none';
    }

    // Copy to clipboard function
    window.copyToClipboard = function(text) {
        navigator.clipboard.writeText(text).then(function() {
            showToast(`Copied "${text}" to clipboard!`);
        }).catch(function(err) {
            console.error('Could not copy text: ', err);
            showToast('Failed to copy to clipboard');
        });
    };

    // Toast notification function
    function showToast(message) {
        const existingToast = document.querySelector('.toast');
        if (existingToast) {
            existingToast.remove();
        }

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
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        `;

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

        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 3000);
    }
}); 