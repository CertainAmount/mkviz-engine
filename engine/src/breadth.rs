//! 폭(breadth): 여러 월간 지표 중 몇 개가 자기 최근 고점 아래로 내려와 있나. NBER가 침체 날짜를 정할 때 보는
//! 여섯 지표(고용, 실질소득, 실질소비, 실질 제조·도소매 판매, 산업생산)로 '지금 몇 개가 꺾였나'를 세고,
//! 지난 침체 시작 달에는 몇 개였는지와 견준다. 고점은 직전 window개월(그 달 포함)의 최고값, drop 이상 낮으면 '내려옴'.

use crate::series::Point;
use std::collections::HashMap;

/// 달(YYYY-MM)마다 내려온 지표 수와 각 지표의 고점 대비 비율(모든 지표가 있는 달만, 오래된 달부터)
pub fn counts(series: &[Vec<Point>], window: usize, drop: f64) -> Vec<(String, usize, Vec<f64>)> {
    let maps: Vec<HashMap<String, f64>> = series.iter().map(|s| s.iter().map(|p| (p.date[..7].to_string(), p.value)).collect()).collect();
    let mut months: Vec<String> = maps[0].keys().filter(|m| maps.iter().all(|x| x.contains_key(*m))).cloned().collect();
    months.sort();
    let mut out = Vec::new();
    for m in &months {
        let gaps: Vec<f64> = series.iter().zip(&maps).map(|(s, map)| {
            let k = s.iter().position(|p| &p.date[..7] == m).unwrap();
            let hi = s[k.saturating_sub(window.saturating_sub(1))..=k].iter().map(|p| p.value).fold(f64::MIN, f64::max);
            map[m] / hi - 1.0
        }).collect();
        let n = gaps.iter().filter(|g| **g <= -drop).count();
        out.push((m.clone(), n, gaps));
    }
    out
}

/// 사건(0/1 월간 계열)이 시작된 달들(직전 달이 0, 그 달이 1)
pub fn starts(ev: &[Point]) -> Vec<String> {
    ev.windows(2).filter(|w| w[0].value < 0.5 && w[1].value >= 0.5).map(|w| w[1].date[..7].to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::day_number;

    fn s(vals: &[(&str, f64)]) -> Vec<Point> {
        vals.iter().map(|(d, v)| Point { date: d.to_string(), day: day_number(d).unwrap(), value: *v }).collect()
    }

    #[test]
    fn counts_series_below_recent_high() {
        let a = s(&[("2020-01-01", 100.0), ("2020-02-01", 101.0), ("2020-03-01", 99.0), ("2020-04-01", 99.5)]);
        let b = s(&[("2020-01-01", 50.0), ("2020-02-01", 51.0), ("2020-03-01", 52.0), ("2020-04-01", 52.0)]);
        let c = counts(&[a, b], 3, 0.005);
        assert_eq!(c.iter().map(|r| r.1).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
        assert!((c[2].2[0] - (99.0 / 101.0 - 1.0)).abs() < 1e-12);
        // window 1이면 자기 자신이 고점이라 늘 0
        let a = s(&[("2020-01-01", 100.0), ("2020-02-01", 90.0)]);
        assert_eq!(counts(&[a], 1, 0.005)[1].1, 0);
    }

    #[test]
    fn starts_are_first_flagged_months() {
        let ev = s(&[("2001-02-01", 0.0), ("2001-03-01", 0.0), ("2001-04-01", 1.0), ("2001-05-01", 1.0), ("2001-12-01", 0.0), ("2008-01-01", 1.0)]);
        assert_eq!(starts(&ev), vec!["2001-04", "2008-01"]);
    }
}
