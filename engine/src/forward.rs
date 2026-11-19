//! 그 뒤에 무슨 일이: 주어진 날짜들 이후 N일 뒤 변화율(예: 10% 하락선을 넘은 날 이후 6개월).

use crate::series::Point;

/// start 이후 처음 나오는 점과, 그로부터 days일 이상 지난 첫 점의 변화율. 자료가 모자라면 None.
pub fn after(pts: &[Point], start_day: i64, days: i64) -> Option<f64> {
    let i = pts.iter().position(|p| p.day >= start_day)?;
    let j = pts[i..].iter().position(|p| p.day >= pts[i].day + days)? + i;
    Some(pts[j].value / pts[i].value - 1.0)
}

/// after와 같되 비율이 아니라 수준 차이(금리처럼 %p가 뜻을 갖는 값).
pub fn after_diff(pts: &[Point], start_day: i64, days: i64) -> Option<f64> {
    let i = pts.iter().position(|p| p.day >= start_day)?;
    let j = pts[i..].iter().position(|p| p.day >= pts[i].day + days)? + i;
    Some(pts[j].value - pts[i].value)
}

/// start 이후 days일 안에서 가장 낮았던 때의 변화율(도중 최저점: 오르기 전에 얼마나 빠졌나).
pub fn worst(pts: &[Point], start_day: i64, days: i64) -> Option<f64> {
    let i = pts.iter().position(|p| p.day >= start_day)?;
    if pts.last()?.day < pts[i].day + days {
        return None;
    }
    // 끝은 after()가 쓰는 날(start+days 이후 첫 관측)까지 넣는다: 2019-03-22 + 1년이 주말이라 1년 값(3/23)이 도중 최저보다 낮게 나왔다(2026-11-18 검증)
    let end = pts[i..].iter().find(|p| p.day >= pts[i].day + days).map_or(pts[i].day + days, |p| p.day);
    pts[i..].iter().take_while(|p| p.day <= end).map(|p| p.value / pts[i].value - 1.0).reduce(f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::{day_number, parse};

    #[test]
    fn forward_change() {
        let p = parse("d,v\n2020-01-01,100\n2020-01-10,90\n2020-02-01,120\n");
        assert!((after(&p, day_number("2020-01-01").unwrap(), 30).unwrap() - 0.2).abs() < 1e-9);
        assert!(after(&p, day_number("2020-01-10").unwrap(), 90).is_none());
        assert!((worst(&p, day_number("2020-01-01").unwrap(), 30).unwrap() + 0.1).abs() < 1e-9);
    }
}

/// after_diff와 같되 기준을 사건일 '이전이거나 같은 날'의 마지막 관측으로(주간 조사처럼 사건일이 관측 사이에 있을 때, 사건 전후 그림과 같은 기준, 2026-11-19)
pub fn after_diff_prior(pts: &[Point], start_day: i64, days: i64) -> Option<f64> {
    let base = pts.iter().filter(|p| p.day <= start_day).last()?;
    let j = pts.iter().position(|p| p.day >= start_day + days)?;
    Some(pts[j].value - base.value)
}
