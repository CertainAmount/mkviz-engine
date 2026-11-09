//! 첫 발표와 나중 추정의 차이(수정치): 고용처럼 몇 달 뒤 다시 고쳐지는 지표에서 '첫 숫자를 얼마나 믿을 수 있나'.
//! 수정치 = 나중 추정 - 첫 발표. 분포(평균·절댓값 중앙값·10~90% 범위), 해마다 평균, 부호가 뒤집힌 달.

pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub mean_abs: f64,
    pub median_abs: f64,
    pub p10: f64,
    pub p90: f64,
    pub share_down: f64,
    pub sign_flips: usize,
}

/// 정렬된 값의 q 분위(선형 보간).
pub fn quantile(sorted: &[f64], q: f64) -> f64 {
    let i = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (i.floor() as usize, i.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (i - lo as f64)
}

/// rows: (날짜, 첫 발표, 나중 추정).
pub fn summarize(rows: &[(String, f64, f64)]) -> Option<Summary> {
    if rows.len() < 2 {
        return None;
    }
    let mut r: Vec<f64> = rows.iter().map(|x| x.2 - x.1).collect();
    let n = r.len();
    let mut a: Vec<f64> = r.iter().map(|v| v.abs()).collect();
    r.sort_by(|x, y| x.partial_cmp(y).unwrap());
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    Some(Summary {
        n,
        mean: r.iter().sum::<f64>() / n as f64,
        mean_abs: a.iter().sum::<f64>() / n as f64,
        median_abs: quantile(&a, 0.5),
        p10: quantile(&r, 0.1),
        p90: quantile(&r, 0.9),
        share_down: r.iter().filter(|v| **v < 0.0).count() as f64 / n as f64,
        sign_flips: rows.iter().filter(|x| x.1 * x.2 < 0.0).count(),
    })
}

/// 해마다 평균 수정치(그해 달이 min_months 이상일 때만): (연도, 평균, 달 수).
pub fn by_year(rows: &[(String, f64, f64)], min_months: usize) -> Vec<(i32, f64, usize)> {
    let mut out: Vec<(i32, f64, usize)> = Vec::new();
    for (d, f, l) in rows {
        let y: i32 = d[..4].parse().unwrap_or(0);
        match out.last_mut() {
            Some(last) if last.0 == y => { last.1 += l - f; last.2 += 1; }
            _ => out.push((y, l - f, 1)),
        }
    }
    out.into_iter().filter(|x| x.2 >= min_months).map(|(y, s, k)| (y, s / k as f64, k)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_stats() {
        let rows: Vec<(String, f64, f64)> = [(100.0, 80.0), (50.0, 70.0), (20.0, -10.0), (10.0, 10.0)]
            .iter().enumerate().map(|(i, (f, l))| (format!("2020-{:02}-01", i + 1), *f, *l)).collect();
        let s = summarize(&rows).unwrap();
        assert_eq!(s.n, 4);
        assert!((s.mean - (-20.0 + 20.0 - 30.0 + 0.0) / 4.0).abs() < 1e-9);
        assert!((s.median_abs - 20.0).abs() < 1e-9);
        assert_eq!(s.sign_flips, 1);
        assert!((s.share_down - 0.5).abs() < 1e-9);
        assert_eq!(by_year(&rows, 4), vec![(2020, -7.5, 4)]);
        assert!(by_year(&rows, 5).is_empty());
    }
}
