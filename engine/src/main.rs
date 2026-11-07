//! mkviz-engine: 공식 장기 시계열로 "과거에는 어떻게 됐나"를 계산하는 분석 엔진. 결과는 JSON(표준 출력).
//! 글마다 즉석 계산하지 않고 여기 기능을 늘려 간다(남이 바로 줄 수 없는 분석이 E-E-A-T가 되게, Rust로 빠르고 가볍게).
//!
//! 명령:
//!   drawdowns <csv> [--threshold 0.10] [--deep 0.20] [--flag <csv>] [--forward 91,182,365]
//!       최고점 대비 threshold 이상 하락한 사례 전부, 바닥·회복·기간, deep 이상(하락장) 갈래 요약,
//!       flag(예: 경기침체) 겹침, 하락선을 넘은 날 이후 N일 변화율.
//!   position <csv> [--change 12] [--from YYYY-MM-DD]
//!       지금 값이 역사 속 어디쯤인지(순위 비율, 최저·최고와 날짜, 중앙값). --change N: N행 전 대비 변화율로 본다(월간 12 = 1년).
//!   turning <csv> [--window 8 --censor 6 --min-phase 4 --min-cycle 16 --big 0.20]
//!       강세·약세장 판정(Pagan & Sossounov 2003). 월말 값으로 고점·저점을 찾고 국면별 변화·기간.
//!   sahm <unrate.csv>     Sahm 규칙(실업률 3개월 평균 - 직전 12개월 최저, 0.50%p 신호)과 지난 신호 날짜.
//!   events <csv> --dates d1,d2,... [--forward 91,182,365]
//!       사건 뒤 N일 변화와 같은 길이 아무 구간(기저율) 비교.
//!   figure underwater|paths <csv> --out <svg> [--title …] [--lines 0.1,0.2 --flag <csv>] [--dates d1,… --days 365 --diff --labels a,b,…]
//!       비주얼 엔진 프레임(viz.rs)을 SVG 파일로.
//!   spikes <csv> [--window 63 --threshold 0.30 --forward 91,182,365]
//!       급등 사례: 직전 window개 점의 최저 대비 threshold 이상 오른 첫날, 그 뒤 최고와 N일 뒤 변화.
//!   recession-prob <spread.csv> --flag <usrec.csv> [--ahead 12]
//!       수익률 곡선(10년-3개월) 월평균으로 ahead개월 뒤 침체 확률 프로빗을 우리 자료로 추정(Estrella & Mishkin 꼴).
//!   analogs <a.csv,b.csv,...> [--mode yoy|level --k 5 --exclude 24 --target <csv> --ahead 12]
//!       과거 비슷한 달 k개(여러 지표의 표준화 값)와 target의 ahead개월 뒤 변화.
//!   figure band|dots <csv> --out <svg> [--pct --suffix % --from YYYY-MM-DD] | [--threshold 0.10 --deep 0.20]
//!   figure fan <csv> --dates d1,d2,… --out <svg> [--weeks 26 --label … --base --base-label …]
//!       사건 날짜들 뒤 경로의 중앙값 선 + 사분위 띠(강조할 지금 사례가 없을 때).
//!   figure response <leader.csv> --follower <csv> --out <svg> [--threshold 0.20 --window 63 --weeks 20]
//!       반응 경로: 선행 지표가 크게 내린 때·오른 때를 0일로 맞춘 후행 지표 경로(중앙값 + 사분위 띠), 두 그룹을 한 축에.
//!   passthrough <leader.csv> --follower <csv> [--threshold 0.20 --window 63 --forward 14,28,56,91]
//!       선행 지표(유가)가 window개 점 안에서 threshold 이상 오른 날·내린 날마다, 후행 지표(휘발유)가 N일 뒤 얼마나 따라왔나.
//!       오를 때와 내릴 때를 나눠 중앙값을 낸다(반영의 비대칭, '로켓과 깃털' 시험).
//!   cycles <rate.csv>[,<newer.csv>] [--flag <usrec.csv> --within 730]
//!       정책 금리 사이클(같은 방향 변화 묶음): 방향, 첫·마지막 변화일, 횟수, 몇 %p. --flag: 첫 변화 뒤 within일 안에 침체가 있었나.
//!       여러 파일은 이어 붙인다(DFEDTAR 1982-2008 + DFEDTARU 2008-).
//!   events … --diff     비율 대신 수준 차이(금리: %p)로.
//!   payment --loan 300000 --rates 6.66,7.18 [--years 30]
//!       고정금리 대출의 월 상환액(원리금 균등: P = L·r / (1 - (1+r)^-n), r = 연이율/12).
//!   spells <csv> --threshold x [--below]   값이 기준 이상(이하)으로 이어진 구간(시작·끝·길이·극값·진행 중).
//!   (공통) --per <csv>: 다른 계열로 나눈 비율.
//!   (공통) --afford <rate.csv>,<income.csv> [--down 0.2 --years 30]: 집값 계열을 월 상환액 / 월 소득으로 바꾼다.
//!   export <csv> --out <json> [--from YYYY]
//!       독자 도구(계산기)용 월간 데이터: {"start":"YYYY-MM","v":[달마다 마지막 값...]}. 빈 달은 직전 값.
//! 모든 결과 JSON에는 method(방법 한 줄과 출처)와 provenance(엔진 버전, 입력 파일, 행 수, 기간, 옵션)가 붙는다(METHODS.md B1).

mod analogs;
mod seasonal;
mod cycles;
mod episodes;
mod events;
mod forward;
mod indicators;
mod lead;
mod probit;
mod series;
mod stats;
mod turning;
mod viz;

use std::collections::HashMap;
use std::fmt::Write;

fn args() -> (Vec<String>, HashMap<String, String>) {
    let (mut pos, mut opt) = (Vec::new(), HashMap::new());
    let mut it = std::env::args().skip(1).peekable();
    while let Some(a) = it.next() {
        if let Some(k) = a.strip_prefix("--") {
            // 값이 없는 깃발(--diff, --pct)은 다음 낱말이 --로 시작하거나 없을 때
            let v = if it.peek().map_or(true, |n| n.starts_with("--")) { String::new() } else { it.next().unwrap() };
            opt.insert(k.to_string(), v);
        } else {
            pos.push(a);
        }
    }
    (pos, opt)
}

fn num(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{}", (x * 1e6).round() / 1e6),
        _ => "null".into(),
    }
}

fn s(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

fn opt_f(o: &HashMap<String, String>, k: &str, d: f64) -> f64 {
    o.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn drawdowns(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let pts = load_with(path, o)?;
    let (th, deep) = (opt_f(o, "threshold", 0.10), opt_f(o, "deep", 0.20));
    let mut eps = episodes::drawdowns(&pts, th);
    if let Some(f) = o.get("flag") {
        episodes::mark(&mut eps, &series::load(f)?);
    }
    let fwd: Vec<i64> = o.get("forward").map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
    let mut j = String::from("{\"episodes\":[");
    for (i, e) in eps.iter().enumerate() {
        let cross = series::day_number(&e.cross).unwrap();
        let f: Vec<String> = fwd.iter().map(|d| format!("\"{d}\":{}", num(forward::after(&pts, cross, *d)))).collect();
        let _ = write!(
            j,
            "{}{{\"peak\":{},\"peak_value\":{},\"cross\":{},\"low\":{},\"low_date\":{},\"recovered\":{},\"days_cross_to_low\":{},\"days_peak_to_recover\":{},\"flag\":{},\"forward\":{{{}}}}}",
            if i > 0 { "," } else { "" }, s(&e.peak), num(Some(e.peak_value)), s(&e.cross), num(Some(e.low)), s(&e.low_date),
            e.recovered.as_deref().map(s).unwrap_or("null".into()), e.days_to_cross_low,
            e.days_to_recover.map(|d| d.to_string()).unwrap_or("null".into()),
            e.flagged.map(|b| b.to_string()).unwrap_or("null".into()), f.join(",")
        );
    }
    let sm = episodes::summarize(&eps, deep);
    let flagged_deep = eps.iter().filter(|e| e.recovered.is_some() && e.low <= -deep && e.flagged == Some(true)).count();
    let last = pts.last().unwrap();
    let peak = pts.iter().fold(&pts[0], |a, p| if p.value >= a.value { p } else { a });
    let _ = write!(
        j,
        "],\"summary\":{{\"threshold\":{th},\"deep\":{deep},\"n\":{},\"stopped\":{},\"deeper\":{},\"median_low_stopped\":{},\"median_low_deeper\":{},\"median_days_recover_stopped\":{},\"median_days_recover_deeper\":{},\"median_days_cross_to_low_stopped\":{},\"deeper_with_flag\":{},\"ci90_days_recover_stopped\":{}}},\"now\":{{\"date\":{},\"value\":{},\"peak\":{},\"peak_value\":{},\"from_peak\":{}}},\"first\":{}}}",
        sm.n, sm.stopped, sm.deeper, num(sm.median_low_stopped), num(sm.median_low_deeper), num(sm.median_recover_stopped),
        num(sm.median_recover_deeper), num(sm.median_cross_to_low_stopped),
        if o.contains_key("flag") { flagged_deep.to_string() } else { "null".into() },
        stats::median_ci(&eps.iter().filter(|e| e.recovered.is_some() && e.low > -deep).map(|e| e.days_to_recover.unwrap() as f64).collect::<Vec<_>>(), 0.9, 4000, 42)
            .map(|(a, b)| format!("[{a},{b}]")).unwrap_or("null".into()),
        s(&last.date), num(Some(last.value)), s(&peak.date), num(Some(peak.value)), num(Some(last.value / peak.value - 1.0)),
        s(&pts[0].date)
    );
    Ok(j)
}

/// --splice <옛 계열 csv>가 있으면 옛 계열을 앞에 이어 붙인다(series::splice).
fn load_with(path: &str, o: &HashMap<String, String>) -> Result<Vec<series::Point>, String> {
    let pts = series::load(path)?;
    let pts = match o.get("splice") { Some(old) => series::splice(&pts, &series::load(old)?), None => pts };
    // --deflate <물가지수 csv>: 오늘 달러로 바꾼 실질 값(예: 금의 1980년 고점은 물가를 빼면 언제 넘었나)
    let pts = match o.get("deflate") { Some(c) => series::deflate(&pts, &series::load(c)?), None => pts };
    // --avg n: n개 이동 평균(계절이 큰 월별 계열을 해 단위로 견줄 때)
    // --per <csv>: 다른 계열로 나눈 비율(집값 / 연 소득 = 소득 몇 년 치)
    let pts = match o.get("per") { Some(c) => series::per(&pts, &series::load(c)?), None => pts };
    // --afford <금리 csv>,<연 소득 csv> [--down 0.2 --years 30]: 집값 -> 월 상환액이 월 소득에서 차지하는 몫(series::afford)
    let pts = match o.get("afford") {
        Some(v) => {
            let (r, i) = v.split_once(',').ok_or("--afford 는 금리csv,소득csv")?;
            let num = |k: &str, d: f64| o.get(k).and_then(|x| x.parse().ok()).unwrap_or(d);
            series::afford(&pts, &series::load(r)?, &series::load(i)?, num("down", 0.2), num("years", 30.0))
        }
        None => pts,
    };
    Ok(match o.get("avg").and_then(|v| v.parse().ok()) { Some(n) => series::rolling_mean(&pts, n), None => pts })
}

fn position(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let mut pts = load_with(path, o)?;
    if let Some(n) = o.get("change").and_then(|v| v.parse().ok()) {
        pts = series::change_over(&pts, n);
    }
    if let Some(from) = o.get("from").and_then(|d| series::day_number(d)) {
        pts.retain(|p| p.day >= from);
    }
    if pts.is_empty() {
        return Err("자료 없음".into());
    }
    let vals: Vec<f64> = pts.iter().map(|p| p.value).collect();
    let last = pts.last().unwrap();
    let lo = pts.iter().fold(&pts[0], |a, p| if p.value < a.value { p } else { a });
    let hi = pts.iter().fold(&pts[0], |a, p| if p.value > a.value { p } else { a });
    // 지금 값 이하였던 마지막 때(예: 이보다 낮았던 건 언제가 마지막인가)
    let below = pts[..pts.len() - 1].iter().rev().find(|p| p.value <= last.value).map(|p| s(&p.date)).unwrap_or("null".into());
    let above = pts[..pts.len() - 1].iter().rev().find(|p| p.value >= last.value).map(|p| s(&p.date)).unwrap_or("null".into());
    Ok(format!(
        "{{\"date\":{},\"value\":{},\"rank\":{},\"n\":{},\"first\":{},\"min\":{},\"min_date\":{},\"max\":{},\"max_date\":{},\"median\":{},\"last_at_or_below\":{},\"last_at_or_above\":{}}}",
        s(&last.date), num(Some(last.value)), num(Some(stats::rank(&vals, last.value))), vals.len(), s(&pts[0].date),
        num(Some(lo.value)), s(&lo.date), num(Some(hi.value)), s(&hi.date), num(stats::median(&vals)), below, above
    ))
}

fn turning_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let m = turning::monthly(&load_with(path, o)?);
    let d = turning::Params::default();
    let g = |k: &str, v: usize| o.get(k).and_then(|x| x.parse().ok()).unwrap_or(v);
    let p = turning::Params { window: g("window", d.window), censor: g("censor", d.censor), min_phase: g("min-phase", d.min_phase),
                              min_cycle: g("min-cycle", d.min_cycle), big: opt_f(o, "big", d.big) };
    let t = turning::turns(&m, &p);
    let mut j = String::from("{\"turns\":[");
    for (k, x) in t.iter().enumerate() {
        let _ = write!(j, "{}{{\"date\":{},\"kind\":{},\"value\":{}}}", if k > 0 { "," } else { "" }, s(&m[x.i].date), s(if x.peak { "peak" } else { "trough" }), num(Some(m[x.i].value)));
    }
    j.push_str("],\"phases\":[");
    let mut bears = (Vec::new(), Vec::new());
    for (k, w) in t.windows(2).enumerate() {
        let ch = m[w[1].i].value / m[w[0].i].value - 1.0;
        let months = w[1].i - w[0].i;
        if w[0].peak {
            bears.0.push(ch);
            bears.1.push(months as f64);
        }
        let _ = write!(j, "{}{{\"kind\":{},\"from\":{},\"to\":{},\"change\":{},\"months\":{months}}}", if k > 0 { "," } else { "" },
                       s(if w[0].peak { "bear" } else { "bull" }), s(&m[w[0].i].date), s(&m[w[1].i].date), num(Some(ch)));
    }
    let _ = write!(j, "],\"summary\":{{\"bears\":{},\"median_bear_change\":{},\"median_bear_months\":{}}},\"provisional_after\":{}}}",
                   bears.0.len(), num(stats::median(&bears.0)), num(stats::median(&bears.1)), s(&m[m.len().saturating_sub(p.window + 1)].date));
    Ok(j)
}

fn sahm_cmd(path: &str) -> Result<String, String> {
    let v = indicators::sahm(&series::load(path)?);
    let t = indicators::triggers(&v, 0.5);
    let last = v.last().ok_or("자료 부족")?;
    Ok(format!("{{\"date\":{},\"value\":{},\"signal\":{},\"triggers\":[{}]}}", s(&last.0), num(Some(last.1)), last.1 >= 0.5,
               t.iter().map(|d| s(d)).collect::<Vec<_>>().join(",")))
}

fn events_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let pts = load_with(path, o)?;
    let dates: Vec<&str> = o.get("dates").map(|v| v.split(',').collect()).unwrap_or_default();
    let ev: Vec<i64> = dates.iter().filter_map(|d| series::day_number(d)).collect();
    if ev.is_empty() {
        return Err("--dates 가 필요".into());
    }
    let hz: Vec<i64> = o.get("forward").map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![91, 182, 365]);
    let mut j = String::from("{\"windows\":[");
    let diff = o.contains_key("diff");
    for (k, w) in events::study(&pts, &ev, &hz, diff).iter().enumerate() {
        let per: Vec<String> = dates.iter().zip(&w.per_event).map(|(d, v)| format!("{}:{}", s(d), num(*v))).collect();
        let worst: Vec<f64> = w.worst.iter().flatten().cloned().collect();
        let ws: Vec<String> = dates.iter().zip(&w.worst).map(|(d, v)| format!("{}:{}", s(d), num(*v))).collect();
        let _ = write!(j, "{}{{\"days\":{},\"n\":{},\"up\":{},\"median\":{},\"base_median\":{},\"base_up_share\":{},\"per_event\":{{{}}},\"worst_along_way\":{{{}}},\"median_worst\":{},\"dips_10pct\":{}}}",
                       if k > 0 { "," } else { "" }, w.days, w.n, w.up, num(w.median), num(w.base_median), num(w.base_up_share), per.join(","),
                       ws.join(","), num(stats::median(&worst)), worst.iter().filter(|&&x| x <= -0.10).count());
    }
    j.push_str("]}");
    Ok(j)
}

/// during: 다른 계열(--episodes)이 크게 빠진 구간마다(고점 → 바닥) 이 계열은 얼마나 움직였나.
/// '안전자산'이 실제로 폭락장에서 버텼는지 같은 질문(조건부 비교, 같은 구간을 맞대는 사건 연구의 꼴).
fn during_rows(pts: &[series::Point], o: &HashMap<String, String>) -> Result<Vec<(String, f64, f64)>, String> {
    let other = series::load(o.get("episodes").ok_or("--episodes <csv> 가 필요")?)?;
    let at = |d: i64| pts.iter().filter(|p| p.day <= d).last().map(|p| p.value);
    let oat = |d: i64| other.iter().filter(|p| p.day <= d).last().map(|p| p.value);
    let mut rows: Vec<(String, f64, f64)> = episodes::drawdowns(&other, opt_f(o, "threshold", 0.20)).iter().filter_map(|e| {
        let (a, b) = (series::day_number(&e.peak)?, series::day_number(&e.low_date)?);
        let (va, vb) = (at(a)?, at(b)?);
        Some((e.peak[..4].to_string(), e.low, vb / va - 1.0))
    }).collect();
    // --windows 시작:끝,…: 기록 고점 기준 하락에 안 잡히는 폭락(예: 2000년 고점을 아직 못 넘은 채 온 2008년)을 직접 더한다
    if let Some(w) = o.get("windows") {
        for pair in w.split(',') {
            let (a, b) = pair.split_once(':').ok_or("--windows 는 시작:끝")?;
            let (da, db) = (series::day_number(a).ok_or("날짜")?, series::day_number(b).ok_or("날짜")?);
            if let (Some(va), Some(vb), Some(oa), Some(ob)) = (at(da), at(db), oat(da), oat(db)) {
                rows.push((a[..4].to_string(), ob / oa - 1.0, vb / va - 1.0));
            }
        }
        rows.sort_by(|x, y| x.0.cmp(&y.0));
    }
    Ok(rows)
}

fn during_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let rows = during_rows(&load_with(path, o)?, o)?;
    let ch: Vec<f64> = rows.iter().map(|r| r.2).collect();
    let items: Vec<String> = rows.iter().map(|(y, f, c)| format!("{{\"year\":{},\"other_fall\":{},\"change\":{}}}", s(y), num(Some(*f)), num(Some(*c)))).collect();
    Ok(format!("{{\"episodes\":[{}],\"n\":{},\"rose\":{},\"median_change\":{}}}", items.join(","), rows.len(), ch.iter().filter(|&&x| x > 0.0).count(), num(stats::median(&ch))))
}

/// spells: 값이 threshold 이상(--below면 이하)으로 이어진 구간마다 시작·끝·점 수·극값. 마지막 구간이 진행 중이면 ongoing.
fn spells_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let pts = load_with(path, o)?;
    let th: f64 = o.get("threshold").and_then(|v| v.parse().ok()).ok_or("--threshold 가 필요")?;
    let below = o.contains_key("below");
    let hit = |v: f64| if below { v <= th } else { v >= th };
    let mut items = Vec::new();
    let mut i = 0;
    while i < pts.len() {
        if !hit(pts[i].value) { i += 1; continue; }
        let s = i;
        while i < pts.len() && hit(pts[i].value) { i += 1; }
        let run = &pts[s..i];
        let ext = run.iter().fold(&run[0], |a, p| if (p.value > a.value) != below { p } else { a });
        items.push(format!(
            "{{\"start\":\"{}\",\"end\":\"{}\",\"points\":{},\"days\":{},\"extreme\":{},\"extreme_date\":\"{}\",\"ongoing\":{}}}",
            run[0].date, run[run.len() - 1].date, run.len(), run[run.len() - 1].day - run[0].day, ext.value, ext.date, i == pts.len()
        ));
    }
    Ok(format!("{{\"threshold\":{th},\"below\":{below},\"n\":{},\"spells\":[{}]}}", items.len(), items.join(",")))
}

/// lead: 신호 구간(--threshold, --below, --gap 365일 안 재진입은 한 구간) 시작 뒤 --event 계열(0/1) 사건이 시작되기까지(lead.rs)
fn lead_rows(path: &str, o: &HashMap<String, String>) -> Result<Vec<lead::Lead>, String> {
    let th: f64 = o.get("threshold").and_then(|v| v.parse().ok()).ok_or("--threshold 가 필요")?;
    let gap: i64 = o.get("gap").and_then(|v| v.parse().ok()).unwrap_or(365);
    let ev = series::load(o.get("event").ok_or("--event <0/1 계열 csv> 가 필요")?)?;
    Ok(lead::leads(&load_with(path, o)?, th, o.contains_key("below"), gap, &ev))
}

fn lead_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let rows = lead_rows(path, o)?;
    let done: Vec<f64> = rows.iter().filter(|r| r.event.is_some()).map(|r| r.lead_days as f64 / 30.44).collect();
    let items: Vec<String> = rows.iter().map(|r| format!("{{\"start\":{},\"end\":{},\"days\":{},\"event\":{},\"lead_months\":{},\"after_end_months\":{}}}",
        s(&r.start), s(&r.end), r.days, r.event.as_deref().map(s).unwrap_or("null".into()), num(Some(r.lead_days as f64 / 30.44)),
        num(Some((r.lead_days - r.days) as f64 / 30.44)))).collect();
    Ok(format!("{{\"n\":{},\"followed\":{},\"median_lead_months\":{},\"min_lead_months\":{},\"max_lead_months\":{},\"episodes\":[{}]}}",
        rows.len(), done.len(), num(stats::median(&done)), num(done.iter().cloned().reduce(f64::min)), num(done.iter().cloned().reduce(f64::max)), items.join(",")))
}

/// seasonal: 해마다 달별 월평균이 그해 평균보다 몇 % 높았나, 달별 중앙값과 높았던 해의 비율
fn seasonal_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let t = seasonal::table(&monthly_avg(&load_with(path, o)?));
    let b = seasonal::by_month(&t);
    let months: Vec<String> = b.iter().enumerate().map(|(k, (m, up))| format!("{{\"month\":{},\"median_dev\":{},\"share_above\":{}}}", k + 1, num(Some(*m)), num(Some(*up)))).collect();
    let (lo, hi) = (b.iter().enumerate().min_by(|a, c| a.1 .0.partial_cmp(&c.1 .0).unwrap()).unwrap().0 + 1, b.iter().enumerate().max_by(|a, c| a.1 .0.partial_cmp(&c.1 .0).unwrap()).unwrap().0 + 1);
    Ok(format!("{{\"years\":{},\"first_year\":{},\"last_year\":{},\"cheapest_month\":{lo},\"dearest_month\":{hi},\"months\":[{}]}}",
               t.len(), t.first().map_or(0, |y| y.year), t.last().map_or(0, |y| y.year), months.join(",")))
}

fn figure_cmd(frame: &str, path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let mut pts = load_with(path, o)?;
    if let Some(from) = o.get("from").and_then(|d| series::day_number(d)) {
        pts.retain(|p| p.day >= from); // 그림 기간(예: 2000년 이후)
    }
    let title = o.get("title").cloned().unwrap_or_default();
    // 그림 옆에 둘 자료(CSV): 기본은 그림에 쓴 계열 그대로, 사례 그림은 사례 표. 독자·다른 사이트가 숫자를 검증·인용하게(Dataset)
    let mut csv = format!("date,value\n{}", pts.iter().map(|p| format!("{},{}\n", p.date, p.value)).collect::<String>());
    let svg = match frame {
        "underwater" => {
            let lines: Vec<f64> = o.get("lines").map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
            let flag = match o.get("flag") { Some(f) => series::load(f)?, None => vec![] };
            let cut = lines.iter().cloned().fold(f64::MAX, f64::min);
            let marks: Vec<(i64, f64)> = if cut < f64::MAX {
                episodes::drawdowns(&pts, cut).iter().filter(|e| e.recovered.is_some()).map(|e| (series::day_number(&e.low_date).unwrap_or(0), e.low)).collect()
            } else { vec![] };
            viz::underwater(&turning::monthly(&pts), &lines, &flag, &marks, &title)
        }
        "paths" => {
            let d: Vec<String> = o.get("dates").map(|v| v.split(',').map(String::from).collect()).unwrap_or_default();
            let labels: Vec<String> = o.get("labels").map(|v| v.split(',').map(String::from).collect()).unwrap_or_default();
            viz::paths_with(&pts, &d, o.get("days").and_then(|v| v.parse().ok()).unwrap_or(365), &title, o.contains_key("diff"), &labels)
        }
        "band" => viz::band(&turning::monthly(&pts), &title, o.contains_key("pct"), o.get("suffix").map(String::as_str).unwrap_or("")),
        "dots" => {
            let eps = episodes::drawdowns(&pts, opt_f(o, "threshold", 0.10));
            let items: Vec<(i64, f64, bool)> = eps.iter().map(|e| (series::day_number(&e.peak).unwrap(), e.low, e.recovered.is_none())).collect();
            viz::dots(&items, opt_f(o, "deep", 0.20), &pts[0].date[..4], &pts.last().unwrap().date[..4], &title)
        }
        "response" => {
            let fol = series::load(o.get("follower").ok_or("--follower <csv> 가 필요")?)?;
            let (w, th) = (o.get("window").and_then(|v| v.parse().ok()).unwrap_or(63), opt_f(o, "threshold", 0.20));
            let weeks: i64 = o.get("weeks").and_then(|v| v.parse().ok()).unwrap_or(20);
            let offsets: Vec<i64> = (0..=weeks).map(|k| k * 7).collect();
            let start = fol[0].day;
            let path = |evs: Vec<(usize, usize, f64)>| -> Vec<Vec<f64>> {
                evs.into_iter().filter(|e| pts[e.1].day >= start).map(|(_, ext, _)| {
                    offsets.iter().map(|d| if *d == 0 { 0.0 } else { forward::after(&fol, pts[ext].day, *d).unwrap_or(f64::NAN) }).collect()
                }).collect()
            };
            let groups = vec![
                (o.get("up-label").cloned().unwrap_or("After big jumps".into()), "#eb6834", path(episodes::spikes(&pts, w, th))),
                (o.get("down-label").cloned().unwrap_or("After big drops".into()), "#2563eb", path(episodes::drops(&pts, w, th))),
            ];
            viz::response(&groups, &offsets, &title)
        }
        "fan" => {
            // 사건 날짜들 뒤의 경로를 중앙값 선 + 사분위 띠로(강조할 '지금'이 없는 참조군 그림, 예: 중간선거 13번)
            let weeks: i64 = o.get("weeks").and_then(|v| v.parse().ok()).unwrap_or(26);
            let offsets: Vec<i64> = (0..=weeks).map(|k| k * 7).collect();
            let starts: Vec<i64> = o.get("dates").ok_or("--dates 가 필요")?.split(',').filter_map(series::day_number).collect();
            let paths: Vec<Vec<f64>> = starts.iter().map(|&d0| offsets.iter().map(|d| if *d == 0 { 0.0 } else { forward::after(&pts, d0, *d).unwrap_or(f64::NAN) }).collect()).collect();
            let mut groups = vec![(o.get("label").cloned().unwrap_or("Events".into()), "#2563eb", paths)];
            if o.contains_key("base") {
                // 기저율: 21번째 점마다 시작한 아무 구간(같은 길이)의 경로(A5: 평소와 견준다)
                let any: Vec<Vec<f64>> = pts.iter().step_by(21).filter(|p| p.day + offsets[offsets.len() - 1] <= pts[pts.len() - 1].day)
                    .map(|p| offsets.iter().map(|d| if *d == 0 { 0.0 } else { forward::after(&pts, p.day, *d).unwrap_or(f64::NAN) }).collect()).collect();
                groups.insert(0, (o.get("base-label").cloned().unwrap_or("Any time".into()), "#94a3b8", any));
            }
            viz::response(&groups, &offsets, &title)
        }
        "cases" => {
            // 사건마다 한 줄: 도중 최저점(첫 시점까지)·첫 시점·둘째 시점, 세로선은 첫 시점의 기저율 중앙값
            // --rows "이름:도중최저:첫값:둘째값;…" 여러 계열의 사례(예: 대형 상장 9곳)를 엔진 events 결과로 받아 한 그림에
            if let Some(r) = o.get("rows") {
                let rows: Vec<(String, f64, f64, Option<f64>)> = r.split(';').filter_map(|x| {
                    let p: Vec<&str> = x.split(':').collect();
                    Some((p.first()?.to_string(), p.get(1)?.parse().ok()?, p.get(2)?.parse().ok()?, p.get(3).and_then(|v| v.parse().ok())))
                }).collect();
                let names: Vec<&str> = o.get("names").map(|v| v.split(',').collect()).unwrap_or(vec!["Worst dip", "6 months", "1 year"]);
                csv = format!("case,worst,first,second\n{}", rows.iter().map(|r| format!("{},{},{},{}\n", r.0, r.1, r.2, r.3.map(|v| v.to_string()).unwrap_or_default())).collect::<String>());
                let svg = viz::cases(&rows, o.get("base").and_then(|v| v.parse().ok()).unwrap_or(f64::NAN), [names[0], names[1], names.get(2).copied().unwrap_or("")], &title);
                let out = o.get("out").ok_or("--out <svg 파일> 이 필요")?;
                std::fs::write(out, &svg).map_err(|e| format!("{out}: {e}"))?;
                if let Some(stem) = out.strip_suffix(".svg") { std::fs::write(format!("{stem}.csv"), &csv).map_err(|e| e.to_string())?; }
                return Ok(format!("{{\"frame\":\"cases\",\"out\":{},\"bytes\":{}}}", s(out), svg.len()));
            }
            let dates: Vec<&str> = o.get("dates").ok_or("--dates 가 필요")?.split(',').collect();
            let hz = list(o, "forward", vec![182, 365]);
            let (h1, h2) = (hz[0], *hz.get(1).unwrap_or(&hz[0]));
            let rows: Vec<(String, f64, f64, Option<f64>)> = dates.iter().filter_map(|d| {
                let e = series::day_number(d)?;
                Some((d[..4].to_string(), forward::worst(&pts, e, h1)?, forward::after(&pts, e, h1)?, forward::after(&pts, e, h2)))
            }).collect();
            let base = events::study(&pts, &[pts[0].day], &[h1], false)[0].base_median.unwrap_or(0.0);
            let names: Vec<&str> = o.get("names").map(|v| v.split(',').collect()).unwrap_or(vec!["Worst dip", "6 months", "1 year"]);
            csv = format!("case,worst_along_way,change_{h1}d,change_{h2}d\n{}", rows.iter().map(|r| format!("{},{:.4},{:.4},{}\n", r.0, r.1, r.2, r.3.map(|v| format!("{v:.4}")).unwrap_or_default())).collect::<String>());
            viz::cases(&rows, base, [names[0], names[1], names[2]], &title)
        }
        "during" => {
            // 다른 계열이 빠진 구간마다 한 줄: 그 계열의 하락(주황)과 이 계열의 변화(파랑), 세로 점선은 이 계열 변화의 중앙값
            let rows = during_rows(&pts, o)?;
            let ch: Vec<f64> = rows.iter().map(|r| r.2).collect();
            let names: Vec<&str> = o.get("names").map(|v| v.split(',').collect()).unwrap_or(vec!["Stocks", "Gold", ""]);
            csv = format!("start_year,other_series_fall,this_series_change\n{}", rows.iter().map(|r| format!("{},{:.4},{:.4}\n", r.0, r.1, r.2)).collect::<String>());
            let r: Vec<(String, f64, f64, Option<f64>)> = rows.into_iter().map(|(y, f, c)| (y, f, c, None)).collect();
            viz::cases(&r, stats::median(&ch).unwrap_or(0.0), [names[0], names[1], names.get(2).copied().unwrap_or("")], &title)
        }
        "pictogram" => {
            // --groups "이름:칠할 수:전체 수:큰 글씨,…" (값은 엔진·패키지 계산에서 온 것을 넣는다. 계열은 쓰지 않는다)
            let g: Vec<(String, usize, usize, String)> = o.get("groups").ok_or("--groups 가 필요")?.split(',').filter_map(|x| {
                let p: Vec<&str> = x.split(':').collect();
                Some((p.first()?.to_string(), p.get(1)?.parse().ok()?, p.get(2)?.parse().ok()?, p.get(3).unwrap_or(&"").to_string()))
            }).collect();
            csv = format!("group,filled,total\n{}", g.iter().map(|x| format!("{},{},{}\n", x.0, x.1, x.2)).collect::<String>());
            viz::pictogram(&g, &title)
        }
        "spells" => {
            // --threshold x [--min-days 180]: 기준 이상 구간을 띠로, 길이 글자(spells 명령과 같은 구간)
            let th: f64 = o.get("threshold").and_then(|v| v.parse().ok()).ok_or("--threshold 가 필요")?;
            let md: i64 = o.get("min-days").and_then(|v| v.parse().ok()).unwrap_or(180);
            // --now 0.29 --now-label "At this week's rate": 마지막 자료 뒤 오늘 추정 점
            let now = o.get("now").and_then(|v| v.parse().ok()).map(|v: f64| (v, o.get("now-label").map(String::as_str).unwrap_or("Now")));
            if let Some((v, _)) = now { csv.push_str(&format!("now,{v}\n")); }
            viz::spells(&pts, th, md, now, &title)
        }
        "strip" => {
            // --rows "이름:값:1(강조)|0;…" --names "강조 묶음,나머지" (값은 엔진 position 결과)
            let rows: Vec<(String, f64, bool)> = o.get("rows").ok_or("--rows 가 필요")?.split(';').filter_map(|x| {
                let p: Vec<&str> = x.split(':').collect();
                Some((p.first()?.to_string(), p.get(1)?.parse().ok()?, p.get(2) == Some(&"1")))
            }).collect();
            let names: Vec<&str> = o.get("names").map(|v| v.split(',').collect()).unwrap_or(vec!["Group", "Others"]);
            csv = format!("name,value,highlight\n{}", rows.iter().map(|r| format!("{},{},{}\n", r.0, r.1, r.2 as u8)).collect::<String>());
            viz::strip(&rows, [names[0], names.get(1).copied().unwrap_or("Others")], &title)
        }
        "multiples" => {
            // --dates 고점 날짜들(또는 --threshold로 하락 사례 자동), --days 고점 뒤 기간. 마지막(진행 중) 사례가 '지금'
            let days: i64 = o.get("days").and_then(|v| v.parse().ok()).unwrap_or(730);
            let starts: Vec<(String, i64)> = match o.get("dates") {
                Some(d) => d.split(',').filter_map(|x| Some((x[..4].to_string(), series::day_number(x)?))).collect(),
                None => episodes::drawdowns(&pts, opt_f(o, "threshold", 0.4)).iter().filter_map(|e| Some((e.peak[..4].to_string(), series::day_number(&e.peak)?))).collect(),
            };
            let cases: Vec<(String, Vec<(f64, f64)>)> = starts.iter().filter_map(|(name, d0)| {
                let i = pts.iter().position(|p| p.day >= *d0)?;
                let w: Vec<&series::Point> = pts[i..].iter().take_while(|p| p.day - pts[i].day <= days).collect();
                let step = (w.len() / 50).max(1);
                Some((name.clone(), w.iter().enumerate().filter(|(k, _)| k % step == 0 || *k + 1 == w.len()).map(|(_, p)| ((p.day - pts[i].day) as f64, p.value / pts[i].value - 1.0)).collect()))
            }).collect();
            csv = format!("case,day,change\n{}", cases.iter().flat_map(|(n, v)| v.iter().map(move |(d, c)| format!("{n},{d},{c:.4}\n"))).collect::<String>());
            viz::multiples(&cases, days as f64, &title)
        }
        "gauge" => {
            // 지금 값이 이 계열 전체 범위의 어디(백분위). --label-now/--label-lo/--label-hi로 글자를 바꿀 수 있다
            let last = pts.last().ok_or("자료 없음")?;
            let rank = pts.iter().filter(|p| p.value <= last.value).count() as f64 / pts.len() as f64;
            let lo = pts.iter().min_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
            let hi = pts.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
            csv = format!("point,date,value\nlow,{},{}\nhigh,{},{}\nnow,{},{}\nrank,,{rank:.4}\n", lo.date, lo.value, hi.date, hi.value, last.date, last.value);
            let d = |k: &str, v: String| o.get(k).cloned().unwrap_or(v);
            viz::gauge(rank, &d("label-now", format!("Higher than {}% of months", (rank * 100.0).round())), &d("label-lo", format!("Low ({})", &lo.date[..4])), &d("label-hi", format!("High ({})", &hi.date[..4])), &title)
        }
        "calendar" => {
            // 해 × 달 계절 지수(seasonal), 맨 아래 달별 중앙값. --scale 색이 끝까지 진해지는 크기(기본 0.1)
            let t = seasonal::table(&monthly_avg(&pts));
            let b = seasonal::by_month(&t);
            let rows: Vec<(i32, [f64; 12])> = t.iter().map(|y| (y.year, y.dev)).collect();
            csv = format!("year,{}\n{}", (1..=12).map(|m| format!("m{m}")).collect::<Vec<_>>().join(","),
                          rows.iter().map(|(y, d)| format!("{y},{}\n", d.iter().map(|v| format!("{v:.4}")).collect::<Vec<_>>().join(","))).collect::<String>());
            viz::calendar(&rows, &b.iter().map(|x| x.0).collect::<Vec<_>>(), opt_f(o, "scale", 0.10), &title)
        }
        "lead" => {
            // lead 명령과 같은 구간: 줄마다 신호 시작(0)부터 사건 시작까지 막대, 사건이 아직 없으면 파랑 'so far'
            let rows = lead_rows(path, o)?;
            csv = format!("signal_start,signal_end,event_start,lead_months\n{}", rows.iter().map(|r| format!("{},{},{},{:.1}\n", r.start, r.end, r.event.clone().unwrap_or_default(), r.lead_days as f64 / 30.44)).collect::<String>());
            let min: i64 = o.get("min-days").and_then(|v| v.parse().ok()).unwrap_or(0);
            // --from-end: 신호가 끝난 날(곡선이 다시 플러스로)부터 사건까지
            let fe = o.contains_key("from-end");
            viz::lead(&rows.iter().filter(|r| r.days >= min || r.event.is_none())
                .map(|r| (r.start[..4].to_string(), (r.lead_days - if fe { r.days } else { 0 }) as f64 / 30.44, r.event.is_some())).collect::<Vec<_>>(), &title)
        }
        "timeline" => {
            // --items "해|사건|결과|good·mixed·bad;…" (글의 주장 장부에 출처가 있는 사실만 넣는다)
            let it: Vec<(String, String, String, String)> = o.get("items").ok_or("--items 가 필요")?.split(';').filter_map(|x| {
                let p: Vec<&str> = x.split('|').map(str::trim).collect();
                Some((p.first()?.to_string(), p.get(1)?.to_string(), p.get(2)?.to_string(), p.get(3).unwrap_or(&"mixed").to_string()))
            }).collect();
            csv = format!("year,event,outcome,tone\n{}", it.iter().map(|x| format!("{},\"{}\",\"{}\",{}\n", x.0, x.1, x.2, x.3)).collect::<String>());
            viz::timeline(&it, &title)
        }
        _ => return Err(format!("모르는 프레임: {frame} (underwater|paths|band|dots|response|fan|cases|during|pictogram|timeline|calendar|gauge|multiples|strip|lead)")),
    };
    let out = o.get("out").ok_or("--out <svg 파일> 이 필요")?;
    std::fs::write(out, &svg).map_err(|e| format!("{out}: {e}"))?;
    if let Some(stem) = out.strip_suffix(".svg") {
        std::fs::write(format!("{stem}.csv"), &csv).map_err(|e| format!("{stem}.csv: {e}"))?;
        // 다시 그릴 수 있게: 그림을 만든 프레임·입력·옵션(갱신 페이지가 같은 명령으로 다시 그린다)
        let mut keys: Vec<_> = o.iter().filter(|(k, _)| *k != "out").collect();
        keys.sort();
        let params: Vec<String> = keys.iter().map(|(k, v)| format!("{}:{}", s(k), s(v))).collect();
        std::fs::write(format!("{stem}.json"), format!("{{\"frame\":{},\"input\":{},\"params\":{{{}}}}}\n", s(frame), s(path), params.join(",")))
            .map_err(|e| format!("{stem}.json: {e}"))?;
    }
    Ok(format!("{{\"frame\":{},\"out\":{},\"bytes\":{}}}", s(frame), s(out), svg.len()))
}

fn list(o: &HashMap<String, String>, k: &str, d: Vec<i64>) -> Vec<i64> {
    o.get(k).map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(d)
}

fn spikes_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let pts = load_with(path, o)?;
    let w = o.get("window").and_then(|v| v.parse().ok()).unwrap_or(63);
    let th = opt_f(o, "threshold", 0.30);
    let fwd = list(o, "forward", vec![91, 182, 365]);
    let sp = episodes::spikes(&pts, w, th);
    let mut j = String::from("{\"spikes\":[");
    let mut gave = Vec::new();
    for (k, &(i, lo, g)) in sp.iter().enumerate() {
        let horizon = *fwd.last().unwrap_or(&365);
        let top = pts[i..].iter().take_while(|p| p.day - pts[i].day <= horizon).fold(&pts[i], |a, p| if p.value > a.value { p } else { a });
        let f: Vec<String> = fwd.iter().map(|d| format!("\"{d}\":{}", num(forward::after(&pts, pts[i].day, *d)))).collect();
        if let Some(x) = forward::after(&pts, pts[i].day, horizon) { gave.push(x); }
        let _ = write!(j, "{}{{\"start\":{},\"from_low_date\":{},\"gain_at_start\":{},\"top\":{},\"top_date\":{},\"top_gain_from_low\":{},\"forward\":{{{}}}}}",
                       if k > 0 { "," } else { "" }, s(&pts[i].date), s(&pts[lo].date), num(Some(g)), num(Some(top.value)), s(&top.date),
                       num(Some(top.value / pts[lo].value - 1.0)), f.join(","));
    }
    let _ = write!(j, "],\"summary\":{{\"n\":{},\"median_change_last_horizon\":{},\"fell_back\":{}}}}}", sp.len(), num(stats::median(&gave)), gave.iter().filter(|&&x| x < 0.0).count());
    Ok(j)
}

fn monthly_avg(pts: &[series::Point]) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64, usize)> = Vec::new();
    for p in pts {
        match out.last_mut() {
            Some(l) if l.0 == p.date[..7] => { l.1 += p.value; l.2 += 1; }
            _ => out.push((p.date[..7].to_string(), p.value, 1)),
        }
    }
    out.into_iter().map(|(d, v, n)| (d, v / n as f64)).collect()
}

fn recession_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let spread = monthly_avg(&load_with(path, o)?);
    let flag: HashMap<String, f64> = series::load(o.get("flag").ok_or("--flag <USREC csv> 가 필요")?)?.iter().map(|p| (p.date[..7].to_string(), p.value)).collect();
    let ahead = o.get("ahead").and_then(|v| v.parse().ok()).unwrap_or(12usize);
    let (mut x, mut y) = (vec![], vec![]);
    for (i, (_, v)) in spread.iter().enumerate() {
        if let Some(f) = spread.get(i + ahead).and_then(|(d, _)| flag.get(d)) {
            x.push(*v);
            y.push(*f);
        }
    }
    let (a, b, it) = probit::fit(&x, &y);
    let (d, now) = spread.last().unwrap();
    Ok(format!("{{\"a\":{},\"b\":{},\"iterations\":{it},\"n\":{},\"recession_months\":{},\"sample\":[{},{}],\"latest_month\":{},\"latest_spread\":{},\"probability\":{},\"ahead_months\":{ahead}}}",
               num(Some(a)), num(Some(b)), x.len(), y.iter().filter(|&&v| v >= 1.0).count(), s(&spread[0].0), s(&spread[x.len() - 1].0), s(d), num(Some(*now)),
               num(Some(probit::cdf(a + b * now)))))
}

fn analogs_cmd(paths: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let yoy = o.get("mode").map(|m| m == "yoy").unwrap_or(true);
    let mut cols: Vec<HashMap<String, f64>> = Vec::new();
    for p in paths.split(',') {
        let m = turning::monthly(&series::load(p)?);
        let m = if yoy { series::change_over(&m, 12) } else { m };
        cols.push(m.into_iter().map(|p| (p.date[..7].to_string(), p.value)).collect());
    }
    let mut months: Vec<String> = cols[0].keys().filter(|d| cols.iter().all(|c| c.contains_key(*d))).cloned().collect();
    months.sort();
    let rows: Vec<Vec<f64>> = months.iter().map(|d| cols.iter().map(|c| c[d]).collect()).collect();
    if rows.len() < 30 { return Err("겹치는 달이 너무 적다".into()); }
    let k = o.get("k").and_then(|v| v.parse().ok()).unwrap_or(5);
    let ex = o.get("exclude").and_then(|v| v.parse().ok()).unwrap_or(24);
    let ahead = o.get("ahead").and_then(|v| v.parse().ok()).unwrap_or(12usize);
    let target: Option<HashMap<String, f64>> = match o.get("target") {
        Some(t) => Some(turning::monthly(&series::load(t)?).into_iter().map(|p| (p.date[..7].to_string(), p.value)).collect()),
        None => None,
    };
    let shift = |d: &str, n: usize| { let (y, m): (i32, i32) = (d[..4].parse().unwrap(), d[5..7].parse().unwrap()); let t = y * 12 + m - 1 + n as i32; format!("{}-{:02}", t / 12, t % 12 + 1) };
    let mut j = format!("{{\"now\":{},\"now_values\":[{}],\"analogs\":[", s(months.last().unwrap()), rows.last().unwrap().iter().map(|v| num(Some(*v))).collect::<Vec<_>>().join(","));
    for (n, (i, dist)) in analogs::nearest(&rows, k, ex).iter().enumerate() {
        let after = target.as_ref().and_then(|t| Some(t.get(&shift(&months[*i], ahead))? / t.get(&months[*i])? - 1.0));
        let _ = write!(j, "{}{{\"month\":{},\"distance\":{},\"values\":[{}],\"target_after\":{}}}", if n > 0 { "," } else { "" }, s(&months[*i]), num(Some(*dist)),
                       rows[*i].iter().map(|v| num(Some(*v))).collect::<Vec<_>>().join(","), num(after));
    }
    j.push_str("]}");
    Ok(j)
}

fn passthrough_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let lead = load_with(path, o)?;
    let fol = series::load(o.get("follower").ok_or("--follower <csv> 가 필요")?)?;
    let (w, th) = (o.get("window").and_then(|v| v.parse().ok()).unwrap_or(63), opt_f(o, "threshold", 0.20));
    let hz = list(o, "forward", vec![14, 28, 56, 91]);
    let start = fol[0].day;
    let mut j = String::from("{");
    for (name, evs) in [("up", episodes::spikes(&lead, w, th)), ("down", episodes::drops(&lead, w, th))] {
        let evs: Vec<_> = evs.into_iter().filter(|e| lead[e.0].day >= start).collect();
        let _ = write!(j, "\"{name}\":{{\"n\":{},\"events\":[", evs.len());
        let mut by_h: Vec<Vec<f64>> = vec![vec![]; hz.len()];
        let mut lead_h: Vec<Vec<f64>> = vec![vec![]; hz.len()];
        let (mut moves, mut during, mut during_raw) = (vec![], vec![], vec![]);
        let mut raw_h: Vec<Vec<f64>> = vec![vec![]; hz.len()];
        for (k, &(i, ext, g)) in evs.iter().enumerate() {
            // 선행 지표가 움직이는 동안(직전 저점·고점 -> 문턱을 넘은 날) 후행 지표가 이미 따라온 몫
            if let Some(x) = forward::after(&fol, lead[ext].day, lead[i].day - lead[ext].day) { during.push(x / g.abs()); during_raw.push(x); }
            // 선행 지표가 문턱을 넘은 날(뉴스가 된 날)부터 후행 지표가 N일 동안 더 움직인 양. 반영 비율 = 그 양 / 선행 지표가 이미 움직인 폭
            moves.push(g);
            let f: Vec<String> = hz.iter().enumerate().map(|(h, d)| {
                let x = forward::after(&fol, lead[i].day, *d);
                let l = forward::after(&lead, lead[i].day, *d);
                if let (Some(x), Some(l)) = (x, l) { by_h[h].push(x / g.abs()); lead_h[h].push(l); raw_h[h].push(x); }
                format!("\"{d}\":{}", num(x))
            }).collect();
            let _ = write!(j, "{}{{\"date\":{},\"from\":{},\"lead_move\":{},\"follower\":{{{}}}}}", if k > 0 { "," } else { "" }, s(&lead[i].date), s(&lead[ext].date), num(Some(g)), f.join(","));
        }
        let med: Vec<String> = hz.iter().enumerate().map(|(h, d)| format!("\"{d}\":{{\"follower_change\":{},\"follower_per_lead_move\":{},\"leader_after\":{},\"n\":{}}}", num(stats::median(&raw_h[h])), num(stats::median(&by_h[h])), num(stats::median(&lead_h[h])), by_h[h].len())).collect();
        let _ = write!(j, "],\"median_lead_move\":{},\"median_during_per_lead_move\":{},\"median_during_change\":{},\"median\":{{{}}}}},", num(stats::median(&moves)), num(stats::median(&during)), num(stats::median(&during_raw)), med.join(","));
    }
    j.pop();
    j.push('}');
    Ok(j)
}

fn cycles_cmd(paths: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let mut pts = Vec::new();
    for p in paths.split(',') {
        let mut q = series::load(p)?;
        if let Some(last) = pts.last().map(|x: &series::Point| x.day) {
            q.retain(|x| x.day > last);
        }
        pts.extend(q);
    }
    let flag = match o.get("flag") { Some(f) => series::load(f)?, None => vec![] };
    let within = o.get("within").and_then(|v| v.parse().ok()).unwrap_or(730i64);
    let mut j = String::from("{\"cycles\":[");
    for (k, c) in cycles::cycles(&pts).iter().enumerate() {
        let a = series::day_number(&c.start).unwrap();
        let rec = if flag.is_empty() { "null".to_string() } else {
            let last_flag = flag.last().map(|f| f.day).unwrap_or(0);
            if a + within > last_flag { "null".into() } else { flag.iter().any(|f| f.day > a && f.day <= a + within && f.value >= 1.0).to_string() }
        };
        let _ = write!(j, "{}{{\"direction\":{},\"start\":{},\"end\":{},\"moves\":{},\"from\":{},\"to\":{},\"recession_within\":{rec}}}",
                       if k > 0 { "," } else { "" }, s(if c.up { "up" } else { "down" }), s(&c.start), s(&c.end), c.moves, num(Some(c.from)), num(Some(c.to)));
    }
    j.push_str("]}");
    Ok(j)
}

pub fn monthly_payment(loan: f64, annual_pct: f64, years: f64) -> f64 {
    let (r, n) = (annual_pct / 1200.0, years * 12.0);
    if r == 0.0 { loan / n } else { loan * r / (1.0 - (1.0 + r).powf(-n)) }
}

fn payment_cmd(o: &HashMap<String, String>) -> Result<String, String> {
    let loan = opt_f(o, "loan", 300000.0);
    let years = opt_f(o, "years", 30.0);
    let rates: Vec<f64> = o.get("rates").ok_or("--rates a,b 가 필요")?.split(',').filter_map(|x| x.parse().ok()).collect();
    let p: Vec<String> = rates.iter().map(|r| format!("{{\"rate\":{r},\"monthly\":{}}}", num(Some((monthly_payment(loan, *r, years) * 100.0).round() / 100.0)))).collect();
    Ok(format!("{{\"loan\":{loan},\"years\":{years},\"payments\":[{}],\"method\":\"Fixed-rate monthly payment: loan x r / (1 - (1 + r)^-n), r = annual rate / 12, n = months.\"}}", p.join(",")))
}

fn export_cmd(path: &str, o: &HashMap<String, String>) -> Result<String, String> {
    let mut m = turning::monthly(&load_with(path, o)?);
    if let Some(y) = o.get("from") {
        m.retain(|p| p.date[..4] >= y[..]);
    }
    let first = m.first().ok_or("자료 없음")?;
    let (mut y, mut mo) = (first.date[..4].parse::<i32>().unwrap(), first.date[5..7].parse::<i32>().unwrap());
    let mut vals = Vec::new();
    let mut k = 0;
    let mut last = first.value;
    while k < m.len() {
        let key = format!("{y}-{mo:02}");
        if m[k].date[..7] == key {
            last = m[k].value;
            k += 1;
        }
        vals.push(format!("{}", (last * 1e4).round() / 1e4));
        mo += 1;
        if mo > 12 { mo = 1; y += 1; }
    }
    let json = format!("{{\"start\":{},\"v\":[{}]}}", s(&first.date[..7]), vals.join(","));
    let out = o.get("out").ok_or("--out <json> 이 필요")?;
    std::fs::write(out, &json).map_err(|e| format!("{out}: {e}"))?;
    Ok(format!("{{\"out\":{},\"months\":{},\"start\":{}}}", s(out), vals.len(), s(&first.date[..7])))
}

/// 방법 한 줄(글 끝 "How we did this"의 재료)과 재현 정보.
fn with_meta(j: String, cmd: &str, path: &str, o: &HashMap<String, String>) -> String {
    let method = match cmd {
        "drawdowns" => "Episodes: every close at least the threshold below the prior record, followed to the lowest close and to a new record (peak-to-trough drawdown).",
        "position" => "Rank of the latest value among all observations in the period; 'last at or below' is the most recent earlier date with an equal or lower value.",
        "turning" => "Bull and bear phases dated with the Pagan and Sossounov (2003) rules on month-end values; the last turning point is provisional.",
        "sahm" => "Sahm rule: three-month average unemployment rate minus its low over the prior 12 months; 0.50 point or more signals recession (Sahm 2019).",
        "spikes" => "Spikes: first day a series closed the threshold or more above its low of the prior window, then its high and later changes.",
        "recession-prob" => "Probit model of a recession the given months ahead on the monthly average 10-year minus 3-month Treasury spread (Estrella and Mishkin form), fitted on our FRED data; in-sample.",
        "analogs" => "Nearest past months to today on the standardized values of the listed series, at least the exclusion window apart (nearest-neighbor analogs).",
        "passthrough" => "For every rise or fall of the threshold or more in the leading series within the window, the later change in the following series from the day the threshold was crossed, divided by the size of the leading move (pass-through).",
        "cycles" => "Policy-rate cycles: consecutive target-rate changes in the same direction grouped together; a new cycle starts when the direction reverses.",
        "export" => "Month-end values for the reader tool, carried forward over gaps.",
        "spells" => "Every stretch of consecutive observations at or above the threshold (or at or below it), with its first and last date, length and most extreme value; the last one may still be under way.",
        "lead" => "Signal episodes: consecutive days at or below the threshold (or at or above it), joined when the gap between them is the gap window or less and no event started in between; days inside an event do not count as signal; lead is the time from an episode's first day to the next start of the event series (a 0-to-1 step, e.g. NBER recession months).",
        "seasonal" => "Each month's average as a percent above or below that calendar year's average, for every complete year; then the median by month and the share of years each month was above its year's average.",
        "during" => "For every fall of the threshold or more in the other series (peak to lowest close), the change in this series over the same dates.",
        "events" => "Change over each window after every event date, compared with the same-length change from all start dates (base rate).",
        _ => "Chart frame drawn from the same data.",
    };
    let pts = series::load(path).ok();
    let (rows, first, last) = pts.as_ref().map(|p| (p.len(), p[0].date.clone(), p.last().unwrap().date.clone())).unwrap_or_default();
    let mut keys: Vec<_> = o.iter().filter(|(k, _)| *k != "out").collect();
    keys.sort();
    let params: Vec<String> = keys.iter().map(|(k, v)| format!("{}:{}", s(k), s(v))).collect();
    let mut j = j;
    j.pop();
    format!("{j},\"method\":{},\"provenance\":{{\"engine\":{},\"input\":{},\"rows\":{rows},\"first\":{},\"last\":{},\"params\":{{{}}}}}}}",
            s(method), s(env!("CARGO_PKG_VERSION")), s(path), s(&first), s(&last), params.join(","))
}

fn main() {
    let (pos, opt) = args();
    let cmd = pos.first().map(String::as_str).unwrap_or("");
    let out = match (cmd, pos.get(1), pos.get(2)) {
        ("drawdowns", Some(p), _) => drawdowns(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("position", Some(p), _) => position(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("turning", Some(p), _) => turning_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("sahm", Some(p), _) => sahm_cmd(p).map(|j| with_meta(j, cmd, p, &opt)),
        ("lead", Some(p), _) => lead_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("spells", Some(p), _) => spells_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("seasonal", Some(p), _) => seasonal_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("during", Some(p), _) => during_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("events", Some(p), _) => events_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("spikes", Some(p), _) => spikes_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("recession-prob", Some(p), _) => recession_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("analogs", Some(p), _) => analogs_cmd(p, &opt).map(|j| with_meta(j, cmd, p.split(',').next().unwrap(), &opt)),
        ("passthrough", Some(p), _) => passthrough_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("cycles", Some(p), _) => cycles_cmd(p, &opt).map(|j| with_meta(j, cmd, p.split(',').next().unwrap(), &opt)),
        ("payment", _, _) => payment_cmd(&opt),
        ("export", Some(p), _) => export_cmd(p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        ("figure", Some(f), Some(p)) => figure_cmd(f, p, &opt).map(|j| with_meta(j, cmd, p, &opt)),
        _ => Err("사용: mkviz-engine drawdowns|position|turning|sahm|events <csv> [옵션] | figure <frame> <csv> --out <svg> (main.rs 머리 주석)".into()),
    };
    match out {
        Ok(j) => println!("{j}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod payment_test {
    #[test]
    fn known_payment() {
        // $300,000, 30년, 6% -> $1,798.65 (표준 상환표 값)
        assert!((super::monthly_payment(300000.0, 6.0, 30.0) - 1798.65).abs() < 0.01);
    }
}
