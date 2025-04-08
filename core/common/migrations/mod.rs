use diesel_migrations::embed_migrations;

embed_migrations!("core/common/migrations");

pub fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    
    let conn = PgConnection::establish(database_url)?;
    
    embedded_migrations::run(&conn)?;
    
    Ok(())
}
