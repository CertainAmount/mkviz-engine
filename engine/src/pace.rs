//! 가속 신호: 최근 몇 달 속도(연율)가 1년 변화율보다 일정 폭 이상 빠른 달마다, 그 1년 변화율이 n달 뒤 더 높아졌나.
//! 근원 물가처럼 느린 계열에서 '추세가 바뀌는 중인가'를 과거 같은 상황의 뒤로 잰다.

use crate::gapfill::{key, Months};

/// (달, 짧은 창 연율, 1년 변화율). 비율(0.01 = 1%)
pub fn rates(m: &Months, short: i64) -> Vec<(i64, f64, f64)> {
    m.iter()
        .filter_map(|(&k, &v)| {
            let s = m.get(&(k - short))?;
            let y = m.get(&(k - 12))?;
            Some((k, (v / s).powf(12.0 / short as f64) - 1.0, v / y - 1.0))
        })
        .collect()
}

/// 가속한 달(연율 - 1년 변화율 >= gap)마다 (달, 그때 1년 변화율, ahead달 뒤 1년 변화율)
pub fn after_accel(m: &Months, short: i64, gap: f64, ahead: i64, from: &str) -> Vec<(i64, f64, f64)> {
    let start = key(from).unwrap_or(0);
    let yoy: std::collections::HashMap<i64, f64> = rates(m, short).into_iter().map(|r| (r.0, r.2)).collect();
    rates(m, short)
        .into_iter()
        .filter(|r| r.0 >= start && r.1 - r.2 >= gap)
        .filter_map(|r| Some((r.0, r.2, *yoy.get(&(r.0 + ahead))?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accel_months_and_later_rate() {
        // 매달 0.1%씩 오르다 마지막 석 달 1%씩: 그 끝달은 가속, 1년 뒤 값이 있으면 짝을 낸다
        let mut m = Months::new();
        let mut v = 100.0;
        for k in 0..40 {
            v *= if (24..27).contains(&k) { 1.01 } else { 1.001 };
            m.insert(k, v);
        }
        let r = rates(&m, 3);
        let last = r.iter().find(|x| x.0 == 26).unwrap();
        assert!(last.1 > 0.12 && last.2 < 0.05);
        let a = after_accel(&m, 3, 0.01, 12, "0000-01");
        assert!(a.iter().any(|x| x.0 == 26) && a.iter().all(|x| x.0 + 12 <= 39));
    }
}
