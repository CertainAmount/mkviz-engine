//! lead: 신호 구간(예: 수익률 곡선 역전)이 시작된 뒤 사건(예: 경기침체)이 시작되기까지 걸린 시간.
//! 신호 구간은 기준 이하(또는 이상)로 이어진 날들을 gap일 안에 다시 들어오면 하나로 묶는다(하루짜리 역전이 수십 개로 쪼개지지 않게).
//! 사이에 사건이 시작됐으면 묶지 않고, 사건 도중의 날은 신호로 치지 않는다.
//! 사건 시작은 사건 계열(USREC 0/1)이 0.5 아래에서 위로 넘은 날. 아직 사건이 없으면 지금까지 걸린 시간만 낸다.

use crate::series::Point;

pub struct Lead {
    pub start: String,
    pub end: String,
    pub days: i64,              // 신호 구간 길이
    pub event: Option<String>,  // 신호 시작 뒤 첫 사건 시작일
    pub lead_days: i64,         // 신호 시작 → 사건 시작(없으면 → 자료 마지막 날)
}

pub fn leads(sig: &[Point], th: f64, below: bool, gap: i64, ev: &[Point]) -> Vec<Lead> {
    // 사건 도중(침체 중)의 날은 신호로 치지 않는다: 1980년 침체 중에도 이어진 역전이 '0개월 뒤 침체'로 잡혔다
    let during = |d: i64| ev.iter().take_while(|p| p.day <= d).last().is_some_and(|p| p.value >= 0.5);
    let hit = |p: &Point| (if below { p.value <= th } else { p.value >= th }) && !during(p.day);
    let starts: Vec<&Point> = ev.windows(2).filter(|w| w[0].value < 0.5 && w[1].value >= 0.5).map(|w| &w[1]).collect();
    let last = sig.last().map_or(0, |p| p.day);
    let mut out: Vec<Lead> = Vec::new();
    let mut i = 0;
    while i < sig.len() {
        if !hit(&sig[i]) { i += 1; continue; }
        let s = i;
        let mut e = i;
        // 마지막 신호일에서 gap일 안이고, 그 사이에 사건이 시작되지 않았으면 같은 구간(1980년 침체 뒤 1980년 9월 역전은 새 신호)
        while i < sig.len() && sig[i].day - sig[e].day <= gap && !starts.iter().any(|p| p.day > sig[e].day && p.day <= sig[i].day) {
            if hit(&sig[i]) { e = i; }
            i += 1;
        }
        let next = starts.iter().find(|p| p.day >= sig[s].day);
        out.push(Lead {
            start: sig[s].date.clone(),
            end: sig[e].date.clone(),
            days: sig[e].day - sig[s].day,
            event: next.map(|p| p.date.clone()),
            lead_days: next.map_or(last, |p| p.day) - sig[s].day,
        });
        i = e + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::day_number;

    fn p(d: &str, v: f64) -> Point { Point { date: d.into(), day: day_number(d).unwrap(), value: v } }

    #[test]
    fn merges_close_spells_and_finds_next_event() {
        let sig = vec![p("2000-01-01", 0.5), p("2000-02-01", -0.1), p("2000-03-01", 0.2), p("2000-05-01", -0.2), p("2000-06-01", 0.3),
                       p("2003-01-01", -0.1), p("2003-02-01", 0.1)];
        let ev = vec![p("2000-01-01", 0.0), p("2001-03-01", 1.0), p("2001-12-01", 0.0)];
        let l = leads(&sig, 0.0, true, 365, &ev);
        assert_eq!(l.len(), 2);
        assert_eq!((l[0].start.as_str(), l[0].end.as_str(), l[0].event.as_deref()), ("2000-02-01", "2000-05-01", Some("2001-03-01")));
        assert_eq!(l[0].lead_days, day_number("2001-03-01").unwrap() - day_number("2000-02-01").unwrap());
        // 사건이 아직 없으면 자료 마지막 날까지
        assert!(l[1].event.is_none() && l[1].lead_days == 31);
    }
}
