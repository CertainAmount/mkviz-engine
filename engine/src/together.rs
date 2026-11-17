//! 함께 켜진 경보: 여러 신호(각자 문턱)를 달마다 켜짐/꺼짐으로 바꾸고(그 달에 한 번이라도 문턱을 넘으면 켜짐),
//! k개 이상이 함께 켜진 구간이 사건(침체) 밖에서 시작된 횟수와 그 뒤 사건까지 걸린 달, 사건마다 시작 전 window개월 안에
//! k개 이상이 함께 켜진 적이 있었는지를 센다. 사건이 끝난 뒤 settle개월 안은 '여진'(실업이 아직 높은 때)이라 시작으로 치지 않는다.

use std::collections::{BTreeMap, BTreeSet};
use crate::series::Point;

pub struct Signal { pub pts: Vec<Point>, pub th: f64, pub below: bool }

fn add(m: &str, k: i32) -> String {
    let (y, mo): (i32, i32) = (m[..4].parse().unwrap(), m[5..7].parse().unwrap());
    let t = y * 12 + mo - 1 + k;
    format!("{:04}-{:02}", t.div_euclid(12), t.rem_euclid(12) + 1)
}

/// 달마다 켜진 신호 수(모든 신호에 자료가 있는 달만)
/// 한 신호의 달마다 켜짐(그 달에 한 번이라도 문턱을 넘으면)
pub fn on_months(s: &Signal) -> BTreeMap<String, bool> {
    let mut m: BTreeMap<String, bool> = BTreeMap::new();
    for p in &s.pts {
        let on = if s.below { p.value <= s.th } else { p.value >= s.th };
        *m.entry(p.date[..7].to_string()).or_insert(false) |= on;
    }
    m
}

pub fn counts(sigs: &[Signal]) -> BTreeMap<String, usize> {
    let per: Vec<BTreeMap<String, bool>> = sigs.iter().map(on_months).collect();
    per[0].keys().filter(|k| per.iter().all(|m| m.contains_key(*k)))
        .map(|k| (k.clone(), per.iter().filter(|m| m[k]).count())).collect()
}

pub struct Together { pub start: String, pub next_event: Option<String>, pub lead_months: Option<i32> }

/// (구간들, 사건마다 시작 전 window개월 안에 k개 이상이었나)
pub fn episodes(c: &BTreeMap<String, usize>, ev: &[Point], k: usize, settle: i32, window: i32) -> (Vec<Together>, Vec<(String, bool)>) {
    let rec: BTreeMap<String, bool> = ev.iter().map(|p| (p.date[..7].to_string(), p.value >= 0.5)).collect();
    let starts: Vec<String> = ev.windows(2).filter(|w| w[0].value < 0.5 && w[1].value >= 0.5).map(|w| w[1].date[..7].to_string()).collect();
    let ends: Vec<String> = ev.windows(2).filter(|w| w[0].value >= 0.5 && w[1].value < 0.5).map(|w| w[1].date[..7].to_string()).collect();
    let calm = |m: &str| !rec.get(m).copied().unwrap_or(false) && !ends.iter().any(|e| e.as_str() <= m && m < add(e, settle).as_str());
    let months = |a: &str, b: &str| { let f = |m: &str| m[..4].parse::<i32>().unwrap() * 12 + m[5..7].parse::<i32>().unwrap(); f(b) - f(a) };
    let mut out = Vec::new();
    let mut prev = false;
    for (m, n) in c {
        let on = *n >= k && calm(m);
        if on && !prev {
            let nx = starts.iter().find(|s| s.as_str() >= m.as_str()).cloned();
            out.push(Together { start: m.clone(), lead_months: nx.as_ref().map(|s| months(m, s)), next_event: nx });
        }
        prev = on;
    }
    let have: BTreeSet<&String> = c.keys().collect();
    let first = c.keys().next().cloned().unwrap_or_default();
    let by = starts.iter().filter(|s| add(s, -window) >= first && have.contains(*s))
        .map(|s| (s.clone(), (1..=window).any(|j| c.get(&add(s, -j)).is_some_and(|n| *n >= k)))).collect();
    (out, by)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::day_number;
    fn s(v: &[(&str, f64)]) -> Vec<Point> { v.iter().map(|(d, x)| Point { date: d.to_string(), day: day_number(d).unwrap(), value: *x }).collect() }
    #[test]
    fn counts_and_episodes() {
        let a = Signal { pts: s(&[("2000-01-03", 0.0), ("2000-01-20", 1.0), ("2000-02-01", 1.0), ("2000-03-01", 0.0), ("2000-04-01", 1.0)]), th: 0.5, below: false };
        let b = Signal { pts: s(&[("2000-01-01", -1.0), ("2000-02-01", -1.0), ("2000-03-01", 1.0), ("2000-04-01", -1.0)]), th: 0.0, below: true };
        let c = counts(&[a, b]);
        assert_eq!(c.values().copied().collect::<Vec<_>>(), vec![2, 2, 0, 2]); // 1월은 하루라도 켜지면 켜짐
        let ev = s(&[("1999-12-01", 0.0), ("2000-01-01", 0.0), ("2000-02-01", 0.0), ("2000-03-01", 1.0), ("2000-04-01", 0.0)]);
        let (e, by) = episodes(&c, &ev, 2, 0, 2);
        assert_eq!(e.len(), 2);
        assert_eq!((e[0].start.as_str(), e[0].lead_months), ("2000-01", Some(2)));
        assert_eq!(e[1].next_event, None);
        assert_eq!(by, vec![("2000-03".to_string(), true)]); // 창(2000-01~02)이 자료 안
        let (_, by) = episodes(&c, &ev, 2, 0, 3);
        assert!(by.is_empty()); // 창의 앞(1999-12)이 자료보다 이르면 뺀다
        let (e, _) = episodes(&c, &ev, 2, 2, 1); // 사건 끝(2000-04) 뒤 2달은 여진
        assert_eq!(e.len(), 1);
    }
}
