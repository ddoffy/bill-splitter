#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::calculate_split_internal;

    fn create_person(
        id: u64,
        name: &str,
        amount_spent: f64,
        quantity: u32,
        tip: f64,
        paid_by: Option<String>,
    ) -> Person {
        Person {
            id,
            name: name.to_string(),
            description: String::new(),
            amount_spent,
            quantity,
            tip,
            is_sponsor: false,
            sponsor_amount: 0.0,
            is_receiver: false,
            paid_by,
        }
    }

    #[test]
    fn test_simple_equal_split() {
        let people = vec![
            create_person(1, "Alice", 100.0, 1, 10.0, None),
            create_person(2, "Bob", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        assert_eq!(response.total_spent, 110.0);
        assert_eq!(response.num_participants, 2);
        assert_eq!(response.per_person_share, 55.0);

        // Alice should receive 55.0 (paid 110, should pay 55)
        // Bob should pay 55.0 (paid 0, should pay 55)
        let alice = response.settlements.iter().find(|s| s.name == "Alice").unwrap();
        let bob = response.settlements.iter().find(|s| s.name == "Bob").unwrap();

        assert_eq!(alice.balance, 55.0);
        assert_eq!(alice.settlement_type, "receive");
        assert_eq!(bob.balance, -55.0);
        assert_eq!(bob.settlement_type, "pay");
    }

    #[test]
    fn test_reimbursement_paid_by_other() {
        // Son pays $1000, marked "paid by Tuan"
        // Son should receive $1000, Tuan should pay $1000
        let people = vec![
            create_person(1, "Son", 1000.0, 1, 0.0, Some("Tuan".to_string())),
            create_person(2, "Tuan", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        // Debug output
        println!("Total spent: {}", response.total_spent);
        println!("Amount to share: {}", response.amount_to_share);
        
        let son = response.settlements.iter().find(|s| s.name == "Son").unwrap();
        let tuan = response.settlements.iter().find(|s| s.name == "Tuan").unwrap();
        
        println!("Son: amount_spent={}, balance={}", son.amount_spent, son.balance);
        println!("Tuan: amount_spent={}, balance={}", tuan.amount_spent, tuan.balance);

        // No shared expenses since it's all reimbursement
        assert_eq!(response.amount_to_share, 0.0);
        assert_eq!(response.per_person_share, 0.0);

        // Son should receive 1000 (will_receive_from_others = 1000, owes = 0)
        assert_eq!(son.balance, 1000.0);
        assert_eq!(son.settlement_type, "receive");

        // Tuan should pay 1000 (will_receive = 0, owes_to_others = 1000)
        assert_eq!(tuan.balance, -1000.0);
        assert_eq!(tuan.settlement_type, "pay");
    }

    #[test]
    fn test_private_expense_paid_by_self() {
        // Son pays $1000 marked "paid by Son" (private expense)
        // Bob pays $500 (shared expense)
        let people = vec![
            create_person(1, "Son", 1000.0, 1, 0.0, Some("Son".to_string())),
            create_person(2, "Bob", 500.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        // Only Bob's $500 is shared (Son's is private)
        assert_eq!(response.amount_to_share, 500.0);
        assert_eq!(response.per_person_share, 250.0);

        let son = response.settlements.iter().find(|s| s.name == "Son").unwrap();
        let bob = response.settlements.iter().find(|s| s.name == "Bob").unwrap();

        // Son pays his share (250) + private expense (1000) = -250
        assert_eq!(son.balance, -250.0);
        assert_eq!(son.settlement_type, "pay");

        // Bob receives: paid 500, should pay 250 = +250
        assert_eq!(bob.balance, 250.0);
        assert_eq!(bob.settlement_type, "receive");
    }

    #[test]
    fn test_complex_scenario_multiple_reimbursements() {
        // Real scenario from user:
        // Son pays multiple expenses, some reimbursed by others
        let people = vec![
            create_person(1, "Son", 990.0, 1, 99.0, Some("Tuan".to_string())),
            create_person(2, "Son", 990.0, 1, 99.0, Some("Doffy".to_string())),
            create_person(3, "Son", 500.0, 1, 50.0, Some("Doffy".to_string())),
            create_person(4, "Son", 990.0, 2, 99.0, Some("Dac".to_string())),
            create_person(5, "Son", 500.0, 1, 50.0, Some("Tuan".to_string())),
            create_person(6, "Dac", 0.0, 1, 0.0, None),
            create_person(7, "Doffy", 0.0, 1, 0.0, None),
            create_person(8, "Tuan", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        // All expenses are reimbursements, no shared pool
        assert_eq!(response.amount_to_share, 0.0);

        let son = response.settlements.iter().find(|s| s.name == "Son").unwrap();
        let dac = response.settlements.iter().find(|s| s.name == "Dac").unwrap();
        let doffy = response.settlements.iter().find(|s| s.name == "Doffy").unwrap();
        let tuan = response.settlements.iter().find(|s| s.name == "Tuan").unwrap();

        // Calculate expected values
        let tuan_owes = 1089.0 + 550.0; // 1639
        let doffy_owes = 1089.0 + 550.0; // 1639
        let dac_owes = 2079.0; // 1980 + 99

        // Son should receive total reimbursements
        assert_eq!(son.balance, tuan_owes + doffy_owes + dac_owes);
        assert_eq!(son.settlement_type, "receive");

        // Each person should pay what they owe
        assert_eq!(tuan.balance, -tuan_owes);
        assert_eq!(tuan.settlement_type, "pay");
        assert_eq!(doffy.balance, -doffy_owes);
        assert_eq!(doffy.settlement_type, "pay");
        assert_eq!(dac.balance, -dac_owes);
        assert_eq!(dac.settlement_type, "pay");

        // Total should balance to zero
        let total_balance: f64 = response.settlements.iter().map(|s| s.balance).sum();
        assert!(total_balance.abs() < 0.01, "Total balance should be near zero, got {}", total_balance);
    }

    #[test]
    fn test_quantity_multiplier() {
        // Test that quantity multiplies the amount correctly
        let people = vec![
            create_person(1, "Alice", 100.0, 2, 10.0, None), // $200 + $10 tip
            create_person(2, "Bob", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        assert_eq!(response.total_spent, 210.0); // 100*2 + 10
        assert_eq!(response.per_person_share, 105.0);
    }

    #[test]
    fn test_mixed_reimbursement_and_shared() {
        // Mix of shared expenses and reimbursements
        let people = vec![
            create_person(1, "Alice", 100.0, 1, 0.0, None), // Shared
            create_person(2, "Bob", 50.0, 1, 0.0, Some("Charlie".to_string())), // Charlie owes Bob
            create_person(3, "Charlie", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);

        println!("Total spent: {}", response.total_spent);
        println!("Amount to share: {}", response.amount_to_share);

        // Only Alice's $100 is shared
        assert_eq!(response.amount_to_share, 100.0);
        assert_eq!(response.per_person_share, 100.0 / 3.0);

        let alice = response.settlements.iter().find(|s| s.name == "Alice").unwrap();
        let bob = response.settlements.iter().find(|s| s.name == "Bob").unwrap();
        let charlie = response.settlements.iter().find(|s| s.name == "Charlie").unwrap();

        let share = 100.0 / 3.0;

        // Alice: paid 100, should pay share
        assert!((alice.balance - (100.0 - share)).abs() < 0.01);

        // Bob: will receive 50 from Charlie, should pay share
        assert!((bob.balance - (50.0 - share)).abs() < 0.01);

        // Charlie: owes 50 to Bob, should pay share
        assert!((charlie.balance - (-50.0 - share)).abs() < 0.01);
    }

    #[test]
    fn test_global_tip_percentage() {
        let people = vec![
            create_person(1, "Alice", 100.0, 1, 0.0, None),
            create_person(2, "Bob", 0.0, 1, 0.0, None),
        ];

        let request = CalculateRequest {
            people,
            include_sponsor: false,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 10.0, // 10% global tip
        };

        let response = calculate_split_internal(request);

        // Total: 100 + 10% tip = 110
        assert_eq!(response.total_spent, 100.0);
        assert!((response.total_tip - 10.0).abs() < 0.01); // Use approximate equality for floating point
        assert!((response.per_person_share - 55.0).abs() < 0.01); // Use approximate equality
    }
}
