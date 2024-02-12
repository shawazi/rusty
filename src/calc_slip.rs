pub async fn calculate_slippage(
    amount_in: f64,   // Amount of input token you are trading
    reserve_in: f64,  // Reserve of the input token in the pool
    reserve_out: f64, // Reserve of the output token in the pool
) -> f64 {
    // Price without trade impact
    let price_initial = reserve_out / reserve_in;

    // New reserves after trade
    let new_reserve_in = reserve_in + amount_in;
    let new_reserve_out = reserve_in * reserve_out / new_reserve_in; // Derived from x * y = k

    // Price with trade impact
    let price_impact = new_reserve_out / new_reserve_in;

    // Slippage
    let slippage = (price_impact - price_initial) / price_initial;

    slippage.abs() * 100.0 // Return slippage as a percentage
}
