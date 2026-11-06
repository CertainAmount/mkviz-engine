//! 계절성: 해마다 각 달의 월평균이 그해 평균보다 몇 % 높았나(계절 지수, 고전적 분해의 비율 방식을 단순화).
//! 12달이 모두 있는 해만 쓴다. 달마다 중앙값과 '그해 평균보다 높았던 해의 비율'을 낸다.

use crate::stats::median;

pub struct Year {
    pub year: i32,
    pub dev: [f64; 12],
}

/// months: ("YYYY-MM", 월평균) 시간순
pub fn table(months: &[(String, f64)]) -> Vec<Year> {
    let mut out: Vec<Year> = vec![];
    let mut i = 0;
    while i < months.len() {
        let y: i32 = months[i].0[..4].parse().unwrap_or(0);
        let ys: Vec<&(String, f64)> = months[i..].iter().take_while(|m| m.0[..4].parse::<i32>().ok() == Some(y)).collect();
        if ys.len() == 12 {
            let mean = ys.iter().map(|m| m.1).sum::<f64>() / 12.0;
            let mut dev = [0.0; 12];
            for m in &ys {
                let k: usize = m.0[5..7].parse::<usize>().unwrap_or(1) - 1;
                dev[k] = m.1 / mean - 1.0;
            }
            out.push(Year { year: y, dev });
        }
        i += ys.len().max(1);
    }
    out
}

/// 달마다 (중앙값, 그해 평균보다 높았던 해의 비율)
pub fn by_month(t: &[Year]) -> Vec<(f64, f64)> {
    (0..12).map(|k| {
        let v: Vec<f64> = t.iter().map(|y| y.dev[k]).collect();
        (median(&v).unwrap_or(0.0), v.iter().filter(|&&x| x > 0.0).count() as f64 / v.len().max(1) as f64)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summer_peak_shows_up() {
        let mut m = vec![];
        for y in 2000..2003 {
            for k in 1..=12 {
                m.push((format!("{y}-{k:02}"), if k == 6 { 110.0 } else { 100.0 }));
            }
        }
        m.push(("2003-01".into(), 100.0)); // 덜 찬 해는 뺀다
        let t = table(&m);
        assert_eq!(t.len(), 3);
        let b = by_month(&t);
        assert!(b[5].0 > 0.08 && b[5].1 == 1.0 && b[0].0 < 0.0);
    }
}
