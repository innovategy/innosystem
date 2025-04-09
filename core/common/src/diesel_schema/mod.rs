// Define schema for Diesel
use diesel::prelude::*;

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
        project_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
    }
}

table! {
    resellers (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        api_key -> Text,
        active -> Bool,
        commission_rate -> Integer,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    customers (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        reseller_id -> Nullable<Uuid>,
        api_key -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    projects (id) {
        id -> Uuid,
        customer_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    runners (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        status -> Text,
        compatible_job_types -> Array<Text>,
        last_heartbeat -> Nullable<Timestamp>,
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
        customer_id -> Uuid,
        reference_id -> Nullable<Uuid>,
        description -> Nullable<Text>,
        job_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
    }
}

table! {
    runner_job_type_compatibility (runner_id, job_type_id) {
        runner_id -> Uuid,
        job_type_id -> Uuid,
        created_at -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(
    job_types,
    jobs,
    customers,
    wallets,
    wallet_transactions,
    resellers,
    projects,
    runners,
    runner_job_type_compatibility,
);
