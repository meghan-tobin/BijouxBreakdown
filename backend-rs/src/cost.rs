const LENGTH_UNITS: &[&str] = &["yards", "feet", "in", "cms"];

fn to_inches(amount: f64, unit: &str) -> f64 {
    match unit {
        "yards" => amount * 36.0,
        "feet"  => amount * 12.0,
        "in"    => amount,
        "cms"   => amount / 2.54,
        _       => amount,
    }
}

fn from_inches(inches: f64, unit: &str) -> f64 {
    match unit {
        "yards" => inches / 36.0,
        "feet"  => inches / 12.0,
        "in"    => inches,
        "cms"   => inches * 2.54,
        _       => inches,
    }
}

pub fn convert_amount(amount: f64, from_unit: &str, to_unit: &str) -> f64 {
    if from_unit == to_unit {
        return amount;
    }
    if !LENGTH_UNITS.contains(&from_unit) || !LENGTH_UNITS.contains(&to_unit) {
        return amount;
    }
    from_inches(to_inches(amount, from_unit), to_unit)
}

pub struct SupplyRef<'a> {
    pub id: &'a str,
    pub cost: f64,
    pub quantity: f64,
    pub unit: &'a str,
}

pub struct UsageRef<'a> {
    pub supply_id: &'a str,
    pub amount: f64,
    pub unit: Option<&'a str>,
}

pub struct TimeCostRef {
    pub duration: f64,
    pub duration_unit: String,
    pub rate: f64,
}

pub fn calc_material_cost(usages: &[UsageRef], supplies: &[SupplyRef]) -> f64 {
    usages.iter().fold(0.0, |sum, u| {
        if let Some(sup) = supplies.iter().find(|s| s.id == u.supply_id) {
            let from = u.unit.unwrap_or(sup.unit);
            let converted = convert_amount(u.amount, from, sup.unit);
            sum + (sup.cost / sup.quantity) * converted
        } else {
            sum
        }
    })
}

pub fn calc_time_cost(tc: &TimeCostRef, amount: f64, unit: &str) -> f64 {
    let hours = if unit == "hours" { amount } else { amount / 60.0 };
    hours * tc.rate
}

pub fn calc_tc_total(tc: &TimeCostRef) -> f64 {
    let hours = if tc.duration_unit == "hours" { tc.duration } else { tc.duration / 60.0 };
    hours * tc.rate
}
