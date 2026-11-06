//! 과거 비슷한 시기(analogs): 지금을 여러 지표의 표준화 값 묶음으로 보고, 가장 가까운 과거 달 k개와 그 뒤에 일어난 일.
//! Farmer & Sidorowich 1987의 최근접 이웃 꼴(METHODS.md A3). 같은 사례가 여러 번 잡히지 않게 서로 exclude달 이상 떨어진 달만 고르고,
//! 지금과 가까운 exclude달도 뺀다. 평균이 아니라 사례 전부를 낸다.

/// rows: 달마다 특징 벡터(같은 길이), 마지막 행이 지금. 돌려주는 것: (행 번호, 거리) 가까운 순.
pub fn nearest(rows: &[Vec<f64>], k: usize, exclude: usize) -> Vec<(usize, f64)> {
    let n = rows.len();
    let dims = rows[0].len();
    // 표준화(전체 표본 평균·표준편차)
    let mean: Vec<f64> = (0..dims).map(|d| rows.iter().map(|r| r[d]).sum::<f64>() / n as f64).collect();
    let sd: Vec<f64> = (0..dims)
        .map(|d| (rows.iter().map(|r| (r[d] - mean[d]).powi(2)).sum::<f64>() / n as f64).sqrt().max(1e-12))
        .collect();
    let z = |r: &Vec<f64>| -> Vec<f64> { (0..dims).map(|d| (r[d] - mean[d]) / sd[d]).collect() };
    let now = z(&rows[n - 1]);
    let mut d: Vec<(usize, f64)> = rows[..n.saturating_sub(exclude)]
        .iter()
        .enumerate()
        .map(|(i, r)| (i, z(r).iter().zip(&now).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()))
        .collect();
    d.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut out: Vec<(usize, f64)> = Vec::new();
    for (i, dist) in d {
        if out.iter().all(|(j, _)| i.abs_diff(*j) >= exclude) {
            out.push((i, dist));
            if out.len() == k {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_separated_neighbors() {
        let mut rows: Vec<Vec<f64>> = (0..100).map(|i| vec![(i as f64 / 10.0).sin(), (i as f64) / 100.0]).collect();
        rows.push(vec![(37.0f64 / 10.0).sin(), 0.37]); // 지금 = 37번째 달과 같다
        let n = nearest(&rows, 3, 12);
        assert_eq!(n[0].0, 37);
        assert!(n.windows(2).all(|w| w[0].0.abs_diff(w[1].0) >= 12));
    }
}
