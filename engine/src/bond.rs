//! 채권 펀드 손익분기: 금리가 jump만큼 오르면 값이 바로 약 듀레이션 × jump만큼 떨어지고(볼록성 반영),
//! 그 뒤로는 더 높은 수익률로 불어난다. 금리가 그대로였을 경로와 만나는 해가 손익분기다(대략 듀레이션 근처).
//! 단순화: 펀드 수익률은 시장 금리 변화를 바로 따르고, 이후 금리는 그대로라고 본다.

/// 금리 상승 직후 값의 변화(비율). 수정 듀레이션 d, 볼록성 c(없으면 d²/2 근사 대신 0), 상승폭 dy(0.01 = 1%p)
pub fn jump_loss(d: f64, c: f64, dy: f64) -> f64 {
    -d * dy + 0.5 * c * dy * dy
}

/// t년 뒤 두 경로의 값(시작 1): (금리 그대로, 금리 상승). y0는 연 수익률(0.045 = 4.5%)
pub fn values(d: f64, c: f64, y0: f64, dy: f64, t: f64) -> (f64, f64) {
    ((1.0 + y0).powf(t), (1.0 + jump_loss(d, c, dy)) * (1.0 + y0 + dy).powf(t))
}

/// 손익분기(년): 상승 경로가 그대로 경로를 따라잡는 때
pub fn breakeven(d: f64, c: f64, y0: f64, dy: f64) -> Option<f64> {
    let k = 1.0 + jump_loss(d, c, dy);
    if dy <= 0.0 || k <= 0.0 { return None; }
    Some(-(k.ln()) / ((1.0 + y0 + dy).ln() - (1.0 + y0).ln()))
}

/// 액면 채권(반년 복리)의 수정 듀레이션: (1 - (1+y/2)^(-2m)) / y. y는 연 수익률(0.05), m은 만기(년)
pub fn par_duration(y: f64, m: f64) -> f64 {
    if y.abs() < 1e-9 { m } else { (1.0 - (1.0 + y / 2.0).powf(-2.0 * m)) / y }
}

/// 따라잡기: 고점(peak 번째 달)의 수익률 y(연, 반년 복리)로 그대로 불어났을 경로를 실제 지수가 다시 넘는 데 걸린 달 수.
/// 손익분기 규칙(≈ 듀레이션 년)을 실제 금리 경로로 시험한다. 끝까지 못 넘으면 None
pub fn catchup(idx: &[f64], peak: usize, y: f64) -> Option<usize> {
    (peak + 1..idx.len()).find(|&k| idx[k] >= idx[peak] * (1.0 + y / 2.0).powf((k - peak) as f64 / 6.0)).map(|k| k - peak)
}

/// 만기 고정 채권 펀드의 총수익 지수(시작 100): 금리 계열(%)의 달마다 마지막 값으로, 매달 그 금리를 쿠폰으로 하는
/// m년 만기 액면 채권을 사서 한 달 뒤 금리로 다시 값을 매기고(반년 복리) 한 달 이자를 더한다(Swinkels 2019의 방식).
/// 수수료·세금은 없다. 1962년부터 있는 국채 금리로 '채권 펀드가 크게 빠진 때와 회복에 걸린 시간'을 잴 수 있다.
pub fn total_return(pts: &[crate::series::Point], m: f64) -> Vec<crate::series::Point> {
    let mut ends: Vec<&crate::series::Point> = Vec::new();
    for p in pts {
        match ends.last() {
            Some(l) if l.date[..7] == p.date[..7] => { *ends.last_mut().unwrap() = p; }
            _ => ends.push(p),
        }
    }
    let mut out = Vec::with_capacity(ends.len());
    let mut idx = 100.0;
    for (k, p) in ends.iter().enumerate() {
        if k > 0 {
            let (y0, y1) = (ends[k - 1].value / 100.0, p.value / 100.0);
            let n = 2.0 * (m - 1.0 / 12.0);
            let v = (1.0 + y1 / 2.0).powf(-n);
            let price = if y1.abs() < 1e-12 { y0 / 2.0 * n + 1.0 } else { y0 / y1 * (1.0 - v) + v };
            idx *= price + y0 / 12.0;
        }
        out.push(crate::series::Point { date: p.date.clone(), day: p.day, value: idx });
    }
    out
}

/// 날마다 다시 값을 매기는 만기 고정 채권 펀드(시작 100): total_return과 같은 방식을 관측일 간격(년 = 날 수 / 365.25)으로.
/// 주식 하락처럼 몇 주짜리 구간에서도 같은 날짜로 채권 쪽 수익을 잴 수 있게 한다(월말 지수는 구간 끝을 놓친다)
pub fn daily_index(pts: &[crate::series::Point], m: f64) -> Vec<crate::series::Point> {
    let mut out: Vec<crate::series::Point> = Vec::with_capacity(pts.len());
    let mut idx = 100.0;
    for (k, p) in pts.iter().enumerate() {
        if k > 0 {
            let (y0, y1) = (pts[k - 1].value / 100.0, p.value / 100.0);
            let t = (p.day - pts[k - 1].day) as f64 / 365.25;
            let n = 2.0 * (m - t);
            let v = (1.0 + y1 / 2.0).powf(-n);
            let price = if y1.abs() < 1e-12 { y0 / 2.0 * n + 1.0 } else { y0 / y1 * (1.0 - v) + v };
            idx *= price + y0 * t;
        }
        out.push(crate::series::Point { date: p.date.clone(), day: p.day, value: idx });
    }
    out
}

/// 주식이 기록에서 threshold 이상 빠진 하락마다(기록일 → 바닥일, 진행 중이면 지금까지 바닥), 같은 날짜 사이 채권 펀드 총수익과 금리 변화(%p)
#[derive(Debug, Clone)]
pub struct Catch { pub peak: String, pub low_date: String, pub fall: f64, pub bond: f64, pub dy: f64, pub y0: f64, pub open: bool }

pub fn catch(stock: &[crate::series::Point], rates: &[crate::series::Point], m: f64, th: f64) -> Vec<Catch> {
    let idx = daily_index(rates, m);
    // 그날이나 그 전 마지막 관측(주식 거래일과 국채 휴장일이 다를 수 있다)
    let at = |d: &str| rates.iter().rposition(|p| p.date.as_str() <= d);
    let first = rates.first().map(|p| p.date.clone()).unwrap_or_default();
    crate::episodes::drawdowns(stock, th).into_iter().filter(|e| e.peak >= first).filter_map(|e| {
        let (a, b) = (at(&e.peak)?, at(&e.low_date)?);
        Some(Catch { peak: e.peak.clone(), low_date: e.low_date.clone(), fall: e.low, bond: idx[b].value / idx[a].value - 1.0,
                     dy: rates[b].value - rates[a].value, y0: rates[a].value, open: e.recovered.is_none() })
    }).collect()
}

/// 한 해의 이자 쿠션: 해 첫 금리 y0(%)로 액면 채권의 듀레이션 D를 구하면, 금리가 y0/D(%p)만큼 오를 때 값의 손실(≈D×상승)이
/// 한 해 이자(≈y0)를 다 지운다. 해마다 (해, 시작 금리, 듀레이션, 쿠션 %p, 실제 금리 변화 %p, 펀드 총수익, 해가 다 찼나).
/// 금리는 해 직전 달(전해 12월) 마지막 값부터 그해 마지막 달까지, 총수익은 total_return 지수의 같은 두 달 비율.
pub struct CushionYear { pub year: i32, pub start: f64, pub duration: f64, pub cushion: f64, pub change: f64, pub ret: f64, pub full: bool }

pub fn cushion(pts: &[crate::series::Point], m: f64) -> Vec<CushionYear> {
    let idx = total_return(pts, m);
    let mut last_in: std::collections::BTreeMap<i32, (f64, f64, String)> = std::collections::BTreeMap::new();
    let mut by_month: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for p in pts { by_month.insert(p.date[..7].to_string(), p.value); }
    for (p, i) in by_month.iter().zip(idx.iter()) {
        let y: i32 = p.0[..4].parse().unwrap_or(0);
        last_in.insert(y, (*p.1, i.value, p.0.clone()));
    }
    let mut out = Vec::new();
    for (y, (r1, i1, mon)) in &last_in {
        let Some((r0, i0, mon0)) = last_in.get(&(y - 1)) else { continue };
        if !mon0.ends_with("-12") { continue; }
        let d = par_duration(r0 / 100.0, m);
        out.push(CushionYear { year: *y, start: *r0, duration: d, cushion: r0 / d, change: r1 - r0, ret: i1 / i0 - 1.0, full: mon.ends_with("-12") });
    }
    out
}

/// 한 해 총수익을 이자 몫과 가격 몫으로 나눈다(total_return과 같은 펀드). 달마다 이자 = 지수 × y0/12, 가격 = 나머지.
/// 해마다 (해, 이자 몫, 가격 몫, 총수익, 해가 다 찼나). 셋 다 전해 12월 말 지수 대비 비율이라 이자 + 가격 = 총수익.
pub struct SplitYear { pub year: i32, pub income: f64, pub price: f64, pub ret: f64, pub full: bool }

pub fn split(pts: &[crate::series::Point], m: f64) -> Vec<SplitYear> {
    let idx = total_return(pts, m);
    let mut by_month: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for p in pts { by_month.insert(p.date[..7].to_string(), p.value); }
    let months: Vec<(&String, &f64)> = by_month.iter().collect();
    // 해마다 (시작 지수, 이자 합, 끝 지수, 끝 달)
    let mut acc: std::collections::BTreeMap<i32, (f64, f64, f64, String)> = std::collections::BTreeMap::new();
    for k in 1..months.len().min(idx.len()) {
        let y: i32 = months[k].0[..4].parse().unwrap_or(0);
        let prev = idx[k - 1].value;
        let inc = prev * months[k - 1].1 / 100.0 / 12.0;
        let e = acc.entry(y).or_insert((prev, 0.0, prev, String::new()));
        if months[k - 1].0[..4] != months[k].0[..4] && !months[k - 1].0.ends_with("-12") { e.0 = f64::NAN; }
        e.1 += inc;
        e.2 = idx[k].value;
        e.3 = months[k].0.clone();
    }
    acc.into_iter().filter(|(_, (s, ..))| s.is_finite()).map(|(y, (s, inc, end, mon))| {
        let ret = end / s - 1.0;
        SplitYear { year: y, income: inc / s, price: ret - inc / s, ret, full: mon.ends_with("-12") }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catchup_and_par_duration() {
        // 5% 액면 7년물 듀레이션 약 5.8년, 0%면 만기와 같다
        assert!((par_duration(0.05, 7.0) - 5.85).abs() < 0.02, "{}", par_duration(0.05, 7.0));
        assert_eq!(par_duration(0.0, 7.0), 7.0);
        // 고점 100에서 떨어졌다가 연 12%(반년 6%)로 그대로였을 경로를 6달째에 넘는다
        let idx = [100.0, 95.0, 97.0, 100.0, 103.0, 104.5, 106.5, 107.0];
        assert_eq!(catchup(&idx, 0, 0.12), Some(6));
        assert_eq!(catchup(&idx[..6], 0, 0.12), None);
    }

    #[test]
    fn split_adds_up_and_flat_year_is_all_interest() {
        use crate::series::Point;
        let p = |d: &str, v: f64| Point { date: d.into(), day: crate::series::day_number(d).unwrap(), value: v };
        let mut pts = vec![p("2020-12-31", 5.0)];
        for mth in 1..=12 { pts.push(p(&format!("2021-{mth:02}-28"), 5.0)); }
        for mth in 1..=6 { pts.push(p(&format!("2022-{mth:02}-28"), 6.0)); }
        let s = split(&pts, 7.0);
        assert_eq!(s.len(), 2);
        assert!(s[0].full && (s[0].price).abs() < 1e-9 && (s[0].income - 0.0512).abs() < 0.001, "{}", s[0].income);
        assert!(!s[1].full && s[1].price < -0.05 && (s[1].income + s[1].price - s[1].ret).abs() < 1e-12);
    }

    #[test]
    fn breakeven_near_duration() {
        // 듀레이션 6년, 4.5% → 5.5%: 바로 약 6% 손실, 손익분기는 약 6년
        let l = jump_loss(6.0, 0.0, 0.01);
        assert!((l + 0.06).abs() < 1e-9);
        let b = breakeven(6.0, 0.0, 0.045, 0.01).unwrap();
        assert!(b > 5.5 && b < 7.0, "{b}");
        let (a, j) = values(6.0, 0.0, 0.045, 0.01, b);
        assert!((a - j).abs() < 1e-9);
    }

    #[test]
    fn total_return_flat_and_jump() {
        use crate::series::Point;
        let p = |d: &str, v: f64| Point { date: d.into(), day: crate::series::day_number(d).unwrap(), value: v };
        // 금리가 그대로면 한 달에 이자만큼(5%/12) 는다
        let flat = total_return(&[p("2020-01-31", 5.0), p("2020-02-29", 5.0)], 7.0);
        assert!((flat[1].value / flat[0].value - (1.0 + 0.05 / 12.0)).abs() < 1e-9);
        // 7년 만기에서 금리가 1%p 오르면 약 듀레이션(6년 안팎)만큼 빠진다
        let up = total_return(&[p("2020-01-31", 5.0), p("2020-02-15", 5.5), p("2020-02-29", 6.0)], 7.0);
        assert_eq!(up.len(), 2);
        let r = up[1].value / up[0].value - 1.0;
        assert!(r < -0.05 && r > -0.065, "{r}");
    }

    #[test]
    fn cushion_flags_years_where_rise_beats_interest() {
        use crate::series::Point;
        let p = |d: &str, v: f64| Point { date: d.into(), day: crate::series::day_number(d).unwrap(), value: v };
        // 2020년 말 2%, 2021년 말 3%(쿠션 약 0.3%p보다 크게 오름 → 손실), 2022년 말 3%(그대로 → 이자만큼 이익)
        let mut v = vec![p("2020-12-31", 2.0)];
        for m in 1..=12 { v.push(p(&format!("2021-{m:02}-28"), 2.0 + m as f64 / 12.0)); }
        for m in 1..=6 { v.push(p(&format!("2022-{m:02}-28"), 3.0)); }
        let c = cushion(&v, 7.0);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].year, 2021);
        assert!((c[0].cushion - 2.0 / par_duration(0.02, 7.0)).abs() < 1e-9 && (c[0].change - 1.0).abs() < 1e-9);
        assert!(c[0].ret < 0.0 && c[0].full);
        assert!(c[1].ret > 0.0 && !c[1].full && c[1].change.abs() < 1e-9);
    }

    #[test]
    fn daily_index_matches_carry_and_catch_pairs_dates() {
        use crate::series::Point;
        let mk = |d: &str, v: f64| Point { date: d.into(), day: crate::series::day_number(d).unwrap(), value: v };
        // 금리가 그대로면 1년 뒤 약 금리만큼 불어난다
        let r: Vec<Point> = (0..=12).map(|k| mk(&format!("{}-{:02}-01", 2020 + (k / 12), k % 12 + 1), 5.0)).collect();
        let i = daily_index(&r, 7.0);
        assert!((i.last().unwrap().value / 100.0 - 1.05).abs() < 0.003, "{}", i.last().unwrap().value);
        // 주식이 20% 빠지는 동안 금리가 1%p 내리면 채권은 오른다
        let st = vec![mk("2020-01-01", 100.0), mk("2020-02-01", 80.0), mk("2020-03-01", 101.0)];
        let rt = vec![mk("2020-01-01", 3.0), mk("2020-02-01", 2.0), mk("2020-03-01", 2.0)];
        let c = catch(&st, &rt, 7.0, 0.10);
        assert_eq!(c.len(), 1);
        assert!(c[0].bond > 0.05 && (c[0].dy + 1.0).abs() < 1e-9 && (c[0].fall + 0.2).abs() < 1e-9 && !c[0].open);
    }
}