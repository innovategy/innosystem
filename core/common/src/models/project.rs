use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::diesel_schema::projects;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Project {
    pub fn new(
        customer_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            name,
            description,
            created_at: None,
            updated_at: None,
        }
    }
}

// For DB insertion with Diesel
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = projects)]
pub struct NewProject {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl From<Project> for NewProject {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            customer_id: project.customer_id,
            name: project.name,
            description: project.description,
        }
    }
}
