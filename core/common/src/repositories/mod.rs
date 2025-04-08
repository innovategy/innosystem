pub mod customer;
pub mod wallet;
pub mod job;
pub mod job_type;
pub mod in_memory;
pub mod diesel;

// Re-export repository traits
pub use customer::CustomerRepository;
pub use wallet::WalletRepository;
pub use job::JobRepository;
pub use job_type::JobTypeRepository;

// Re-export in-memory implementations for Phase 1
pub use in_memory::{InMemoryCustomerRepository, InMemoryJobRepository, InMemoryJobTypeRepository, InMemoryWalletRepository};

// Re-export diesel implementations for Phase 2
pub use diesel::DieselJobTypeRepository;
