// State management
let people = [];
let editingPersonId = null;

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
const restrictSponsorCheckbox = document.getElementById('restrictSponsorToSpent');
const calculateBtn = document.getElementById('calculateBtn');
const clearAllBtn = document.getElementById('clearAllBtn');
const resultsSection = document.getElementById('resultsSection');
const submitBtn = document.getElementById('submitBtn');
const cancelEditBtn = document.getElementById('cancelEditBtn');

// Event listeners
addPersonForm.addEventListener('submit', handleAddPerson);
calculateBtn.addEventListener('click', calculateSplit);
clearAllBtn.addEventListener('click', clearAllPeople);
isSponsorCheckbox.addEventListener('change', toggleSponsorAmount);
cancelEditBtn.addEventListener('click', cancelEdit);

function toggleSponsorAmount() {
    sponsorAmountGroup.style.display = isSponsorCheckbox.checked ? 'block' : 'none';
    if (!isSponsorCheckbox.checked) {
        sponsorAmountInput.value = '0';
    }
}

// Initialize: Load people from local storage
loadPeople();

// Load people from local storage
function loadPeople() {
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

// Save people to local storage
function savePeople() {
    localStorage.setItem('splitBillsPeople', JSON.stringify(people));
    renderPeople(people);
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
    if (!people || people.length === 0) {
        peopleList.innerHTML = '<div class="empty-message">No people added yet. Add someone to get started!</div>';
        return;
    }
    
    peopleList.innerHTML = people.map(person => `
        <div class="person-item ${person.is_sponsor ? 'sponsor' : ''}">
            <div class="person-info">
                <div class="person-name">
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
            <div class="person-actions" style="display: flex; gap: 5px; flex-direction: column;">
                <button class="btn" style="background: #4299e1; color: white; padding: 6px 12px; font-size: 14px; width: auto;" onclick="editPerson(${person.id})">Edit</button>
                <button class="btn btn-danger" onclick="removePerson(${person.id})">Remove</button>
            </div>
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
        const restrictSponsor = restrictSponsorCheckbox.checked;
        
        const calcResponse = await fetch('/api/calculate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                people,
                include_sponsor: includeSponsor,
                restrict_sponsor_to_spent: restrictSponsor
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
