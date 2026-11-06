//! 작은 통계 도구.

pub fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = s.len();
    Some(if n % 2 == 1 { s[n / 2] } else { (s[n / 2 - 1] + s[n / 2]) / 2.0 })
}

/// x보다 작거나 같은 값의 비율(0~1). 지금 값이 역사 속 어디쯤인지.
pub fn rank(v: &[f64], x: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().filter(|&&y| y <= x).count() as f64 / v.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_rank() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(rank(&[1.0, 2.0, 3.0, 4.0], 2.0), 0.5);
    }
}

/// 중앙값의 붓스트랩 구간(백분위, 시드 고정: 같은 입력이면 같은 답, METHODS.md A6). (낮은 쪽, 높은 쪽)
pub fn median_ci(v: &[f64], level: f64, reps: usize, seed: u64) -> Option<(f64, f64)> {
    if v.len() < 3 {
        return None;
    }
    let mut s = seed.max(1);
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let mut meds: Vec<f64> = (0..reps)
        .map(|_| median(&(0..v.len()).map(|_| v[(next() % v.len() as u64) as usize]).collect::<Vec<_>>()).unwrap())
        .collect();
    meds.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let a = ((1.0 - level) / 2.0 * reps as f64) as usize;
    Some((meds[a], meds[(reps - 1 - a).min(reps - 1)]))
}

#[cfg(test)]
mod ci_tests {
    use super::*;

    #[test]
    fn ci_brackets_median_and_is_repeatable() {
        let v: Vec<f64> = (1..=21).map(|x| x as f64).collect();
        let (lo, hi) = median_ci(&v, 0.9, 2000, 7).unwrap();
        assert!(lo <= 11.0 && hi >= 11.0 && lo >= 5.0 && hi <= 17.0);
        assert_eq!(median_ci(&v, 0.9, 2000, 7), Some((lo, hi)));
    }
}
