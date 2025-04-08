// Define schema for Diesel
table! {
    job_types (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        processing_logic_id -> Text,
        processor_type -> Text,
        standard_cost_cents -> Integer,
        enabled -> Bool,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    jobs (id) {
        id -> Uuid,
        job_type_id -> Uuid,
        customer_id -> Uuid,
        status -> Text,
        cost_cents -> Integer,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
    }
}

table! {
    customers (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    wallets (id) {
        id -> Uuid,
        customer_id -> Uuid,
        balance_cents -> Integer,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    wallet_transactions (id) {
        id -> Uuid,
        wallet_id -> Uuid,
        amount_cents -> Integer,
        transaction_type -> Text,
        reference_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(
    job_types,
    jobs,
    customers,
    wallets,
    wallet_transactions,
);
