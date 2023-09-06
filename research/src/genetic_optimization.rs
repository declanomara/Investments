use std::{path::Path, error::Error};

use crate::backtesting::{Backtest, HistoricalPriceStream, AlphaModel};


struct GeneticOptimizer {
    population: Vec<Box<dyn AlphaModel>>,
    population_size: usize,
    mutation_rate: f32,

    data: HistoricalPriceStream,
    initial_capital: f64,
}

impl GeneticOptimizer {
    pub fn new<Model: AlphaModel>(population_size: usize, mutation_rate: f32, data: &Path) -> Result<Self, Box<dyn Error>> {
        let mut population: Vec<Box<dyn AlphaModel>> = Vec::new();
        let data = HistoricalPriceStream::new(data)?;
        for _ in 0..population_size {
            population.push(Model::random());
        }
        Ok(GeneticOptimizer {
            population,
            population_size,
            mutation_rate,
            data,
            initial_capital: 100_000.0, // TODO: make this configurable
        })
    }

    // TODO: Fix genetic algorithm optimization
    fn fitness(&self, model: Box<dyn AlphaModel>) -> f64 {
        let mut fitness = 0.0;
        // let mut backtest = Backtest::new(model, self.initial_capital);
        // backtest.run(&self.data);
        

        fitness
    }
}