mod optimization;

fn test_func(x: f64, y: f64) -> f64 {
    (x - 1.0).powi(2) + (y - 2.0).powi(2)
}

fn fitness(arr: [f64; 2]) -> f64 {
    test_func(arr[0], arr[1])
}

fn main() {
    let initial = [100.0, 100.0];
    let result = optimization::optimize(initial, fitness, 10.0, 10.0);
    println!("Result: {:?}", result);
}
