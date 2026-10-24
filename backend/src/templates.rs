//! Cypher templates: the agents' only read tools. Verified against FalkorDB before use.
//! Optional params are passed as empty strings: `$day = ''` means no day filter.
//! The path is returned as plain arrays (nodes, edges), not as a Path value, because the
//! falkordb 0.10.5 client fails to decode Path values.
pub const T1_TALKS_FOR_MY_ISSUES: &str = "\
MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})\
-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) \
WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) \
RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, \
[r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, \
size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, \
k.start_time AS start_time, s.name AS speaker \
ORDER BY day, start_time LIMIT 5";

pub const T2_TALKS_FOR_ISSUE: &str = "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2";
pub const T3_PEOPLE_TO_MEET: &str = "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5";
pub const T4_MEMORY_RECALL: &str = "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3";
pub const T5_ISSUES_FOR_ENCOUNTER: &str = "MATCH p = (e:Encounter {uid: $encounter})-[:DISCUSSED]->(t:Topic)<-[:ABOUT]-(i:Issue {gh_state: 'open'})<-[:HAS_ISSUE]-(:Repo)<-[:MAINTAINS|TRACKS]-(:Person {kind: 'self'}) MATCH (e)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.uid AS issue_uid, i.title AS issue, c.uid AS contact_uid, c.name AS contact, t.name AS topic";
pub const T6_TALKS_BY_TOPIC: &str = "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5";

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
}
