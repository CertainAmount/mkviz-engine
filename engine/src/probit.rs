//! 수익률 곡선으로 본 침체 확률: P(12개월 뒤 침체) = Φ(a + b·금리차). Estrella & Mishkin(1996/1998)과 NY Fed 모델의 꼴.
//! 계수는 남의 값을 옮기지 않고 우리 자료(FRED T10Y3M 월평균, USREC)로 직접 추정한다(최대우도, 뉴턴법). 표본 안 추정이라
//! 과거를 잘 맞힌 것처럼 보인다(METHODS.md A4·A6): 글에는 "이 모델로 보면"과 표본 기간·N을 함께 쓴다.

fn phi(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// 표준정규 누적분포(Abramowitz–Stegun 7.1.26 근사, 오차 < 1.5e-7).
pub fn cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs() / 2f64.sqrt());
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-(x * x) / 2.0).exp();
    if x >= 0.0 { 0.5 * (1.0 + y) } else { 0.5 * (1.0 - y) }
}

/// (a, b, 반복 수). x: 설명 변수, y: 0/1.
pub fn fit(x: &[f64], y: &[f64]) -> (f64, f64, usize) {
    let (mut a, mut b) = (0.0, 0.0);
    for it in 0..100 {
        let (mut g0, mut g1, mut h00, mut h01, mut h11) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for (&xi, &yi) in x.iter().zip(y) {
            let z = a + b * xi;
            let p = cdf(z).clamp(1e-10, 1.0 - 1e-10);
            let d = phi(z);
            let w = d * (yi - p) / (p * (1.0 - p));
            g0 += w;
            g1 += w * xi;
            // 기대 정보 행렬(피셔 스코어링)
            let i = d * d / (p * (1.0 - p));
            h00 += i;
            h01 += i * xi;
            h11 += i * xi * xi;
        }
        let det = h00 * h11 - h01 * h01;
        if det.abs() < 1e-12 {
            return (a, b, it);
        }
        let (da, db) = ((h11 * g0 - h01 * g1) / det, (h00 * g1 - h01 * g0) / det);
        a += da;
        b += db;
        if da.abs() + db.abs() < 1e-9 {
            return (a, b, it + 1);
        }
    }
    (a, b, 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdf_values() {
        assert!((cdf(0.0) - 0.5).abs() < 1e-7);
        assert!((cdf(1.96) - 0.975).abs() < 1e-4);
    }

    #[test]
    fn recovers_known_coefficients() {
        // 가짜 자료: x가 낮을수록 1이 많게 a=-0.5, b=-0.8의 확률을 결정적으로 흉내(구간마다 비율대로 1)
        let (mut x, mut y) = (vec![], vec![]);
        for k in 0..400 {
            let xi = -2.0 + 6.0 * (k as f64) / 400.0;
            let p = cdf(-0.5 - 0.8 * xi);
            let n = 400;
            let ones = (p * n as f64).round() as usize;
            for j in 0..n {
                x.push(xi);
                y.push(if j < ones { 1.0 } else { 0.0 });
            }
        }
        let (a, b, _) = fit(&x, &y);
        assert!((a + 0.5).abs() < 0.02 && (b + 0.8).abs() < 0.02, "{a} {b}");
    }
}
