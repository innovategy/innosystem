pub mod customer;
pub mod wallet;
pub mod job;
pub mod job_type;
pub mod reseller;
pub mod project;
pub mod runner;
pub mod wallet_transaction;
pub mod diesel;

// Re-export repository traits
pub use customer::CustomerRepository;
pub use wallet::WalletRepository;
pub use job::JobRepository;
pub use job_type::JobTypeRepository;
pub use reseller::ResellerRepository;
pub use project::ProjectRepository;
pub use runner::RunnerRepository;
pub use wallet_transaction::WalletTransactionRepository;

// Phase 1 in-memory implementations are removed in Phase 3

// Re-export diesel implementations
pub use diesel::{
    DieselJobTypeRepository,
    DieselCustomerRepository,
    DieselWalletRepository,
    DieselJobRepository,
    DieselResellerRepository,
    DieselProjectRepository,
    DieselRunnerRepository,
    DieselWalletTransactionRepository
};
