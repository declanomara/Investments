// General particle swarm optimizer for n-dimensional function
pub fn optimize<const N: usize>(
    initial: [f64; N],
    fitness: impl Fn([f64; N]) -> f64,
    perturbation: f64,
    speed: f64,
) -> [f64; N] {
    const PARTICLE_COUNT: usize = 100;
    const INERTIA: f64 = 0.5;
    const COGNITIVE: f64 = 1.0;
    const SOCIAL: f64 = 1.0;
    const MINIMUM_SPEED: f64 = 0.01;

    // Initialize the particles as a cloud around the initial point
    let mut particles: Vec<[f64; N]> = vec![initial; PARTICLE_COUNT];
    for particle in &mut particles {
        for i in 0..N {
            particle[i] += perturbation * rand::random::<f64>();
        }
    }

    // Initialize the local best positions for each particle to the initial position
    let mut local_best_positions = vec![initial; PARTICLE_COUNT];
    let mut local_best_fitnesses = vec![fitness(initial); PARTICLE_COUNT];

    // Initialize the global best position to the initial position
    let mut global_best_position = initial;
    let mut global_best_fitness = fitness(initial);

    // Generate a random velocity for each particle in the cloud with a magnitude of `speed`
    let mut velocities = vec![[0.0; N]; PARTICLE_COUNT];
    for velocity in &mut velocities {
        for i in 0..N {
            velocity[i] = rand::random::<f64>();
        }

        let magnitude = (velocity.iter().map(|x| x.powi(2)).sum::<f64>()).sqrt();
        for i in 0..N {
            velocity[i] *= speed / magnitude;
        }
    }

    // Now we can start the optimization loop
    let mut iteration = 0;
    while global_best_fitness > 0.0001 {
        iteration += 1;
        for i in 0..PARTICLE_COUNT {
            // Update the position of the particle
            for j in 0..N {
                particles[i][j] += velocities[i][j];
            }

            // Update the local best position if the fitness is better
            let current_fitness = fitness(particles[i]);
            if current_fitness < local_best_fitnesses[i] {
                local_best_positions[i] = particles[i];
                local_best_fitnesses[i] = current_fitness;
            }

            // Update the global best position if the fitness is better
            if current_fitness < global_best_fitness {
                global_best_position = particles[i];
                global_best_fitness = current_fitness;
            }

            // Update the velocity of the particle
            // velocities[i] = velocities[i] * inertia + cognitive * (local_best_positions[i] - particles[i]) + social * (global_best_position - particles[i])
            // magnitude of velocity cannot be less than MINIMUM_SPEED
            for j in 0..N {
                velocities[i][j] = INERTIA * velocities[i][j]
                    + COGNITIVE * (local_best_positions[i][j] - particles[i][j])
                    + SOCIAL * (global_best_position[j] - particles[i][j]);
            }

            let magnitude = (velocities[i].iter().map(|x| x.powi(2)).sum::<f64>()).sqrt();
            if magnitude < MINIMUM_SPEED {
                for j in 0..N {
                    velocities[i][j] *= MINIMUM_SPEED / magnitude;
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
