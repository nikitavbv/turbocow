use colour::red;

pub fn print_intro() {
    println!(
        r#"   
     __             __                            
    / /___  _______/ /_  ____  _________ _      __
   / __/ / / / ___/ __ \/ __ \/ ___/ __ \ | /| / /
  / /_/ /_/ / /  / /_/ / /_/ / /__/ /_/ / |/ |/ / 
  \__/\__,_/_/  /_.___/\____/\___/\____/|__/|__/ "#
    );

    if cfg!(debug_assertions) {
        red!("\nWARNING: YOU ARE RUNNING IN DEBUG MODE. Keep in mind that everything is way slower than it should be.\n\n");
    }
}