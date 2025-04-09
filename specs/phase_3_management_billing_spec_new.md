# Phase 3: Entity Management & Billing Integration

## Overview

Phase 3 focuses on building out APIs for managing Resellers, Customers, and Projects, integrating job costing with the Wallet system using transactions, and implementing basic runner discovery and registration. This phase will be broken down into smaller sub-phases to ensure incremental, compilable progress at each step.

## Architectural Principles

1. **API-First Design**: Implement RESTful APIs with proper authentication and authorization
2. **Transaction Safety**: Ensure billing operations are atomic and consistent
3. **Entity Relationship Management**: Maintain proper relationships between business entities
4. **Runner Discovery**: Implement a reliable mechanism for runner registration and heartbeat monitoring
5. **Extensible Billing**: Design the billing system to support future pricing models

## Database Schema Additions

The primary tables for Phase 3 include:

- `resellers` (new)
- `projects` (new)
- `runners` (new)
- `wallet_transactions` (new)
- `customers` (modifications)
- `jobs` (modifications)

## Sub-Phases

### Phase 3.1: Entity Model Extension

#### Objectives
- Define new Diesel models for added entities
- Modify existing models to support new relationships
- Create database migrations for new tables and modifications

#### Tasks
1. **3.1.1: Reseller and Project Models**
   - Define `Reseller` and `Project` models with proper Diesel attributes
   - Create migrations for `resellers` and `projects` tables
   - Modify `Customer` model to include reseller relationship

2. **3.1.2: Runner Model**
   - Define `Runner` model with capability tracking
   - Create migration for `runners` table
   - Implement serialization for job type compatibility list

3. **3.1.3: Wallet Transaction Model**
   - Define `WalletTransaction` model for billing records
   - Create migration for `wallet_transactions` table
   - Implement transaction type enum and conversion logic

### Phase 3.2: Repository Implementation

#### Objectives
- Implement repositories for new entity models
- Update existing repositories to support new relationships
- Ensure proper transaction handling for financial operations

#### Tasks
1. **3.2.1: Reseller and Project Repositories**
   - Implement `ResellerRepository` with CRUD and API key lookup
   - Implement `ProjectRepository` with CRUD and customer-specific queries
   - Update `CustomerRepository` to handle reseller relationship

2. **3.2.2: Runner Repository**
   - Implement `RunnerRepository` with registration and heartbeat functionality
   - Add methods for listing active and compatible runners
   - Implement status tracking and update mechanisms

3. **3.2.3: Wallet Transaction Repository**
   - Implement `WalletTransactionRepository` for transaction recording
   - Modify `WalletRepository` to create transaction records during balance changes
   - Ensure atomicity for financial operations

### Phase 3.3: API Layer Implementation

#### Objectives
- Implement RESTful API endpoints for entity management
- Create authentication middleware for API key validation
- Support proper authorization for resource access

#### Tasks
1. **3.3.1: Authentication Middleware**
   - Implement middleware for API key validation
   - Support different authorization levels (Admin, Reseller, Customer)
   - Add header parsing and validation logic

2. **3.3.2: Reseller and Customer Endpoints**
   - Implement CRUD endpoints for Reseller management
   - Enhance Customer endpoints with reseller relationship
   - Add API key generation functionality

3. **3.3.3: Project and Runner Endpoints**
   - Implement CRUD endpoints for Project management
   - Create runner registration and discovery endpoints
   - Add authorization checks for resource access

### Phase 3.4: Billing Integration

#### Objectives
- Integrate job processing with billing
- Implement transaction recording for job costs
- Enable funds management operations

#### Tasks
1. **3.4.1: Job Cost Calculation**
   - Implement final cost calculation for completed jobs
   - Add cost recording to job processing workflow
   - Support dynamic cost factors (placeholder for future)

2. **3.4.2: Wallet Transaction Integration**
   - Connect job completion with wallet transactions
   - Implement atomic balance updates with transaction records
   - Add refund and credit operations

3. **3.4.3: Runner Health and Compatibility**
   - Implement heartbeat monitoring system
   - Add job-runner compatibility verification
   - Implement health status tracking and reporting

## Implementation Guidelines

### Model Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]  
#[diesel(table_name = schema::resellers)]
pub struct Reseller {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub active: bool,
    pub commission_rate: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::resellers)]
pub struct NewReseller {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub active: bool,
    pub commission_rate: BigDecimal,
}
```

### Transaction Type Implementation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub enum TransactionType {
    Charge,
    Refund,
    Credit,
}

impl ToSql<Text, Pg> for TransactionType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match self {
            TransactionType::Charge => "charge",
            TransactionType::Refund => "refund",
            TransactionType::Credit => "credit",
        };
        <String as ToSql<Text, Pg>>::to_sql(&s.to_string(), out)
    }
}

impl FromSql<Text, Pg> for TransactionType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        match s.as_str() {
            "charge" => Ok(TransactionType::Charge),
            "refund" => Ok(TransactionType::Refund),
            "credit" => Ok(TransactionType::Credit),
            _ => {
                let err_msg = format!("Unrecognized transaction type: {}", s);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, err_msg)))
            }
        }
    }
}
```

### Repository Implementation

```rust
#[async_trait]
impl ResellerRepository for DieselResellerRepository {
    async fn create(&self, new_reseller: NewReseller) -> Result<Reseller> {
        let mut conn = get_connection(&self.pool)?;
        
        diesel::insert_into(resellers)
            .values(&new_reseller)
            .get_result(&mut conn)
            .map_err(|e| {
                Error::Database(e.to_string())
            })
    }
    
    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<Reseller>> {
        let mut conn = get_connection(&self.pool)?;
        
        resellers
            .filter(api_key.eq(api_key))
            .filter(active.eq(true))
            .first::<Reseller>(&mut conn)
            .optional()
            .map_err(|e| {
                Error::Database(e.to_string())
            })
    }
    
    // Other repository methods
    // ...
}
```

### API Implementation

```rust
async fn create_reseller(
    State(app_state): State<AppState>,
    Extension(admin): Extension<AdminUser>,  // From auth middleware
    Json(payload): Json<CreateResellerRequest>,
) -> Result<Json<Reseller>, ApiError> {
    let api_key = generate_api_key();
    
    let new_reseller = NewReseller {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        api_key,
        active: true,
        commission_rate: payload.commission_rate,
    };
    
    let reseller = app_state.reseller_repository
        .create(new_reseller)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        
    Ok(Json(reseller))
}
```

### Runner Registration and Heartbeat

```rust
async fn register_runner(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRunnerRequest>,
) -> Result<Json<Runner>, ApiError> {
    let runner_id = payload.id.unwrap_or_else(|| Uuid::new_v4());
    
    let runner = app_state.runner_repository
        .register(NewRunner {
            id: runner_id,
            name: payload.name,
            description: payload.description,
            status: "active".to_string(),
            compatible_job_types: payload.compatible_job_types,
            last_heartbeat: Some(Utc::now()),
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        
    Ok(Json(runner))
}
```

## Git Branching Strategy

To maintain a stable codebase and ensure quality throughout the development process, we'll follow a strict branching strategy:

1. **Feature Branches**: Create a new Git branch for each specific sub-phase
   - Branch naming convention: `phase-3.x.y/feature-description`
   - Example: `phase-3.1.1/reseller-model-definition`

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

## Common Pitfalls to Avoid

1. **API Key Security**: Don't expose API keys in logs or responses
2. **Transaction Atomicity**: Ensure financial operations are atomic
3. **Race Conditions**: Handle concurrent access to wallet balances properly
4. **Authorization Leaks**: Verify authorization for every API request
5. **Runner Stale State**: Properly handle runner heartbeat failures
6. **Schema Mismatches**: Ensure model fields match database schema exactly

## Testing Strategy

1. **Unit Tests**: Test individual repository methods and API handlers
2. **Integration Tests**: Test API endpoints with authenticated requests
3. **Transaction Tests**: Verify billing operations maintain consistency
4. **Auth Tests**: Test authorization across different user types
5. **Error Handling**: Test error scenarios and proper error responses

## Conclusion

Phase 3 builds upon the foundation laid in Phase 2, adding entity management capabilities and integrating the billing system. By following a structured approach with clear sub-phases, we ensure that the implementation remains manageable and of high quality. The resulting system will provide a complete platform for managing resellers, customers, projects, and billing, setting the stage for future enhancements in subsequent phases.
