use quantlib::models::alpha_model::{create_model_from_config, read_config, AlphaModel};

use optimization::{create_fitness_function, pso_optimize};

mod optimization;

fn main() {
    let config_path = "config.json";
    let config = read_config(config_path);
    let model: Box<dyn AlphaModel> = create_model_from_config(&config);
    let fitness = create_fitness_function(&config);
    let initial = model.to_vec();
    let result = pso_optimize(initial, fitness, 10.0, 10.0);
    println!("Result: {:?}", result);
}
