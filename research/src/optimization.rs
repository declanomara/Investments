use crate::backtesting::{self, backtest, BacktestReport};
use anyhow::Result;
use backtesting::{save_aggregate_report, EMAStrategy};
use quantlib::oanda::objects::Price;
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};
use rayon::prelude::*;

const NOISE_FACTOR: f64 = 0.25;
const GEN_SIZE: usize = 250;
const EXPECTED_RETURN: f64 = 0.01;
const TRADES_PER_WEEK: f64 = 5.0;
const ELITE_FRACTION: f64 = 0.05;

fn randomly_sample(first: f64, second: f64, rng: &mut impl Rng) -> f64 {
    // Return a random number between the two inputs, regardless of their order
    // first is not necessarily less than second
    if rng.gen_bool(0.5) {
        first + (second - first) * rng.gen::<f64>()
    } else {
        second + (first - second) * rng.gen::<f64>()
    }
}

fn create_new_generation(
    parents: &[&EMAStrategy],
    initial_price: f32,
    gen_size: usize,
) -> Vec<EMAStrategy> {
    let num_elites = parents.len();

    (0..gen_size)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::rngs::SmallRng::from_entropy();
            let parent1 = parents[rng.gen_range(0..num_elites)];
            let parent2 = parents[rng.gen_range(0..num_elites)];

            // The method of crossover is to pick a random weight in the range of the parents' weights
            // Furthermore, we add a random noise factor to the weights to encourage exploration
            // We also must ensure that the fast EMA weight is greater than the slow EMA weight

            // First generate the slow EMA weight
            let slow_weight =
                randomly_sample(parent1.slow_ema_weight, parent2.slow_ema_weight, &mut rng)
                    * (1.0 + rng.gen_range(-NOISE_FACTOR..NOISE_FACTOR));

            let parent1_fast_weight_ratio = parent1.fast_ema_weight / parent1.slow_ema_weight;
            let parent2_fast_weight_ratio = parent2.fast_ema_weight / parent2.slow_ema_weight;

            let fast_weight_ratio = randomly_sample(
                parent1_fast_weight_ratio,
                parent2_fast_weight_ratio,
                &mut rng,
            ) * (1.0 + rng.gen_range(-NOISE_FACTOR..NOISE_FACTOR));

            // Now calculate the fast weight
            let fast_weight = slow_weight * fast_weight_ratio;

            EMAStrategy::new(
                initial_price as f64,
                initial_price as f64,
                fast_weight,
                slow_weight,
            )
        })
        .collect()
}

// Helper function to output strategy performance and parameters
fn print_strategy_performance(strategy_tuple: &(EMAStrategy, Vec<BacktestReport>, f64)) {
    let num_trades = strategy_tuple.1.iter().map(|b| b.num_trades).sum::<u32>();
    println!(
        "Fitness: {:.2} | Num Trades: {} | Fast EMA Weight: {} | Slow EMA Weight: {}",
        strategy_tuple.2,
        num_trades,
        strategy_tuple.0.fast_ema_weight,
        strategy_tuple.0.slow_ema_weight
    );
}

// Helper function to find average frequency of data points
fn average_frequency(data_set: &Vec<Price>) -> f64 {
    let mut sum = 0.0;
    for i in 1..data_set.len() {
        sum += (data_set[i].time - data_set[i - 1].time) as f64;
    }
    sum / (data_set.len() - 1) as f64
}

fn perform_backtests(
    strategies: &mut Vec<EMAStrategy>,
    data_sets: &Vec<Vec<Price>>,
) -> Vec<Vec<BacktestReport>> {
    // Perform backtests in parallel
    // Each strategy will be tested on each data set
    // The results will be a vector of vectors of backtest reports
    // The outer vector will contain the results for each strategy
    // The inner vector will contain the results for each data set
    // This will allow us to evaluate the fitness of each strategy across multiple data sets

    strategies
        .par_iter_mut()
        .map(|s| {
            data_sets
                .iter()
                .map(|d| backtest(d, s).unwrap())
                .collect::<Vec<BacktestReport>>()
        })
        .collect()
}

fn evaluate_fitness(results: &Vec<Vec<BacktestReport>>) -> Vec<f64> {
    // Results contains a vector of vectors of backtest reports
    // Each inner vector is a set of backtests for a single strategy
    // To evaluate the fitness of a strategy, we will use the average profit of all backtests
    let fitness: Vec<f64> = results
        .iter()
        .map(|r| {
            let base_fitness = r
                .iter()
                .map(|b| (b.final_value - b.initial_balance) / b.initial_balance)
                .sum::<f64>()
                / r.len() as f64;

            // We will penalize strategies that trade too frequently
            // let num_trades = r.iter().map(|b| b.num_trades).sum::<u32>() as f32;
            // let num_weeks = r.len() as f32;
            // base_fitness * (TRADES_PER_WEEK * num_weeks) / num_trades
            base_fitness
        })
        .collect();

    // Our benchmark is 1% profit weekly, so we will scale the fitness accordingly
    fitness.iter().map(|f| f / EXPECTED_RETURN as f64).collect()
}

pub fn optimize_ema(data_sets: &Vec<Vec<Price>>) -> Result<()> {
    // Print the average frequency of data points
    println!(
        "Average time between data points: {:.2}ms",
        average_frequency(&data_sets[0])
    );

    // Initialize a population of strategies
    let mut strategies: Vec<EMAStrategy> = Vec::new();
    let bid = data_sets[0][0].bid;
    let initial_slow_weight = 0.01 / average_frequency(&data_sets[0]);
    for _ in 0..GEN_SIZE {
        let slow_weight: f64 = rand::thread_rng().gen_range(0.0..initial_slow_weight);
        let fast_weight: f64 = slow_weight * rand::thread_rng().gen_range(1.0..10.0);
        let strategy = EMAStrategy::new(bid as f64, bid as f64, fast_weight, slow_weight);
        strategies.push(strategy);
    }

    println!("Running optimization");
    for generation_number in 0..10 {
        // Sanity check: make sure our population size doesn't change
        assert_eq!(strategies.len(), GEN_SIZE);

        // Run backtests for each strategy
        println!(
            "Running backtests for generation {} (population: {})",
            generation_number, GEN_SIZE
        );
        let results: Vec<Vec<BacktestReport>> = perform_backtests(&mut strategies, &data_sets);

        // Evaluate the fitness of each strategy
        println!("Evaluating fitness of generation {}", generation_number);
        let fitness: Vec<f64> = evaluate_fitness(&results);

        // Combine the strategies, results, and fitness into a single population vector to couple them together
        let mut population: Vec<(EMAStrategy, Vec<BacktestReport>, f64)> = strategies
            .iter()
            .cloned()
            .zip(results.into_iter())
            .zip(fitness.into_iter())
            .map(|((s, r), f)| (s, r, f))
            .collect();

        // Sort the strategies by fitness
        population.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Print the best strategy's performance
        print_strategy_performance(&population[0]);

        // Print the worst strategy's performance
        print_strategy_performance(&population[population.len() - 1]);

        // Save the best strategy's performance to a CSV for human inspection
        println!("Saving best strategy's performance to CSV");
        save_aggregate_report(
            &mut population[0].1,
            &format!("backtest-results/generation_{}.csv", generation_number),
        )?;

        // Select the top 10% of strategies as parents
        let num_elites: usize = (GEN_SIZE as f64 * ELITE_FRACTION).ceil() as usize;
        let elites: Vec<&EMAStrategy> = population.iter().take(num_elites).map(|i| &i.0).collect();

        // Print the avg, min, and max fitness of the elites for this generation
        // We will have to take num_elites from the population again to reference the fitness
        let avg_fitness: f64 =
            population.iter().take(num_elites).map(|i| i.2).sum::<f64>() / num_elites as f64;
        let min_fitness: f64 = population
            .iter()
            .take(num_elites)
            .map(|i| i.2)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_fitness: f64 = population
            .iter()
            .take(num_elites)
            .map(|i| i.2)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        println!(
            "Generation {} Parents: avg fitness: {:.2}, min fitness: {:.2}, max fitness: {:.2}",
            generation_number + 1,
            avg_fitness,
            min_fitness,
            max_fitness
        );

        // Create a new generation of strategies
        println!("Creating new generation of strategies");
        let new_gen = create_new_generation(&elites, bid, GEN_SIZE);

        strategies = new_gen;
    }

    Ok(())
}
