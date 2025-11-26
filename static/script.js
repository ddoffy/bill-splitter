// State management
let people = [];
let history = [];
let editingPersonId = null;
let currentSessionId = null;
let currentEditSecret = null;
let isReadOnly = false;

// DOM elements
const addPersonForm = document.getElementById('addPersonForm');
const personNameInput = document.getElementById('personName');
const descriptionInput = document.getElementById('description');
const amountSpentInput = document.getElementById('amountSpent');
const isSponsorCheckbox = document.getElementById('isSponsor');
const sponsorAmountGroup = document.getElementById('sponsorAmountGroup');
const sponsorAmountInput = document.getElementById('sponsorAmount');
const peopleList = document.getElementById('peopleList');
const includeSponsorCheckbox = document.getElementById('includeSponsorInSplit');
const calculateBtn = document.getElementById('calculateBtn');
const clearAllBtn = document.getElementById('clearAllBtn');
const resultsSection = document.getElementById('resultsSection');
const submitBtn = document.getElementById('submitBtn');
const cancelEditBtn = document.getElementById('cancelEditBtn');
const shareBtn = document.getElementById('shareBtn');
const shareLinks = document.getElementById('shareLinks');
const viewLinkInput = document.getElementById('viewLink');
const editLinkInput = document.getElementById('editLink');
const copyViewLinkBtn = document.getElementById('copyViewLink');
const copyEditLinkBtn = document.getElementById('copyEditLink');
const archiveBtn = document.getElementById('archiveBtn');
const historyList = document.getElementById('historyList');
const participantSelect = document.getElementById('participantSelect');

// Event listeners
addPersonForm.addEventListener('submit', handleAddPerson);
calculateBtn.addEventListener('click', calculateSplit);
clearAllBtn.addEventListener('click', clearAllPeople);
isSponsorCheckbox.addEventListener('change', toggleSponsorAmount);
cancelEditBtn.addEventListener('click', cancelEdit);
if (shareBtn) shareBtn.addEventListener('click', shareSplit);
if (copyViewLinkBtn) copyViewLinkBtn.addEventListener('click', () => copyToClipboard(viewLinkInput, copyViewLinkBtn));
if (copyEditLinkBtn) copyEditLinkBtn.addEventListener('click', () => copyToClipboard(editLinkInput, copyEditLinkBtn));
if (archiveBtn) archiveBtn.addEventListener('click', archiveSession);
if (participantSelect) {
    participantSelect.addEventListener('change', function() {
        if (this.value) {
            personNameInput.value = this.value;
            this.value = ""; // Reset selection
            amountSpentInput.focus(); // Move to next field
        }
    });
}

function copyToClipboard(inputElement, buttonElement) {
    inputElement.select();
    inputElement.setSelectionRange(0, 99999); // For mobile devices
    
    // Try using the modern clipboard API
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(inputElement.value).then(() => {
            showCopyFeedback(buttonElement);
        }).catch(err => {
            console.error('Failed to copy: ', err);
            // Fallback
            document.execCommand('copy');
            showCopyFeedback(buttonElement);
        });
    } else {
        // Fallback for older browsers or non-secure contexts
        document.execCommand('copy');
        showCopyFeedback(buttonElement);
    }
}

function showCopyFeedback(buttonElement) {
    const originalText = buttonElement.textContent;
    buttonElement.textContent = 'Copied!';
    buttonElement.style.background = '#48bb78'; // Green
    
    setTimeout(() => {
        buttonElement.textContent = originalText;
        buttonElement.style.background = '#718096'; // Original gray
    }, 2000);
}

function toggleSponsorAmount() {
    sponsorAmountGroup.style.display = isSponsorCheckbox.checked ? 'block' : 'none';
    if (!isSponsorCheckbox.checked) {
        sponsorAmountInput.value = '0';
    }
}

// Initialize
init();

async function init() {
    loadHistoryFromLocalStorage();

    const urlParams = new URLSearchParams(window.location.search);
    const sessionId = urlParams.get('session');
    const secret = urlParams.get('secret');

    if (sessionId) {
        currentSessionId = sessionId;
        currentEditSecret = secret;
        
        if (!secret) {
            isReadOnly = true;
            document.body.classList.add('read-only');
            if (addPersonForm) addPersonForm.style.display = 'none';
            if (clearAllBtn) clearAllBtn.style.display = 'none';
            if (shareBtn) shareBtn.style.display = 'none';
        }
        
        await loadSession(sessionId);
    } else {
        loadPeopleFromLocalStorage();
    }
}

async function loadSession(id) {
    try {
        const response = await fetch(`/api/sessions/${id}`);
        if (response.ok) {
            const data = await response.json();
            people = data.people;
            renderPeople(people);
        } else {
            alert('Session not found! Loading local data instead.');
            loadPeopleFromLocalStorage();
        }
    } catch (e) {
        console.error(e);
        loadPeopleFromLocalStorage();
    }
}

function loadPeopleFromLocalStorage() {
    try {
        const storedPeople = localStorage.getItem('splitBillsPeople');
        if (storedPeople) {
            people = JSON.parse(storedPeople);
        } else {
            people = [];
        }
        renderPeople(people);
    } catch (error) {
        console.error('Error loading people from local storage:', error);
        people = [];
        renderPeople(people);
    }
}

// Save people
async function savePeople() {
    // Always save to local storage as backup/cache
    localStorage.setItem('splitBillsPeople', JSON.stringify(people));
    renderPeople(people);
    
    // If we are in an editable session, sync to server
    if (currentSessionId && currentEditSecret) {
        try {
            await fetch(`/api/sessions/${currentSessionId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Edit-Secret': currentEditSecret
                },
                body: JSON.stringify({ people })
            });
        } catch (e) {
            console.error('Failed to sync session', e);
        }
    }
}

async function shareSplit() {
    if (people.length === 0) {
        alert('Add some expenses first!');
        return;
    }
    
    try {
        const response = await fetch('/api/sessions', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ people })
        });
        
        if (response.ok) {
            const data = await response.json();
            currentSessionId = data.id;
            currentEditSecret = data.edit_secret;
            
            const baseUrl = window.location.origin;
            const viewUrl = `${baseUrl}/?session=${data.id}`;
            const editUrl = `${baseUrl}/?session=${data.id}&secret=${data.edit_secret}`;
            
            viewLinkInput.value = viewUrl;
            editLinkInput.value = editUrl;
            shareLinks.style.display = 'block';
            
            // Update URL without reloading
            window.history.pushState({}, '', editUrl);
        }
    } catch (e) {
        console.error(e);
        alert('Failed to create share link');
    }
}

// Add person to the list
function handleAddPerson(e) {
    e.preventDefault();
    
    const name = personNameInput.value.trim();
    const description = descriptionInput.value.trim();
    const amount = parseFloat(amountSpentInput.value);
    const isSponsor = isSponsorCheckbox.checked;
    const sponsorAmount = isSponsor ? parseFloat(sponsorAmountInput.value) : 0;
    
    if (name && amount >= 0) {
        if (editingPersonId) {
            // Update existing person
            people = people.map(p => {
                if (p.id === editingPersonId) {
                    return {
                        ...p,
                        name,
                        description,
                        amount_spent: amount,
                        is_sponsor: isSponsor,
                        sponsor_amount: sponsorAmount
                    };
                }
                return p;
            });
            
            // Reset edit mode
            cancelEdit();
        } else {
            // Add new person
            const newPerson = {
                id: Date.now(),
                name,
                description,
                amount_spent: amount,
                is_sponsor: isSponsor,
                sponsor_amount: sponsorAmount,
                is_receiver: false
            };

            people.push(newPerson);
            
            // Reset form
            personNameInput.value = '';
            descriptionInput.value = '';
            amountSpentInput.value = '0';
            isSponsorCheckbox.checked = false;
            sponsorAmountInput.value = '0';
            toggleSponsorAmount();
            
            // Focus back on name input for quick entry
            personNameInput.focus();
        }
        
        savePeople();
        
        // Hide results when modifying list
        resultsSection.style.display = 'none';
    }
}

// Edit person
function editPerson(id) {
    const person = people.find(p => p.id === id);
    if (!person) return;
    
    editingPersonId = id;
    
    // Populate form
    personNameInput.value = person.name;
    descriptionInput.value = person.description || '';
    amountSpentInput.value = person.amount_spent;
    isSponsorCheckbox.checked = person.is_sponsor;
    sponsorAmountInput.value = person.sponsor_amount;
    
    toggleSponsorAmount();
    
    // Update UI for edit mode
    submitBtn.textContent = 'Update Person';
    submitBtn.classList.remove('btn-primary');
    submitBtn.classList.add('btn-success');
    cancelEditBtn.style.display = 'block';
    
    // Scroll to form
    addPersonForm.scrollIntoView({ behavior: 'smooth', block: 'center' });
    personNameInput.focus();
}

// Cancel edit
function cancelEdit() {
    editingPersonId = null;
    
    // Reset form
    personNameInput.value = '';
    descriptionInput.value = '';
    amountSpentInput.value = '0';
    isSponsorCheckbox.checked = false;
    sponsorAmountInput.value = '0';
    toggleSponsorAmount();
    
    // Reset UI
    submitBtn.textContent = 'Add Person';
    submitBtn.classList.remove('btn-success');
    submitBtn.classList.add('btn-primary');
    cancelEditBtn.style.display = 'none';
}

// Remove person from list
function removePerson(id) {
    if (editingPersonId === id) {
        cancelEdit();
    }
    people = people.filter(p => p.id !== id);
    savePeople();
    resultsSection.style.display = 'none';
}

// Clear all people
function clearAllPeople() {
    if (confirm('Are you sure you want to remove all people?')) {
        people = [];
        savePeople();
        resultsSection.style.display = 'none';
    }
}

// Render people list
function renderPeople(people) {
    renderParticipants(people);

    const expensesCount = document.getElementById('expensesCount');
    const expensesTotal = document.getElementById('expensesTotal');
    const expensesSponsor = document.getElementById('expensesSponsor');

    if (people) {
        if (expensesCount) expensesCount.textContent = `(${people.length})`;
        
        const total = people.reduce((sum, p) => sum + (p.amount_spent || 0), 0);
        const sponsor = people.reduce((sum, p) => sum + (p.is_sponsor ? (p.sponsor_amount || 0) : 0), 0);
        
        if (expensesTotal) expensesTotal.textContent = total.toFixed(2);
        if (expensesSponsor) expensesSponsor.textContent = sponsor.toFixed(2);
    } else {
        if (expensesCount) expensesCount.textContent = '(0)';
        if (expensesTotal) expensesTotal.textContent = '0.00';
        if (expensesSponsor) expensesSponsor.textContent = '0.00';
    }

    if (!people || people.length === 0) {
        peopleList.innerHTML = '<div class="empty-message">No expenses added yet. Add one to get started!</div>';
        return;
    }
    
    peopleList.innerHTML = people.map((person, index) => `
        <div class="person-item ${person.is_sponsor ? 'sponsor' : ''}">
            <div class="person-info">
                <div class="person-name">
                    <span style="color: #888; font-size: 0.9em; margin-right: 8px;">#${index + 1}</span>
                    ${person.name}
                    ${person.is_sponsor ? '<span class="person-badge">SPONSOR</span>' : ''}
                    ${person.is_receiver ? '<span class="person-badge receiver-badge">RECEIVER</span>' : ''}
                </div>
                ${person.description ? `<div class="person-description">${person.description}</div>` : ''}
                <div class="person-amount">Spent: $${person.amount_spent.toFixed(2)}</div>
                ${person.is_sponsor && person.sponsor_amount > 0 ? `<div class="person-amount">Sponsoring: $${person.sponsor_amount.toFixed(2)}</div>` : ''}
                <div class="receiver-option">
                    <label style="font-size: 0.85em; cursor: pointer; display: flex; align-items: center; gap: 5px; margin-top: 5px;">
                        <input type="radio" name="receiver_group" 
                            ${person.is_receiver ? 'checked' : ''} 
                            onclick="setReceiver(${person.id})">
                        Mark as Receiver
                    </label>
                </div>
            </div>
            <div class="person-actions" style="display: flex; gap: 5px; flex-direction: column; ${isReadOnly ? 'display: none !important;' : ''}">
                <button class="btn" style="background: #4299e1; color: white; padding: 6px 12px; font-size: 14px; width: auto;" onclick="editPerson(${person.id})">Edit</button>
                <button class="btn btn-danger" onclick="removePerson(${person.id})">Remove</button>
            </div>
        </div>
    `).join('');
}

function renderParticipants(people) {
    const participantsList = document.getElementById('participantsList');
    const uniquePeopleCount = document.getElementById('uniquePeopleCount');
    
    if (!people) {
        participantsList.innerHTML = '';
        uniquePeopleCount.textContent = '(0)';
        return;
    }

    const uniqueNames = [...new Set(people.map(p => p.name))].sort();
    uniquePeopleCount.textContent = `(${uniqueNames.length})`;

    const participantSelect = document.getElementById('participantSelect');
    if (participantSelect) {
        if (uniqueNames.length > 0) {
            participantSelect.style.display = 'block';
            participantSelect.innerHTML = '<option value="">-- Select existing participant --</option>' + 
                uniqueNames.map(name => `<option value="${name}">${name}</option>`).join('');
        } else {
            participantSelect.style.display = 'none';
        }
    }

    if (uniqueNames.length === 0) {
        participantsList.innerHTML = '<div style="color: #888; font-style: italic;">No participants yet</div>';
        return;
    }

    participantsList.innerHTML = uniqueNames.map((name, index) => `
        <div style="padding: 10px 0; border-bottom: 1px solid #eee; display: flex; align-items: center;">
            <span style="color: #888; font-size: 0.9em; min-width: 30px; margin-right: 12px; display: inline-block; text-align: right;">#${index + 1}</span>
            <span style="font-weight: 600; font-size: 1.05em;">${name}</span>
        </div>
    `).join('');
}

// Set receiver
function setReceiver(id) {
    people = people.map(p => {
        if (p.id === id) {
            // Toggle if clicking the same one? No, radio buttons don't toggle off usually.
            // But for better UX let's allow toggling off if we click the label/input again?
            // Actually, the onclick fires after change.
            // Let's just set it. If they want to unset, they can clear all or we can add a logic.
            // For now, just set.
            return { ...p, is_receiver: true };
        }
        return { ...p, is_receiver: false };
    });
    savePeople();
}

// Calculate bill split
async function calculateSplit() {
    try {
        if (people.length === 0) {
            alert('Please add at least one person!');
            return;
        }
        
        const includeSponsor = includeSponsorCheckbox.checked;
        
        const calcResponse = await fetch('/api/calculate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                people,
                include_sponsor: includeSponsor
            })
        });
        
        if (!calcResponse.ok) {
            throw new Error('Calculation failed');
        }
        
        const result = await calcResponse.json();
        
        if (result.num_participants === 0 && result.amount_to_share > 0) {
            alert('No participants to split the bill! Either add non-sponsor people or include sponsors in the split.');
            return;
        }
        
        displayResults(result);
    } catch (error) {
        console.error('Error calculating split:', error);
        alert('Failed to calculate split. Please try again.');
    }
}

// Display calculation results
function displayResults(result) {
    document.getElementById('totalSpent').textContent = result.total_spent.toFixed(2);
    document.getElementById('totalSponsored').textContent = result.total_sponsored.toFixed(2);
    document.getElementById('amountToShare').textContent = result.amount_to_share.toFixed(2);
    document.getElementById('numParticipants').textContent = result.num_participants;
    document.getElementById('perPersonShare').textContent = result.per_person_share.toFixed(2);
    
    const settlementsList = document.getElementById('settlementsList');
    
    const receiver = result.settlements.find(s => s.is_receiver);

    settlementsList.innerHTML = result.settlements.map(settlement => {
        let message = '';
        let cssClass = '';
        
        if (settlement.settlement_type === 'pay') {
            const payTo = receiver && !settlement.is_receiver ? ` to <strong>${receiver.name}</strong>` : '';
            message = `
                <div class="settlement-header">
                    <strong>${settlement.name}</strong> should pay <span class="settlement-amount">$${Math.abs(settlement.balance).toFixed(2)}</span>${payTo}
                </div>
                <div class="settlement-details">
                    (Spent: $${settlement.amount_spent.toFixed(2)} - Sponsor: $${settlement.sponsor_cost.toFixed(2)} - Share: $${settlement.share_cost.toFixed(2)})
                </div>`;
            cssClass = 'pay';
        } else if (settlement.settlement_type === 'receive') {
            const receiveFrom = receiver && !settlement.is_receiver ? ` from <strong>${receiver.name}</strong>` : '';
            message = `
                <div class="settlement-header">
                    <strong>${settlement.name}</strong> should receive <span class="settlement-amount">$${settlement.balance.toFixed(2)}</span>${receiveFrom}
                </div>
                <div class="settlement-details">
                    (Spent: $${settlement.amount_spent.toFixed(2)} - Sponsor: $${settlement.sponsor_cost.toFixed(2)} - Share: $${settlement.share_cost.toFixed(2)})
                </div>`;
            cssClass = 'receive';
        } else {
            message = `
                <div class="settlement-header">
                    <strong>${settlement.name}</strong> is all settled up! <span class="settlement-amount">$0.00</span>
                </div>
                <div class="settlement-details">
                    (Spent: $${settlement.amount_spent.toFixed(2)} - Sponsor: $${settlement.sponsor_cost.toFixed(2)} - Share: $${settlement.share_cost.toFixed(2)})
                </div>`;
            cssClass = 'settled';
        }
        
        if (settlement.is_receiver) {
             message += `<div class="receiver-note" style="font-size: 0.85em; color: #666; margin-top: 4px;">(Designated Receiver)</div>`;
        }

        return `<div class="settlement-item ${cssClass}">${message}</div>`;
    }).join('');
    
    resultsSection.style.display = 'block';
    resultsSection.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

// History Management

function loadHistoryFromLocalStorage() {
    try {
        const storedHistory = localStorage.getItem('splitBillsHistory');
        if (storedHistory) {
            history = JSON.parse(storedHistory);
        } else {
            history = [];
        }
        renderHistory();
    } catch (error) {
        console.error('Error loading history:', error);
        history = [];
    }
}

function saveHistoryToLocalStorage() {
    localStorage.setItem('splitBillsHistory', JSON.stringify(history));
    renderHistory();
}

function renderHistory() {
    if (!historyList) return;
    
    historyList.innerHTML = '';
    
    if (history.length === 0) {
        historyList.innerHTML = '<div style="text-align: center; color: #999; padding: 20px;">No archives yet</div>';
        return;
    }
    
    history.slice().reverse().forEach((item, index) => {
        // Calculate actual index in original array
        const originalIndex = history.length - 1 - index;
        
        const el = document.createElement('div');
        el.className = 'history-item';
        el.style.background = '#f7fafc';
        el.style.border = '1px solid #e2e8f0';
        el.style.borderRadius = '6px';
        el.style.padding = '10px';
        el.style.marginBottom = '10px';
        
        const date = new Date(item.timestamp).toLocaleDateString();
        const total = item.people.reduce((sum, p) => sum + p.amount_spent, 0).toFixed(2);
        
        const uniqueNames = [...new Set(item.people.map(p => p.name))].sort();
        const count = uniqueNames.length;
        const namesList = uniqueNames.join(', ');
        
        el.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                <span style="font-weight: bold; font-size: 0.9em;">${date}</span>
                <span style="font-size: 0.8em; color: #666;">$${total}</span>
            </div>
            <div style="font-size: 0.8em; color: #666; margin-bottom: 8px;">
                <strong>${count} Participants:</strong> <span style="font-style: italic;">${namesList}</span>
            </div>
            <div style="display: flex; gap: 5px;">
                <button onclick="useTemplate(${originalIndex})" class="btn btn-sm" style="flex: 1; background: #4299e1; color: white; padding: 4px; font-size: 0.8em; border: none; border-radius: 4px; cursor: pointer;">Use Template</button>
                <button onclick="deleteHistoryItem(${originalIndex})" class="btn btn-sm" style="background: #e53e3e; color: white; padding: 4px 8px; font-size: 0.8em; border: none; border-radius: 4px; cursor: pointer;">×</button>
            </div>
        `;
        
        historyList.appendChild(el);
    });
}

function archiveSession() {
    if (people.length === 0) {
        alert('Nothing to archive!');
        return;
    }
    
    if (!confirm('This will save the current session to history and clear the workspace. Continue?')) {
        return;
    }
    
    const historyItem = {
        timestamp: Date.now(),
        people: JSON.parse(JSON.stringify(people)), // Deep copy
        sessionId: currentSessionId
    };
    
    history.push(historyItem);
    saveHistoryToLocalStorage();
    
    // Clear current session
    clearAllPeople();
    
    // If we were in a shared session, reset URL to home
    if (currentSessionId) {
        window.history.pushState({}, '', '/');
        currentSessionId = null;
        currentEditSecret = null;
        isReadOnly = false;
        document.body.classList.remove('read-only');
        if (addPersonForm) addPersonForm.style.display = 'block';
        if (clearAllBtn) clearAllBtn.style.display = 'block';
        if (shareBtn) shareBtn.style.display = 'block';
        if (shareLinks) shareLinks.style.display = 'none';
    }
}

function useTemplate(index) {
    if (people.length > 0) {
        if (!confirm('This will overwrite your current workspace. Continue?')) {
            return;
        }
    }
    
    const item = history[index];
    if (!item) return;
    
    // Deep copy people from history
    people = JSON.parse(JSON.stringify(item.people));
    
    // Reset session context
    currentSessionId = null;
    currentEditSecret = null;
    isReadOnly = false;
    document.body.classList.remove('read-only');
    if (addPersonForm) addPersonForm.style.display = 'block';
    if (clearAllBtn) clearAllBtn.style.display = 'block';
    if (shareBtn) shareBtn.style.display = 'block';
    if (shareLinks) shareLinks.style.display = 'none';
    window.history.pushState({}, '', '/');
    
    savePeople();
    renderPeople(people);
}

function deleteHistoryItem(index) {
    if (!confirm('Delete this archive?')) return;
    history.splice(index, 1);
    saveHistoryToLocalStorage();
}

// Expose functions to global scope for inline onclick handlers
window.useTemplate = useTemplate;
window.deleteHistoryItem = deleteHistoryItem;
