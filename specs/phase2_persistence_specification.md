# Phase 2: Persistence Specification

## Overview

Phase 2 involves implementing proper persistence for the Innosystem application, transitioning from in-memory repositories to database-backed implementations using Diesel ORM. This phase will be broken down into smaller sub-phases to ensure incremental, compilable progress at each step.

## Architectural Principles

1. **Repository Pattern**: Maintain the separation between business logic and data access logic
2. **Type Safety**: Ensure proper type definitions and conversions between application and database models
3. **Error Handling**: Implement consistent error handling throughout the persistence layer
4. **Async Operations**: Support asynchronous database operations for better performance
5. **Testing**: Each module should have appropriate unit and integration tests

## Database Schema

The primary tables for Phase 2 include:

- `job_types`
- `jobs`
- `customers`
- `wallets`
- `wallet_transactions`

## Sub-Phases

### Phase 2.1: Model Definition and Schema Alignment

#### Objectives
- Define Diesel models that align with the database schema
- Implement proper serialization/deserialization for complex types
- Ensure models implement necessary Diesel traits

#### Tasks
1. **2.1.1: Basic Model Definition**
   - Define all models with proper Diesel attributes
   - Ensure fields match the database schema exactly
   - Add necessary Diesel traits (`Queryable`, `Insertable`, etc.)

2. **2.1.2: Custom Type Implementation**
   - Implement `ToSql` and `FromSql` for custom types like `ProcessorType`
   - Add proper error handling for custom type conversions

3. **2.1.3: Schema Validation**
   - Validate models against the actual database schema
   - Fix any discrepancies between models and schema

### Phase 2.2: Repository Implementation

#### Objectives
- Implement Diesel-backed repositories for each model
- Ensure proper database connection management
- Maintain compatibility with the existing repository traits

#### Tasks
1. **2.2.1: Basic CRUD Operations**
   - Implement create, read, update, delete operations for each repository
   - Ensure proper error handling for database operations

2. **2.2.2: Query Implementation**
   - Implement more complex query operations (filtering, sorting, etc.)
   - Optimize queries for performance

3. **2.2.3: Transaction Support**
   - Implement transaction support for operations that require atomicity
   - Handle rollbacks and commits properly

### Phase 2.3: Migration System

#### Objectives
- Implement a database migration system
- Ensure forward and backward compatibility
- Support database versioning

#### Tasks
1. **2.3.1: Migration Framework**
   - Set up Diesel migrations framework
   - Define initial migration files

2. **2.3.2: Seed Data**
   - Implement seed data for development and testing
   - Ensure idempotent seed operations

### Phase 2.4: Integration and Testing

#### Objectives
- Integrate Diesel repositories with the application
- Write comprehensive tests for all persistence operations
- Validate functionality across different environments

#### Tasks
1. **2.4.1: Repository Integration**
   - Replace in-memory repositories with Diesel implementations in the application
   - Handle the transition smoothly with minimal code changes in business logic

2. **2.4.2: Test Implementation**
   - Write unit tests for all repository methods
   - Implement integration tests for database operations
   - Set up test database environment

3. **2.4.3: Performance Testing**
   - Test performance of database operations
   - Optimize slow queries

## Implementation Guidelines

### Model Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = schema::job_types)]
pub struct JobType {
    pub id: Uuid,
    pub name: String,
    // Fields must exactly match database schema
    // ...
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::job_types)]
pub struct NewJobType {
    pub id: Uuid,
    pub name: String,
    // Fields for insertion
    // ...
}
```

### Custom Types

```rust
impl ToSql<Text, Pg> for ProcessorType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = self.to_string();
        <String as ToSql<Text, Pg>>::to_sql(&s, out)
    }
}

impl FromSql<Text, Pg> for ProcessorType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        ProcessorType::from_str(&s).ok_or_else(|| {
            // Proper error handling
            let err_msg = format!("Unrecognized processor type: {}", s);
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, err_msg))
        })
    }
}
```

### Repository Implementation

```rust
#[async_trait]
impl JobTypeRepository for DieselJobTypeRepository {
    async fn create(&self, new_job_type: NewJobType) -> Result<JobType> {
        let mut conn = get_connection(&self.pool)?;
        
        diesel::insert_into(job_types)
            .values(&new_job_type)
            .get_result(&mut conn)
            .map_err(|e| {
                // Proper error handling
                Error::Database(e.to_string())
            })
    }
    
    // Other repository methods
    // ...
}
```

## Git Branching Strategy

To maintain a stable codebase and ensure quality throughout the development process, we'll follow a strict branching strategy:

1. **Feature Branches**: Create a new Git branch for each specific change or sub-phase
   - Branch naming convention: `phase-2.x.y/feature-description`
   - Example: `phase-2.1.1/job-type-model-definition`

2. **Compile-First Policy**: Ensure the code compiles successfully before committing changes
   - Run `cargo check` and `cargo build` before each commit
   - Fix any compilation errors immediately

3. **Pull Request Workflow**:
   - Create a pull request for each completed feature branch
   - Include clear descriptions of changes and references to the spec document
   - Request code reviews from team members

4. **Merge Requirements**:
   - All changes must have passing builds before merging
   - Run the full test suite before merging
   - Merge only when CI/CD pipeline is green
   - Squash commits if necessary for a clean history

5. **Integration Testing**:
   - After merging, verify the integration with existing features
   - Run the application to ensure functionality works as expected

This approach ensures that the main branch remains stable and each change is isolated, tested, and verified before integration.

## Common Pitfalls to Avoid

1. **Type Mismatches**: Ensure database types match Rust types
2. **Missing Fields**: Verify all required fields are present in models
3. **Improper Error Handling**: Handle all possible database errors properly
4. **SQL Injection**: Use parameterized queries to prevent SQL injection
5. **Connection Leaks**: Ensure database connections are properly closed/returned to pool
6. **Missing Diesel Attributes**: Make sure all models have proper Diesel attributes

## Testing Strategy

1. **Unit Tests**: Test individual repository methods in isolation
2. **Integration Tests**: Test repository methods with a real database
3. **Migration Tests**: Verify migrations work properly
4. **Edge Cases**: Test error handling and edge cases

## Conclusion

By following this specification and breaking down the implementation into smaller sub-phases, we ensure that each step of the development process is manageable, testable, and can compile successfully. This incremental approach allows for better tracking of progress and easier debugging of issues as they arise.
