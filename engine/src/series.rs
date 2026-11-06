//! 시계열: data/macro/*.csv(date,close,...)를 읽는다. 날짜는 일수(1970-01-01 기준)로 바꿔 계산한다.

#[derive(Clone, Debug)]
pub struct Point {
    pub date: String,
    pub day: i64,
    pub value: f64,
}

/// "YYYY-MM-DD" -> 1970-01-01 기준 일수(그레고리력, Howard Hinnant 알고리즘).
pub fn day_number(date: &str) -> Option<i64> {
    let mut it = date.get(..10)?.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// CSV 본문 -> 점 목록. 첫 줄은 머리글, 첫 열 날짜, 둘째 열 값. 값이 없거나 숫자가 아니면 건너뛴다.
pub fn parse(text: &str) -> Vec<Point> {
    let mut out: Vec<Point> = text
        .lines()
        .skip(1)
        .filter_map(|l| {
            let mut c = l.split(',');
            let date = c.next()?.trim().to_string();
            let value: f64 = c.next()?.trim().parse().ok()?;
            Some(Point { day: day_number(&date)?, date, value })
        })
        .collect();
    out.sort_by_key(|p| p.day);
    out
}

pub fn load(path: &str) -> Result<Vec<Point>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let pts = parse(&text);
    if pts.len() < 2 {
        return Err(format!("{path}: 자료가 2개 미만"));
    }
    Ok(pts)
}

/// 잇기(chain-linking): 새 계열이 시작하기 전 구간을 옛 계열로 채우되, 이음 시점의 옛 값이 같은 기간 새 계열 평균과 같도록 비율을 맞춘다.
/// 지수 기준이 다른 두 계열(예: 넓은 달러 지수 1973~2019와 2006~)을 하나의 긴 흐름으로 본다. 겹치는 구간은 새 계열을 쓴다.
pub fn splice(new: &[Point], old: &[Point]) -> Vec<Point> {
    let Some(first) = new.first() else { return old.to_vec() };
    let Some(anchor) = old.iter().filter(|p| p.day <= first.day).last() else { return new.to_vec() };
    // 하루 값이 아니라 이음 달의 평균으로 맞춘다(하루 값 하나로 맞추면 결과가 이음 날의 잡음에 흔들린다)
    let next = old.iter().find(|p| p.day > anchor.day).map_or(i64::MAX, |p| p.day);
    let month: Vec<f64> = new.iter().take_while(|p| p.day < next).map(|p| p.value).collect();
    let k = month.iter().sum::<f64>() / month.len() as f64 / anchor.value;
    let mut out: Vec<Point> = old.iter().filter(|p| p.day < first.day).map(|p| Point { value: p.value * k, ..p.clone() }).collect();
    out.extend_from_slice(new);
    out
}

/// 이동 평균(n개, 끝을 포함한 뒤쪽 창). 월별 요금처럼 계절이 큰 계열의 "최근 12달 평균 대 그 전 12달"에 쓴다.
pub fn rolling_mean(pts: &[Point], n: usize) -> Vec<Point> {
    if n <= 1 { return pts.to_vec(); }
    pts.windows(n).map(|w| Point { value: w.iter().map(|p| p.value).sum::<f64>() / n as f64, ..w[n - 1].clone() }).collect()
}

/// 물가 조정: 각 값을 그 시점(그 날 이전 가장 가까운 달)의 물가지수로 나누고 마지막 물가지수로 다시 곱해 '오늘 달러'로 맞춘다.
pub fn deflate(pts: &[Point], cpi: &[Point]) -> Vec<Point> {
    let Some(last) = cpi.last() else { return pts.to_vec() };
    pts.iter().filter_map(|p| {
        let c = cpi.iter().filter(|c| c.day <= p.day).last()?;
        Some(Point { value: p.value / c.value * last.value, ..p.clone() })
    }).collect()
}

/// 집값 부담: 각 시점 집값에서 계약금을 뺀 대출의 월 상환액(그날 이전 가장 가까운 고정금리, 원리금 균등)이
/// 그날 이전 가장 가까운 연 소득의 1/12에서 차지하는 몫. 소득은 연 자료라 다음 발표 전까지 그대로 쓴다.
pub fn afford(price: &[Point], rate: &[Point], income: &[Point], down: f64, years: f64) -> Vec<Point> {
    let at = |s: &[Point], d: i64| s.iter().filter(|p| p.day <= d).last().map(|p| p.value);
    price.iter().filter_map(|p| {
        let (r, inc) = (at(rate, p.day)? / 1200.0, at(income, p.day)?);
        let loan = p.value * (1.0 - down);
        let pay = if r == 0.0 { loan / (years * 12.0) } else { loan * r / (1.0 - (1.0 + r).powf(-years * 12.0)) };
        Some(Point { value: pay / (inc / 12.0), ..p.clone() })
    }).collect()
}

/// 다른 계열로 나눈 비율(그날 이전 가장 가까운 값). 예: 집값 / 연 소득 = 소득 몇 년 치.
pub fn per(pts: &[Point], by: &[Point]) -> Vec<Point> {
    pts.iter().filter_map(|p| {
        let b = by.iter().filter(|b| b.day <= p.day).last()?.value;
        (b != 0.0).then(|| Point { value: p.value / b, ..p.clone() })
    }).collect()
}

/// lag 행 전 대비 변화율(월간 자료 12 = 1년 전 대비).
pub fn change_over(pts: &[Point], lag: usize) -> Vec<Point> {
    // 달력으로 lag달 전(같은 달의 마지막 값)과 비교한다. 행 수로 세면 빠진 달(2025년 10월 CPI: 정부 셧다운) 하나에 한 달씩 밀렸다(2026-11-06 주장 검증)
    let ym = |d: &str| -> Option<i64> { Some(d.get(..4)?.parse::<i64>().ok()? * 12 + d.get(5..7)?.parse::<i64>().ok()? - 1) };
    let mut by_month: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for p in pts {
        if let Some(k) = ym(&p.date) { by_month.insert(k, p.value); }
    }
    pts.iter()
        .filter_map(|p| {
            let base = *by_month.get(&(ym(&p.date)? - lag as i64))?;
            if base == 0.0 { return None; }
            Some(Point { value: p.value / base - 1.0, ..p.clone() })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_and_parse() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("2000-03-01").unwrap() - day_number("2000-02-28").unwrap(), 2);
        let p = parse("date,close\n2020-01-02,10\n2020-01-01,.\n2019-12-31,9,x\n");
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].date, "2019-12-31");
    }

    #[test]
    fn yoy() {
        // 2020-10이 빠져도 2021-01은 2020-01과 비교된다
        let p = parse("d,v\n2020-01-01,100\n2020-02-01,110\n2020-11-01,115\n2021-01-01,120\n2021-02-01,121\n");
        let c = change_over(&p, 12);
        assert_eq!(c.len(), 2);
        assert!((c[0].value - 0.2).abs() < 1e-9 && (c[1].value - 0.1).abs() < 1e-9);
    }

    #[test]
    fn splice_links_at_start_of_new() {
        let old = parse("d,v\n2000-01-01,50\n2000-02-01,100\n2000-03-01,110\n");
        let new = parse("d,v\n2000-02-15,200\n2000-02-20,200\n2000-03-15,220\n");
        let s = splice(&new, &old);
        assert_eq!(s.len(), 5);
        assert!((s[0].value - 100.0).abs() < 1e-9 && (s[1].value - 200.0).abs() < 1e-9 && s[2].value == 200.0);
    }

    #[test]
    fn afford_share_of_income() {
        // 집값 125,000, 계약금 20% -> 대출 100,000, 연 6% 30년 -> 월 599.55, 연 소득 71,946 -> 월 5,995.5의 10%
        let p = parse("d,v\n2020-04-01,125000\n");
        let r = parse("d,v\n2020-03-01,6\n");
        let i = parse("d,v\n2020-01-01,71946\n");
        let a = afford(&p, &r, &i, 0.2, 30.0);
        assert!((a[0].value - 0.1).abs() < 1e-3);
        assert!(afford(&p, &r, &parse("d,v\n2021-01-01,1\n2022-01-01,1\n"), 0.2, 30.0).is_empty());
    }

    #[test]
    fn per_divides_by_latest_earlier() {
        let a = per(&parse("d,v\n2020-06-01,10\n2019-06-01,4\n"), &parse("d,v\n2020-01-01,2\n"));
        assert_eq!(a.len(), 1);
        assert!((a[0].value - 5.0).abs() < 1e-9);
    }

    #[test]
    fn deflate_to_latest_dollars() {
        let p = parse("d,v\n2000-01-15,100\n2001-01-15,100\n");
        let c = parse("d,v\n2000-01-01,50\n2001-01-01,100\n");
        let d = deflate(&p, &c);
        assert!((d[0].value - 200.0).abs() < 1e-9 && (d[1].value - 100.0).abs() < 1e-9);
    }
}
