pub struct HiveNanoCount {
    pub hive_id: String,
    pub nano_bees: f64,   // nano-bee units
    pub nano_var: f64,    // variance in nano-bee^2
}

pub struct RegionNanoCount {
    pub region_id: String,
    pub nano_bees_managed: f64,
    pub nano_bees_wild: f64,
    pub nano_var_managed: f64,
    pub nano_var_wild: f64,
}

pub struct GlobalNanoCount {
    pub timestamp: String,
    pub nano_bees_mean: f64,
    pub nano_bees_std: f64,
}

pub fn aggregate_global_nano(
    regions: &[RegionNanoCount]
) -> GlobalNanoCount {
    let mut mean_nb = 0.0_f64;
    let mut var_nb  = 0.0_f64;

    for r in regions {
        let m = r.nano_bees_managed + r.nano_bees_wild;
        let v = r.nano_var_managed + r.nano_var_wild;
        mean_nb += m;
        var_nb  += v;
    }

    GlobalNanoCount {
        timestamp: "2026-02-08T00:00:00Z".into(),
        nano_bees_mean: mean_nb,
        nano_bees_std: var_nb.sqrt(),
    }
}
