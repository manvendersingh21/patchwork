//! Cypher templates: the agents' only read tools. Verified against FalkorDB before use.
//! Optional params are passed as empty strings: `$day = ''` means no day filter.
//! The path is returned as plain arrays (nodes, edges), not as a Path value, because the
//! falkordb 0.10.5 client fails to decode Path values.
pub const T1_TALKS_FOR_MY_ISSUES: &str = "\
MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})\
-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) \
WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) \
RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, \
[r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, \
size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, \
k.start_time AS start_time, s.name AS speaker \
ORDER BY day, start_time LIMIT 5";

pub const T2_TALKS_FOR_ISSUE: &str = "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2";
pub const T3_PEOPLE_TO_MEET: &str = "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5";
pub const T4_MEMORY_RECALL: &str = "MATCH (i:Issue {uid: $issue})<-[:HAS_ISSUE]-(:Repo)<-[:MAINTAINS|TRACKS]-(owner:Person) MATCH (owner)-[:LOGGED]->(e:Encounter)-[:MET]->(c:Person) MATCH p = (i)-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e)-[:MET]->(c) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3";
pub const T5_ISSUES_FOR_ENCOUNTER: &str = "MATCH (e:Encounter {uid: $encounter})<-[:LOGGED]-(owner:Person) MATCH (owner)-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'}) MATCH p = (e)-[:DISCUSSED]->(t:Topic)<-[:ABOUT]-(i) MATCH (e)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.uid AS issue_uid, i.title AS issue, c.uid AS contact_uid, c.name AS contact, t.name AS topic";
pub const T6_TALKS_BY_TOPIC: &str = "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5";
pub const T7_CONFERENCE_DAYS: &str = "MATCH (k:Talk) RETURN DISTINCT k.day AS day ORDER BY day";

/// The graph stores `day` as an ISO date, but models pass "day 2", "thursday",
/// "2026-10-29" and more. Map anything recognizable to the conference's ISO
/// dates; anything else becomes '' (no filter) so an odd format widens the
/// result instead of emptying it.
pub fn normalize_day(raw: &str) -> String {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return String::new();
    }
    if s.contains("2026-10-28") || s.contains("day 1") || s.contains("day1") || s.contains("first") || s.contains("wed") || s == "1" {
        "2026-10-28".into()
    } else if s.contains("2026-10-29") || s.contains("day 2") || s.contains("day2") || s.contains("second") || s.contains("thu") || s == "2" {
        "2026-10-29".into()
    } else {
        String::new()
    }
}

/// The graph compares `start_time` as 24h "HH:MM" strings. Accept "10:30 AM",
/// "9am", "12pm", "14:00" and bare hours; anything unparseable becomes ''.
pub fn normalize_after(raw: &str) -> String {
    let s = raw.trim().to_lowercase().replace(' ', "");
    if s.is_empty() {
        return String::new();
    }
    let am = s.ends_with("am");
    let pm = s.ends_with("pm");
    let core = s.trim_end_matches("am").trim_end_matches("pm");
    let (h, m) = match core.split_once(':') {
        Some((h, m)) => (h, m),
        None => (core, "0"),
    };
    let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) else {
        return String::new();
    };
    if m > 59 || ((am || pm) && !(1..=12).contains(&h)) || (!am && !pm && h > 23) {
        return String::new();
    }
    let h24 = if am {
        if h == 12 { 0 } else { h }
    } else if pm {
        if h == 12 { 12 } else { h + 12 }
    } else {
        h
    };
    format!("{h24:02}:{m:02}")
}

/// A day mention found in a user question: a real conference day, a
/// day-looking token that maps to nothing (carried verbatim so the caller can
/// correct the user), or no mention at all.
#[derive(Debug, PartialEq)]
pub enum DayMention { None, Valid(String), Invalid(String) }

/// Lowercase and split on anything that is not alphanumeric or a hyphen, so
/// date tokens like "2026-10-28" stay whole while punctuation vanishes.
pub fn tokenize(q: &str) -> Vec<String> {
    q.to_lowercase().split(|c: char| !c.is_alphanumeric() && c != '-').filter(|t| !t.is_empty()).map(str::to_string).collect()
}

/// True for exact YYYY-MM-DD shaped tokens ("2026-10-30"), false for anything else.
fn iso_date(t: &str) -> bool {
    let b = t.as_bytes();
    b.len() == 10 && b[..4].iter().all(|c| c.is_ascii_digit()) && b[4] == b'-'
        && b[5..7].iter().all(|c| c.is_ascii_digit()) && b[7] == b'-'
        && b[8..].iter().all(|c| c.is_ascii_digit())
}

/// Token-equality day-mention extractor: "trust" never matches "thu" and
/// "someday" never matches "day". Any Valid mention wins; otherwise the
/// earliest invalid token is reported; otherwise no mention.
pub fn day_from_question(question: &str) -> DayMention {
    let t = tokenize(question);
    let (mut valid, mut invalid) = (None, None);
    for (i, tok) in t.iter().enumerate() {
        let next = t.get(i + 1).map(|s| s.as_str());
        let prev = if i > 0 { Some(t[i - 1].as_str()) } else { None };
        let mention = if tok == "day" {
            if let Some(n) = next.filter(|n| n.chars().all(|c| c.is_ascii_digit())) {
                match n {
                    "1" => Some(DayMention::Valid("2026-10-28".into())),
                    "2" => Some(DayMention::Valid("2026-10-29".into())),
                    _ => Some(DayMention::Invalid(n.to_string())),
                }
            } else {
                let ord = [next, prev].into_iter().flatten().find(|s| matches!(*s, "first" | "second" | "third"));
                match ord {
                    Some("first") => Some(DayMention::Valid("2026-10-28".into())),
                    Some("second") => Some(DayMention::Valid("2026-10-29".into())),
                    Some(o) => Some(DayMention::Invalid(o.to_string())),
                    None => None,
                }
            }
        } else {
            match tok.as_str() {
                "wed" | "wednesday" => Some(DayMention::Valid("2026-10-28".into())),
                "thu" | "thurs" | "thursday" => Some(DayMention::Valid("2026-10-29".into())),
                "fri" | "friday" | "mon" | "monday" | "tue" | "tuesday" | "sat" | "saturday" | "sun" | "sunday" => Some(DayMention::Invalid(tok.clone())),
                "2026-10-28" => Some(DayMention::Valid("2026-10-28".into())),
                "2026-10-29" => Some(DayMention::Valid("2026-10-29".into())),
                _ if iso_date(tok) => Some(DayMention::Invalid(tok.clone())),
                _ => None,
            }
        };
        match mention {
            Some(DayMention::Valid(d)) => valid = valid.or(Some(d)),
            Some(DayMention::Invalid(raw)) => invalid = invalid.or(Some(raw)),
            _ => {}
        }
    }
    if let Some(d) = valid { DayMention::Valid(d) } else if let Some(raw) = invalid { DayMention::Invalid(raw) } else { DayMention::None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_forms() {
        assert_eq!(normalize_day("2"), "2026-10-29");
        assert_eq!(normalize_day("Day 2 "), "2026-10-29");
        assert_eq!(normalize_day("thursday"), "2026-10-29");
        assert_eq!(normalize_day("2026-10-29"), "2026-10-29");
        assert_eq!(normalize_day("day 1"), "2026-10-28");
        assert_eq!(normalize_day("wed"), "2026-10-28");
        assert_eq!(normalize_day(""), "");
        assert_eq!(normalize_day("someday"), "");
    }

    #[test]
    fn after_forms() {
        assert_eq!(normalize_after("10:30 AM"), "10:30");
        assert_eq!(normalize_after("9am"), "09:00");
        assert_eq!(normalize_after("12pm"), "12:00");
        assert_eq!(normalize_after("12am"), "00:00");
        assert_eq!(normalize_after("14:00"), "14:00");
        assert_eq!(normalize_after("2:10 pm"), "14:10");
        assert_eq!(normalize_after(""), "");
        assert_eq!(normalize_after("lunch"), "");
    }

    #[test]
    fn day_mentions() {
        assert_eq!(day_from_question("Which talks on day 2 morning help with my open issues?"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("day 3 friday"), DayMention::Invalid("3".into()));
        assert_eq!(day_from_question("day 1"), DayMention::Valid("2026-10-28".into()));
        assert_eq!(day_from_question("friday"), DayMention::Invalid("friday".into()));
        assert_eq!(day_from_question("Which talks about trust and auth?"), DayMention::None);
        assert_eq!(day_from_question("someday"), DayMention::None);
        assert_eq!(day_from_question("2026-10-30"), DayMention::Invalid("2026-10-30".into()));
        assert_eq!(day_from_question("2026-10-29"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("first day"), DayMention::Valid("2026-10-28".into()));
        assert_eq!(day_from_question("third day"), DayMention::Invalid("third".into()));
    }

    #[test]
    fn day_mention_token_boundaries() {
        assert_eq!(day_from_question("trust the thursday plan"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("what about thurs?"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("DAY 2, please"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("on the second day"), DayMention::Valid("2026-10-29".into()));
        assert_eq!(day_from_question("saturday"), DayMention::Invalid("saturday".into()));
        assert_eq!(day_from_question("day 10"), DayMention::Invalid("10".into()));
    }
}
