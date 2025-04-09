pub mod customer;
pub mod wallet;
pub mod job;
pub mod job_type;
pub mod diesel;

// Re-export repository traits
pub use customer::CustomerRepository;
pub use wallet::WalletRepository;
pub use job::JobRepository;
pub use job_type::JobTypeRepository;

// Phase 1 in-memory implementations are removed in Phase 3

// Re-export diesel implementations
pub use diesel::{DieselJobTypeRepository, DieselCustomerRepository, DieselWalletRepository, DieselJobRepository};
