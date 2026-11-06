//! 하락 사례: 직전 최고점에서 threshold 이상 떨어진 때를 하나의 사례로 묶고, 바닥과 회복(새 최고점)을 찾는다.
//! "과거 N번 중 몇 번이 더 나빠졌고 얼마나 걸렸나"의 재료(2026-11-05 나스닥 글).

use crate::series::Point;
use crate::stats::median;

#[derive(Debug, Clone)]
pub struct Episode {
    pub peak: String,
    pub peak_value: f64,
    pub cross: String,
    pub low: f64,
    pub low_date: String,
    pub recovered: Option<String>,
    pub days_to_cross_low: i64,
    pub days_to_recover: Option<i64>,
    pub flagged: Option<bool>,
}

/// threshold: 0.10이면 10% 하락. 마지막 사례가 회복 전이면 recovered = None(진행 중).
pub fn drawdowns(pts: &[Point], threshold: f64) -> Vec<Episode> {
    let mut out = Vec::new();
    let mut peak = &pts[0];
    let mut cur: Option<(Episode, i64, i64)> = None; // (사례, 교차일, 바닥일)
    for p in pts {
        if p.value >= peak.value {
            if let Some((mut e, _, _)) = cur.take() {
                e.recovered = Some(p.date.clone());
                e.days_to_recover = Some(p.day - crate::series::day_number(&e.peak).unwrap());
                out.push(e);
            }
            peak = p;
            continue;
        }
        let dd = p.value / peak.value - 1.0;
        match cur.as_mut() {
            None if dd <= -threshold => {
                cur = Some((
                    Episode {
                        peak: peak.date.clone(), peak_value: peak.value, cross: p.date.clone(), low: dd,
                        low_date: p.date.clone(), recovered: None, days_to_cross_low: 0, days_to_recover: None, flagged: None,
                    },
                    p.day,
                    p.day,
                ));
            }
            Some((e, cross_day, low_day)) if dd < e.low => {
                e.low = dd;
                e.low_date = p.date.clone();
                *low_day = p.day;
                e.days_to_cross_low = *low_day - *cross_day;
            }
            _ => {}
        }
    }
    if let Some((e, _, _)) = cur {
        out.push(e);
    }
    out
}

/// flag 시계열(예: FRED USREC, 1 = 경기침체)이 고점~바닥 사이에 1이었는지 표시한다(월간이면 그 달 1일 기준).
pub fn mark(episodes: &mut [Episode], flag: &[Point]) {
    for e in episodes.iter_mut() {
        let a = crate::series::day_number(&e.peak).unwrap() - 31;
        let b = crate::series::day_number(&e.low_date).unwrap();
        e.flagged = Some(flag.iter().any(|f| f.day >= a && f.day <= b && f.value >= 1.0));
    }
}

pub struct Summary {
    pub n: usize,
    pub stopped: usize,
    pub deeper: usize,
    pub median_low_stopped: Option<f64>,
    pub median_low_deeper: Option<f64>,
    pub median_recover_stopped: Option<f64>,
    pub median_recover_deeper: Option<f64>,
    pub median_cross_to_low_stopped: Option<f64>,
}

/// 끝난 사례만으로 요약. deeper: 바닥이 -deep 이하(예: 0.20 하락장).
pub fn summarize(eps: &[Episode], deep: f64) -> Summary {
    let done: Vec<&Episode> = eps.iter().filter(|e| e.recovered.is_some()).collect();
    let (s, d): (Vec<&Episode>, Vec<&Episode>) = done.iter().partition(|e| e.low > -deep);
    let lows = |v: &[&Episode]| median(&v.iter().map(|e| e.low).collect::<Vec<_>>());
    let rec = |v: &[&Episode]| median(&v.iter().map(|e| e.days_to_recover.unwrap() as f64).collect::<Vec<_>>());
    Summary {
        n: done.len(),
        stopped: s.len(),
        deeper: d.len(),
        median_low_stopped: lows(&s),
        median_low_deeper: lows(&d),
        median_recover_stopped: rec(&s),
        median_recover_deeper: rec(&d),
        median_cross_to_low_stopped: median(&s.iter().map(|e| e.days_to_cross_low as f64).collect::<Vec<_>>()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn finds_episodes_and_recovery() {
        let p = parse("d,v\n2020-01-01,100\n2020-01-02,85\n2020-01-03,70\n2020-01-04,101\n2020-01-05,95\n2020-01-06,89\n");
        let e = drawdowns(&p, 0.10);
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].cross, "2020-01-02");
        assert!((e[0].low + 0.30).abs() < 1e-9);
        assert_eq!(e[0].recovered.as_deref(), Some("2020-01-04"));
        assert_eq!(e[0].days_to_recover, Some(3));
        assert!(e[1].recovered.is_none()); // 진행 중
        let s = summarize(&e, 0.20);
        assert_eq!((s.n, s.stopped, s.deeper), (1, 0, 1));
    }

    #[test]
    fn marks_recession_overlap() {
        let p = parse("d,v\n2020-01-01,100\n2020-03-01,80\n2020-06-01,101\n");
        let mut e = drawdowns(&p, 0.10);
        mark(&mut e, &parse("d,v\n2020-02-01,1\n"));
        assert_eq!(e[0].flagged, Some(true));
    }
}

/// 급등 사례(유가·금리 급등이 얼마나 갔나): 직전 window개 점 중 최저 대비 threshold 이상 오른 첫날을 시작으로,
/// 앞의 급등과 window개 점 이상 떨어져야 새 사례다. (시작 인덱스, 직전 최저 인덱스, 시작 시 상승률)
pub fn spikes(pts: &[Point], window: usize, threshold: f64) -> Vec<(usize, usize, f64)> {
    let mut out: Vec<(usize, usize, f64)> = Vec::new();
    for i in window..pts.len() {
        let (lo_i, lo) = pts[i - window..i].iter().enumerate().fold((0, f64::MAX), |a, (k, p)| if p.value < a.1 { (k, p.value) } else { a });
        if lo <= 0.0 {
            continue; // 저점이 0 이하(2020년 4월 WTI 마이너스)면 상승률이 뜻이 없다
        }
        let g = pts[i].value / lo - 1.0;
        if g >= threshold && out.last().map_or(true, |l| i - l.0 >= window) {
            out.push((i, i - window + lo_i, g));
        }
    }
    out
}

/// 급락 사례: spikes의 반대(직전 window개 점의 최고 대비 threshold 이상 내린 첫날). (시작 인덱스, 직전 최고 인덱스, 시작 시 하락률(음수))
pub fn drops(pts: &[Point], window: usize, threshold: f64) -> Vec<(usize, usize, f64)> {
    let mut out: Vec<(usize, usize, f64)> = Vec::new();
    for i in window..pts.len() {
        let (hi_i, hi) = pts[i - window..i].iter().enumerate().fold((0, f64::MIN), |a, (k, p)| if p.value > a.1 { (k, p.value) } else { a });
        if hi <= 0.0 || pts[i].value <= 0.0 {
            continue;
        }
        let g = pts[i].value / hi - 1.0;
        if g <= -threshold && out.last().map_or(true, |l| i - l.0 >= window) {
            out.push((i, i - window + hi_i, g));
        }
    }
    out
}

#[cfg(test)]
mod spike_tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn spike_starts_once() {
        let p = parse("d,v\n2020-01-01,100\n2020-01-02,100\n2020-01-03,120\n2020-01-04,140\n2020-01-05,150\n");
        let s = spikes(&p, 2, 0.3);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].0, 3); // 140 / 100 - 1 = 0.4
        let d = drops(&parse("d,v\n2020-01-01,100\n2020-01-02,95\n2020-01-03,70\n2020-01-04,60\n"), 2, 0.25);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].0, 2); // 70 / 100 - 1 = -0.3
    }
}
