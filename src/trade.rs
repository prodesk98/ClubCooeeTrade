use colored::Colorize;

pub struct Trade {
    history: Vec<f64>,
    current: f64,
}

impl Trade {
    pub fn new(history: Vec<f64>, current: f64) -> Self {
        Trade {
            history,
            current,
        }
    }

    fn remove_outliers_iqr(&mut self) {
        if self.history.len() < 4 {
            return;
        }

        let mut sorted_history = self.history.clone();
        sorted_history.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = sorted_history[sorted_history.len() / 4];
        let q3 = sorted_history[sorted_history.len() * 3 / 4];
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        self.history.retain(|&x| x >= lower_bound && x <= upper_bound);
    }

    fn moving_average(&self, period: usize) -> f64 {
        let len = self.history.len();
        if len < period {
            return 0.0;
        }

        let sum: f64 = self.history[len - period..].iter().sum();
        sum / period as f64
    }

    fn std(&self) -> f64 {
        let recent_history: Vec<f64> = self.history.iter().rev().take(10).cloned().collect();
        let len = self.history.len();
        if len < 2 {
            return 0.0;
        }

        let mean = recent_history.iter().sum::<f64>() / len as f64;
        let variance = recent_history.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / len as f64;
        variance.sqrt()
    }

    fn rsi(&self, period: usize) -> f64 {
        let len = self.history.len();
        if len < period {
            return 0.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;
        for i in 1..len {
            let diff = self.history[i] - self.history[i - 1];
            if diff > 0.0 {
                gains += diff;
            } else {
                losses += diff.abs();
            }
        }

        let rs = gains / losses;
        100.0 - (100.0 / (1.0 + rs))
    }

    pub fn resale(&self, desired_profit_margin: f64) -> f64 {
        let resale_price = self.current * (1.0 + desired_profit_margin / 100.0);
        resale_price.ceil()
    }

    pub fn strategy(&mut self, desired_profit_margin: f64) -> bool {
        self.remove_outliers_iqr();
        let ma = self.moving_average(10);
        let std = self.std();
        let rsi = self.rsi(10);
        let resale_price = self.resale(desired_profit_margin);

        eprintln!("std: {:.2}, rsi: {:.2}, ma: {:.2}, history: {:?}", std, rsi, ma, self.history);

        if
        resale_price < ma && self.history.len() >= 10 && self.current >= 190.0 &&
            self.current <= 1990.0 && rsi >= 50.0 && rsi < 70.0 &&
            std <= 600.0
        {
            eprintln!("{} current: {:.2}, ma: {:.2}, resale: {:.2}, history: {:?}, std: {:.2}", "[+]".green().bold(),
              self.current,
              ma,
              resale_price,
              self.history,
              std
            );
            true
        } else {
            false
        }
    }
}