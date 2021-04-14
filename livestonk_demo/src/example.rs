use livestonk::Component;

pub trait Database {

    fn provide_data(&self) -> String;
}

pub struct Postgres {
}

impl Database for Postgres {
    
    fn provide_data(&self) -> String {
        "Hello from Postgres!".to_string()
    }
}

pub struct Mongo {
}

impl Database for Mongo {

    fn provide_data(&self) -> String {
        "Hello from Mongo!".to_string()
    }
}

pub trait WebController {

    fn process_request(&self);
}

#[derive(Component)]
pub struct APIController {
    pub database: Box<dyn Database>,
}

impl WebController for APIController {

    fn process_request(&self) {
        println!("Request is processed: {}", &self.database.provide_data())
    }
}