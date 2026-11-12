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

/// 추세를 걷은 계절 몫(고전적 분해의 '이동평균 대비 비율'): 각 달 값 ÷ 그 달을 가운데 둔 2×12 이동평균 - 1.
/// 매달 값이 꾸준히 내리는 품목(TV처럼)도 추세가 빠지므로, 남는 것은 그 달 특유의 높고 낮음이다. 반환: (달 번호 년*12+월-1, 몫)
pub fn detrended(months: &[(String, f64)]) -> Vec<(i64, f64)> {
    let key = |d: &str| -> Option<i64> { Some(d.get(..4)?.parse::<i64>().ok()? * 12 + d.get(5..7)?.parse::<i64>().ok()? - 1) };
    let v: Vec<(i64, f64)> = months.iter().filter_map(|(d, x)| Some((key(d)?, *x))).collect();
    let mut out = vec![];
    for i in 6..v.len().saturating_sub(6) {
        if v[i + 6].0 - v[i - 6].0 != 12 { continue; } // 빠진 달이 끼면 건너뛴다(2025년 10월)
        let ma = (0.5 * v[i - 6].1 + v[i - 5..=i + 5].iter().map(|p| p.1).sum::<f64>() + 0.5 * v[i + 6].1) / 12.0;
        out.push((v[i].0, v[i].1 / ma - 1.0));
    }
    out
}

/// 기준 달(base, 1~12)에서 그 뒤 몇 달(ahead)까지: 해마다 (그 달 몫 + 1) ÷ (기준 달 몫 + 1) - 1의 중앙값과 내려간 해의 비율.
/// 반환: 뒤 달마다 (몇 달 뒤, 중앙값, 내려간 비율, 해 수)
pub fn from_base(d: &[(i64, f64)], base: i64, ahead: i64) -> Vec<(i64, f64, f64, usize)> {
    let m: std::collections::HashMap<i64, f64> = d.iter().copied().collect();
    (1..=ahead).map(|h| {
        let ch: Vec<f64> = d.iter().filter(|(k, _)| k.rem_euclid(12) == base - 1).filter_map(|(k, b)| Some((m.get(&(k + h))? + 1.0) / (b + 1.0) - 1.0)).collect();
        (h, median(&ch).unwrap_or(0.0), ch.iter().filter(|&&x| x < 0.0).count() as f64 / ch.len().max(1) as f64, ch.len())
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

    #[test]
    fn detrend_removes_steady_decline() {
        // 매달 1%씩 내리고 11월만 3% 더 내렸다 12월에 되돌아오는 값: 추세를 걷으면 10월→11월만 약 -3%
        let mut m = vec![];
        let mut v = 100.0;
        for y in 2000..2006 {
            for k in 1..=12 {
                v *= 0.99;
                m.push((format!("{y}-{k:02}"), if k == 11 { v * 0.97 } else { v }));
            }
        }
        let d = detrended(&m);
        let f = from_base(&d, 10, 2);
        assert!((f[0].1 + 0.03).abs() < 0.005 && f[0].2 == 1.0, "{:?}", f);
        assert!(f[1].1.abs() < 0.005);
    }
}
