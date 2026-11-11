//! 빠진 달 메우기: BLS가 권하는 앞뒤 달 지수의 기하평균(√(앞달 × 뒷달))으로 빠진 달을 메우고 n달 변화율을 낸다.
//! 그 방법이 과거에 얼마나 틀렸는지는, 값이 있는 모든 달을 '빠졌다고 치고' 같은 방법으로 메워 실제 변화율과 견줘 잰다.
//! 같은 달(예: 10월)만 모은 오차의 평균이 그 달 특유의 치우침(계절: 11월 할인으로 11월 지수가 내려간다)이다.

/// 달 번호(년*12 + 월-1) -> 지수
pub type Months = std::collections::BTreeMap<i64, f64>;

pub fn key(d: &str) -> Option<i64> {
    Some(d.get(..4)?.parse::<i64>().ok()? * 12 + d.get(5..7)?.parse::<i64>().ok()? - 1)
}

pub fn label(k: i64) -> String {
    format!("{}-{:02}", k.div_euclid(12), k.rem_euclid(12) + 1)
}

/// 앞뒤 달의 기하평균(BLS가 설명하는 근사). 둘 중 하나라도 없으면 None
pub fn fill(m: &Months, k: i64) -> Option<f64> {
    Some((m.get(&(k - 1))? * m.get(&(k + 1))?).sqrt())
}

/// 앞달에 직전 12달 상승률의 한 달 몫(복리를 걷은 12제곱근)을 곱한 값: 재무부가 물가연동국채(TIPS)에 쓰는 대체 규칙(31 CFR 356 부록 B)
pub fn trend(m: &Months, k: i64) -> Option<f64> {
    let (a, b) = (m.get(&(k - 1))?, m.get(&(k - 13))?);
    Some(a * (a / b).powf(1.0 / 12.0))
}

/// 값이 있는 달 k마다: (k, 메운 값으로 잰 lag달 뒤까지의 변화율 - 실제 변화율). 변화율은 비율(0.01 = 1%p)
pub fn backtest(m: &Months, lag: i64, from: i64, to: i64, fill: fn(&Months, i64) -> Option<f64>) -> Vec<(i64, f64)> {
    m.iter()
        .filter(|(k, _)| **k >= from && **k <= to)
        .filter_map(|(&k, &v)| {
            let later = m.get(&(k + lag))?;
            let f = fill(m, k)?;
            Some((k, (later / f - 1.0) - (later / v - 1.0)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_and_backtest() {
        // 9월 100, 10월 104(빠졌다고 칠 달), 11월 100, 이듬해 10월 110
        let mut m = Months::new();
        for (d, v) in [("2024-09", 100.0), ("2024-10", 104.0), ("2024-11", 100.0), ("2025-10", 110.0)] {
            m.insert(key(d).unwrap(), v);
        }
        assert!((fill(&m, key("2024-10").unwrap()).unwrap() - 100.0).abs() < 1e-9);
        let b = backtest(&m, 12, 0, i64::MAX, fill);
        assert_eq!(b.len(), 1);
        // 메우면 10% 상승, 실제는 110/104 - 1 = 5.77%: 메운 쪽이 4.23%p 높다
        assert!((b[0].1 - (0.10 - (110.0 / 104.0 - 1.0))).abs() < 1e-9);
        assert_eq!(label(key("2026-10").unwrap()), "2026-10");
        // 재무부 규칙: 2025-09 324.8, 2024-09 315.301 -> 325.604(2025년 10월 TIPS 지수와 같다)
        let mut t = Months::new();
        t.insert(key("2024-09").unwrap(), 315.301);
        t.insert(key("2025-09").unwrap(), 324.8);
        assert!((trend(&t, key("2025-10").unwrap()).unwrap() - 325.604).abs() < 1e-3);
    }
}
