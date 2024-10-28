use quantlib::models::alpha_model::{AlphaModel, Config, DiamondHands, Random};
use rand::{thread_rng, Rng};

// General particle swarm optimizer for n-dimensional function
pub fn pso_optimize(
    initial: Vec<f64>,
    fitness: impl Fn(&Vec<f64>) -> f64,
    perturbation: f64,
    speed: f64,
) -> Vec<f64> {
    const PARTICLE_COUNT: usize = 100;
    const INERTIA: f64 = 0.5;
    const COGNITIVE: f64 = 1.0;
    const SOCIAL: f64 = 1.0;
    const MINIMUM_SPEED: f64 = 0.01;

    // Initialize the particles as a cloud around the initial point
    let mut local_best_positions: Vec<Vec<f64>> = vec![initial.clone(); PARTICLE_COUNT];
    let mut local_best_fitnesses: Vec<f64> = vec![fitness(&initial.clone()); PARTICLE_COUNT];
    let mut particles: Vec<Vec<f64>> = vec![initial.clone(); PARTICLE_COUNT];
    for particle in &mut particles {
        for coordinate in particle.iter_mut() {
            // Perturb by up to `perturbation` in either direction
            *coordinate += rand::random::<f64>() * perturbation - perturbation / 2.0;
        }
    }

    // Initialize the global best position to the initial position
    let mut global_best_position = initial.clone();
    let mut global_best_fitness = fitness(&initial);

    // Generate a random velocity for each particle in the cloud with a magnitude of `speed`
    let mut velocities = vec![vec![0.0; initial.len()]; PARTICLE_COUNT];
    for velocity in &mut velocities {
        for mut component in velocity.iter_mut() {
            *component = rand::random::<f64>() * 2.0 - 1.0;
        }

        // Normalize the velocity to have a magnitude of `speed`
        let magnitude = (velocity.iter().map(|x| x.powi(2)).sum::<f64>()).sqrt();
        for component in velocity.iter_mut() {
            *component *= speed / magnitude;
        }
    }

    // Now we can start the optimization loop
    let mut iteration = 0;
    while iteration < 5000 {
        iteration += 1;

        for (((mut position, mut velocity), mut local_best_fitness), mut local_best_position) in
            particles
                .iter_mut()
                .zip(velocities.iter_mut())
                .zip(local_best_fitnesses.iter_mut())
                .zip(local_best_positions.iter_mut())
        {
            // Update the position of the particle
            for (coordinate, mut component) in position.iter_mut().zip(velocity.iter_mut()) {
                *coordinate += *component;
            }

            // Update the local best position if the fitness is better
            let current_fitness = fitness(position);
            if current_fitness < *local_best_fitness {
                *local_best_position = position.clone();
                *local_best_fitness = current_fitness;
            }

            // Update the global best position if the fitness is better
            if current_fitness < global_best_fitness {
                global_best_position = position.clone();
                global_best_fitness = current_fitness;
            }

            // Update the velocity of the particle
            let cognitive_force = local_best_position
                .iter()
                .zip(position.iter())
                .map(|(a, b)| a - b)
                .collect::<Vec<f64>>();
            let social_force = global_best_position
                .iter()
                .zip(position.iter())
                .map(|(a, b)| a - b)
                .collect::<Vec<f64>>();
            for ((mut component, cognitive), social) in velocity
                .iter_mut()
                .zip(cognitive_force.iter())
                .zip(social_force.iter())
            {
                *component = INERTIA * *component + COGNITIVE * cognitive + SOCIAL * social;
            }

            // Ensure the velocity is at least `MINIMUM_SPEED`
            let magnitude = (velocity.iter().map(|x| x.powi(2)).sum::<f64>()).sqrt();
            if magnitude < MINIMUM_SPEED {
                for component in velocity.iter_mut() {
                    *component *= MINIMUM_SPEED / magnitude;
                }
            }
        }

        println!(
            "Global best position: {:?}, fitness: {}",
            global_best_position, global_best_fitness
        );
    }

    println!("Optimization finished in {} iterations", iteration);

    global_best_position
}

pub fn create_fitness_function(config: &Config) -> impl Fn(&Vec<f64>) -> f64 {
    match config.model_name.as_str() {
        "random" => {
            move |params: &Vec<f64>| {
                let mut model = Random {
                    buy_threshold: params[0],
                    sell_threshold: params[1],
                    rng: thread_rng(),
                };

                let backtest_results = backtest_model(&mut model);

                // backtest_results.sharpe_ratio()
                (params[0] - 1.0).powi(2) + (params[1] - 2.0).powi(2)
            }
        }

        "diamond_hands" => move |params: &Vec<f64>| {
            let mut model = DiamondHands;
            let backtest_results = backtest_model(&mut model);

            // backtest_results.sharpe_ratio()
            (params[0] - 5.0).powi(2) + (params[1] - 1.0).powi(2)
        },
        _ => panic!("Unknown model name"),
    }
}

pub fn backtest_model(model: &mut impl AlphaModel) -> f64 {
    0.0
}
