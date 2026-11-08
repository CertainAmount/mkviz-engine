//! fit: 한 계열을 다른 계열들로 설명하는 최소제곱 회귀(y = a + b1·x1 + …). 학습 기간에서 계수를 정하고
//! 모든 달에 '이 조건이면 보통 이 정도'(예상값)와 실제와의 차이(잔차)를 낸다. 소비자 심리 vs 실업률·물가가 첫 쓰임.

/// 계수 [a, b1, …]와 R². rows: 각 줄 [x1, x2, …], y: 같은 길이.
pub fn ols(rows: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, f64)> {
    let k = rows.first()?.len() + 1;
    if y.len() <= k { return None; }
    // 정규 방정식 (X'X) b = X'y 를 가우스 소거로(설명 변수 몇 개라 이것으로 충분)
    let mut m = vec![vec![0.0; k + 1]; k];
    for (r, &yi) in rows.iter().zip(y) {
        let x: Vec<f64> = std::iter::once(1.0).chain(r.iter().copied()).collect();
        for i in 0..k {
            for j in 0..k { m[i][j] += x[i] * x[j]; }
            m[i][k] += x[i] * yi;
        }
    }
    for c in 0..k {
        let p = (c..k).max_by(|&a, &b| m[a][c].abs().partial_cmp(&m[b][c].abs()).unwrap())?;
        m.swap(c, p);
        if m[c][c].abs() < 1e-12 { return None; }
        for r in 0..k {
            if r == c { continue; }
            let f = m[r][c] / m[c][c];
            for j in c..=k { m[r][j] -= f * m[c][j]; }
        }
    }
    let b: Vec<f64> = (0..k).map(|i| m[i][k] / m[i][i]).collect();
    let mean = y.iter().sum::<f64>() / y.len() as f64;
    let (mut ss, mut st) = (0.0, 0.0);
    for (r, &yi) in rows.iter().zip(y) {
        let e = yi - predict(&b, r);
        ss += e * e;
        st += (yi - mean).powi(2);
    }
    Some((b, 1.0 - ss / st))
}

pub fn predict(b: &[f64], x: &[f64]) -> f64 {
    b[0] + b[1..].iter().zip(x).map(|(c, v)| c * v).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_exact_line() {
        // y = 2 + 3·x1 - 1·x2
        let rows: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![2.0, 1.0], vec![3.0, 5.0], vec![0.0, 2.0], vec![4.0, 4.0]];
        let y: Vec<f64> = rows.iter().map(|r| 2.0 + 3.0 * r[0] - r[1]).collect();
        let (b, r2) = ols(&rows, &y).unwrap();
        assert!((b[0] - 2.0).abs() < 1e-9 && (b[1] - 3.0).abs() < 1e-9 && (b[2] + 1.0).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }
}
