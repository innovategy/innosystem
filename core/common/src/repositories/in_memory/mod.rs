pub mod customer;
pub mod job;
pub mod job_type;
pub mod wallet;

// Re-export repositories
pub use customer::InMemoryCustomerRepository;
pub use job::InMemoryJobRepository;
pub use job_type::InMemoryJobTypeRepository;
pub use wallet::InMemoryWalletRepository;
