# Livestonk

This is a minimal DI framework, which performs all the work completely in runtime thanks to powerful macro system in Rust. Runtime overhead is minimal or non-existent.

Forgot to bind a dependecy? No problem, your crate will not compile.

## Usage

Let's say you have a trait for Database:
```
pub trait Database {
    fn provide_data(&self) -> String;
}
```

and an implementation for this trait:

```
pub struct Postgres {
}

impl Database for Postgres {
    
    fn provide_data(&self) -> String {
        "Hello from Postgres!".to_string()
    }
}
```

You can bind this implementation to trait using `livestonk::bind_to_instance` macro:
```
livestonk::bind_to_instance!(dyn Database, Postgres {});
```

you can place this binding anywhere, but we advice to do this in `main.rs` or in `bindings.rs`.

Now you can resolve and use implementation anywhere you need like this:
```
let database: Box<dyn Database> = Livestonk::resolve();
let data = database.provide_data();
```

Let's say now you have another trait and implementation which depends on Database:
```
pub trait WebController {

    fn process_request(&self);
}

pub struct APIController {
    pub database: Box<dyn Database>,
}

impl WebController for APIController {

    fn process_request(&self) {
        println!("Request is processed: {}", &self.database.provide_data())
    }
}
```

Livestonk can inject dependencies for you! Just add `#[derive(Component)]`:
```
#[derive(Component)]
pub struct APIController {
    pub database: Box<dyn Database>,
}
```

And of course you can use this component just like any other dependency:

```
let controller: Box<APIController> = Livestonk::resolve();
```

Use `livestonk::bind` to bind component to trait:
```
livestonk::bind!(dyn WebController, APIController);
```
