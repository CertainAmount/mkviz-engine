//! 널리 쓰이는 경기 지표를 공식 정의대로 계산한다(METHODS.md A4).
//! Sahm 규칙(Sahm 2019, FRED SAHMCURRENT 정의): 실업률 3개월 평균 - 직전 12개월의 3개월 평균 중 최저. 0.50%p 이상이면 신호.

use crate::series::Point;

/// (날짜, Sahm 값). 월간 실업률(%)을 받는다. 앞 14달은 계산할 수 없어 뺀다.
pub fn sahm(u: &[Point]) -> Vec<(String, f64)> {
    let avg: Vec<f64> = (0..u.len()).map(|i| if i >= 2 { (u[i].value + u[i - 1].value + u[i - 2].value) / 3.0 } else { f64::NAN }).collect();
    (14..u.len())
        .map(|i| {
            let low = avg[i - 12..i].iter().cloned().fold(f64::INFINITY, f64::min);
            (u[i].date.clone(), avg[i] - low)
        })
        .collect()
}

/// 신호가 처음 켜진 달들(직전 달은 기준 미만, 이번 달 이상).
pub fn triggers(v: &[(String, f64)], line: f64) -> Vec<String> {
    v.windows(2).filter(|w| w[0].1 < line && w[1].1 >= line).map(|w| w[1].0.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn sahm_rule() {
        let mut s = String::from("d,v\n");
        for k in 0..24 {
            let u = if k < 18 { 4.0 } else { 4.0 + 0.3 * (k - 17) as f64 };
            s += &format!("{}-{:02}-01,{u}\n", 2000 + k / 12, k % 12 + 1);
        }
        let v = sahm(&parse(&s));
        // 마지막 달: (5.8 + 5.5 + 5.2) / 3 = 5.5, 직전 12달의 3개월 평균 최저 4.0 -> 1.5
        assert!((v.last().unwrap().1 - 1.5).abs() < 1e-9);
        let t = triggers(&v, 0.5);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0], "2001-09-01"); // (4.9 + 4.6 + 4.3) / 3 = 4.6, 최저 4.0 -> 0.6 (전달은 0.3)
    }
}
